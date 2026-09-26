//! Производный владелец WorldServer-соединений у LoginServer из
//! `nets/netlogin/mynetserver_world.cpp`; Realm — listener World-соединений
//! Login-направления. Источник контракта — та же точная пара,
//! что у [`crate::app::login_message`]; доменные call sites `CGame` остаются
//! у владельца процесса.
//!
//! Owner создаёт accepted WorldServer через virtual-фабрику, обрабатывает общий
//! command snapshot и выдаёт конкретную FIFO. Долгоживущие Linux I/O actions
//! возвращаются runtime через общий `CServer`; этот owner не исполняет доменные
//! сообщения и не создаёт общий parser. Константы component: максимум
//! незавершённых send — `100`, per-client send-buffer limit — `0x1000000`.
//! `CGame` после `Host` записывал receive-rate/ban, max WorldServer count,
//! send limits, поздний backlog и таймаут первого пакета. Поле
//! `bWorldCheckMsgCon` также записывалось, но `CMyNetServerClient_World::
//! OnReceive` всегда проверял content CRC и не читал его; Rust не создаёт
//! неиспользуемый флаг.
//!
//! В отличие от client-направления, World receive-ошибки только очищают вход и
//! возвращаются в snapshot: component не добавляет IP в forbid-map и не ставит
//! `QUIT`. Общий receive-rate guard при включённом `bWorldCheckNet` сохраняет
//! собственный ban/QUIT независимо от parser. Missing identity callbacks у
//! этого производного сервера не переопределены и остаются no-op.
//!
//! Windows `SetSendRevBuf/SO_SNDBUF=0` остаётся локальной transport-задачей
//! принятого World-соединения; server-owner не подменяет её несовместимым
//! Linux socket option. `field +0x120 = 0` конструктора не связан с живым
//! состоянием и не получает пустого аналога.
//!
//! Доказательства: docs/reconstruction/realm-services.md#login-listeners

use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::net::TcpStream;

use nebokrai_shared::network::{
    AcceptStart, AdmissionOutcome, CMsgQueue, CServer, CServerClient, ComponentReceiveErrorAction,
    ServerCommandHandle, ServerComponentCallbacks, ServerHostError, ServerSnapshot,
};

use super::login_message::CMessage;
use super::login_world_server_client::{CMyNetServerClientWorld, WorldReceiveError};

const WORLD_DEFAULT_MAX_IN_FLIGHT_SENDS: i32 = 100;
const WORLD_DEFAULT_PERMITTED_SEND_BYTES: i32 = 0x100_0000;

/// Владелец общего `CServer` и WorldServer FIFO `CMessage`.
pub struct CMyNetServerWorld {
    base: CServer,
    received_messages: CMsgQueue<CMessage>,
}

impl CMyNetServerWorld {
    /// Создаёт World server с исходными component defaults.
    pub fn new(now_ms: u32) -> Self {
        let mut base = CServer::new(now_ms);
        base.configure_send_limits(
            WORLD_DEFAULT_MAX_IN_FLIGHT_SENDS,
            WORLD_DEFAULT_PERMITTED_SEND_BYTES,
        );
        Self {
            base,
            received_messages: CMsgQueue::new(),
        }
    }

    /// Создаёт Linux IPv4 listener через единственного transport-owner.
    pub fn host(
        &mut self,
        port: u32,
        address: Option<Ipv4Addr>,
        socket_type: i32,
        legacy_flag: bool,
    ) -> Result<(), ServerHostError> {
        self.base.host(port, address, socket_type, legacy_flag)
    }

    /// Применяет общий World receive-rate/ban guard из setup.
    pub fn configure_receive_guard(
        &mut self,
        enabled: bool,
        maximum_bytes_per_second: u32,
        forbid_time_ms: u32,
    ) {
        self.base.configure_receive_guard(
            enabled,
            maximum_bytes_per_second as i32,
            forbid_time_ms as i32,
        );
    }

    /// Применяет max connections, concurrent sends и send-buffer setup.
    pub fn configure_connection_limits(
        &mut self,
        maximum_clients: i32,
        maximum_in_flight_sends: i32,
        permitted_send_bytes: i32,
    ) {
        self.base.configure_max_clients(maximum_clients);
        self.base
            .configure_send_limits(maximum_in_flight_sends, permitted_send_bytes);
    }

    /// Повторяет две поздние setup-ex записи после уже выполненного `Host`.
    pub fn configure_accept_limits_after_host(
        &mut self,
        maximum_block_connections: i32,
        first_receive_timeout_ms: i32,
    ) {
        self.base.configure_accept_limits_after_host(
            maximum_block_connections,
            first_receive_timeout_ms,
        );
    }

    /// Сохраняет две исходные local-address записи после успешного `Host`.
    pub fn set_local_identity(&mut self, ip: &[u8], ipv4_word: u32) {
        self.base.set_local_identity(ip, ipv4_word);
    }

    /// Возвращает canonical dotted IPv4, который старый `CGame` читал из
    /// `s_pNetServer_World` при записи Login-строки `server_info`.
    pub fn local_ip(&self) -> &[u8] {
        self.base.local_ip()
    }

    /// Возвращает тот же x86 DWORD IPv4, который `CGame::Init` писал в ServLog.
    pub const fn local_ipv4_word(&self) -> u32 {
        self.base.local_ipv4_word()
    }

    /// Начинает одну общую accept-operation либо сообщает причину ожидания.
    pub fn begin_accept(&self) -> AcceptStart {
        self.base.begin_accept()
    }

    /// Выполняет admission через World virtual-фабрику и ставит общий `ADD`.
    pub fn queue_accepted(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
    ) -> AdmissionOutcome {
        self.base
            .queue_accepted_with(stream, peer, now_ms, CMyNetServerClientWorld::new_state)
    }

    /// Применяет один net-thread snapshot с конкретными World callbacks.
    pub fn process_network_snapshot(&mut self, now_ms: u32) -> ServerSnapshot<WorldReceiveError> {
        let mut callbacks = WorldNetworkCallbacks {
            messages: &self.received_messages,
        };
        self.base.process_command_snapshot(&mut callbacks, now_ms)
    }

    /// Возвращает producer handle доказанных World server-команд.
    pub fn command_handle(&self) -> ServerCommandHandle {
        self.base.command_handle()
    }

    /// Возвращает число сообщений, ожидающих доменного snapshot-прохода.
    pub fn pending_messages(&self) -> i32 {
        self.received_messages.get_size()
    }

    /// Передаёт старейшее World-сообщение фактическому владельцу `CGame`.
    pub fn pop_received_message(&self) -> Option<CMessage> {
        self.received_messages.pop()
    }

    /// Сообщает, остались ли записи в исходном `m_Clients` map.
    pub fn has_clients(&self) -> bool {
        self.base.has_clients()
    }
}

struct WorldNetworkCallbacks<'a> {
    messages: &'a CMsgQueue<CMessage>,
}

impl ServerComponentCallbacks for WorldNetworkCallbacks<'_> {
    type Error = WorldReceiveError;

    fn on_receive(
        &mut self,
        client: &mut CServerClient,
        recv_time_ms: u32,
    ) -> Result<(), Self::Error> {
        CMyNetServerClientWorld::on_receive(client, recv_time_ms, self.messages).map(|_| ())
    }

    fn on_receive_error(
        &mut self,
        _client: &mut CServerClient,
        _error: &Self::Error,
    ) -> ComponentReceiveErrorAction {
        ComponentReceiveErrorAction::None
    }

    fn on_close(&mut self, client: &mut CServerClient) {
        CMyNetServerClientWorld::on_close(client, self.messages);
    }

    fn on_receive_rate_exceeded(
        &mut self,
        _client: &mut CServerClient,
        _actual: i32,
        _permitted: i32,
    ) {
        // World не переопределял пустой общий diagnostic callback.
    }

    fn on_missing_map_id_client(&mut self, _map_id: i32, _socket_id: i32) {
        // World server не emitted отдельный OnMapIDError.
    }

    fn on_missing_map_name_client(&mut self, _map_name: &[u8], _socket_id: i32) {
        // World-вариант не emitted producer строковой identity.
    }
}

impl Default for CMyNetServerWorld {
    fn default() -> Self {
        Self::new(0)
    }
}

// Совместимая семантика `SetSendRevBuf` и назначение поля конструктора
// `+0x120 = 0` остаются неизвестными transport/layout границами; пустые
// Rust-аналоги для них не вводятся.
