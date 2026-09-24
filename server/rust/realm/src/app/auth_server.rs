//! Производный сетевой владелец AuthServer из `nets/netauth/mynetserver_auth.cpp`,
//! перенесённый в Realm — сервер принятого Auth-направления. Источник контракта —
//! та же точная пара, что у [`crate::app::auth_message`]. Доказанные call sites
//! `CGame` сохранены прежним владельцем процесса.
//!
//! Машинно подтверждённые точки (первая секция `.exe/authserver.exe`):
//! - ctor `CMyNetServer_Auth` `0x412770`: базовый `CServer` ctor `0x4118B0`,
//!   vtable `0x42E178`, поле `+0x120 = 0`, limits `+0x14C = 0x64` (100) и
//!   `+0x150 = 0x1000000` — точные константы component;
//! - `CreateServerClient` `0x4127B0`: `new` объекта `0xC8` байт, ctor
//!   `CMyNetServerClient_Auth` `0x415A10` с владельцем `this`.

use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;

use tokio::net::TcpStream;

use nebokrai_shared::network::{
    AcceptStart, AdmissionOutcome, CMsgQueue, CServer, CServerClient, ComponentReceiveErrorAction,
    ServerCommandHandle, ServerComponentCallbacks, ServerHostError, ServerSnapshot,
};

use super::auth_message::CMessage;
use super::auth_server_client::{AuthReceiveError, CMyNetServerClientAuth};

const AUTH_MAX_BLOCK_CONNECTIONS_AFTER_HOST: i32 = 10;

const AUTH_DEFAULT_MAX_IN_FLIGHT_SENDS: i32 = 100;
const AUTH_DEFAULT_PERMITTED_SEND_BYTES: i32 = 0x100_0000;

/// Владелец общего `CServer` и его конкретной FIFO `CMessage`.
pub struct CMyNetServerAuth {
    base: CServer,
    received_messages: CMsgQueue<CMessage>,
}

impl CMyNetServerAuth {
    /// Создаёт Auth server с исходными component defaults.
    pub fn new(now_ms: u32) -> Self {
        let mut base = CServer::new(now_ms);
        base.configure_send_limits(
            AUTH_DEFAULT_MAX_IN_FLIGHT_SENDS,
            AUTH_DEFAULT_PERMITTED_SEND_BYTES,
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

    /// Применяет Auth setup и две поздние записи исходного `CGame`.
    pub fn configure_limits(
        &mut self,
        max_login_servers: i32,
        max_in_flight_sends: i32,
        permitted_send_bytes: i32,
        new_accept_timeout_ms: i32,
    ) {
        self.base.configure_max_clients(max_login_servers);
        self.base
            .configure_send_limits(max_in_flight_sends, permitted_send_bytes);
        self.base.configure_accept_limits_after_host(
            AUTH_MAX_BLOCK_CONNECTIONS_AFTER_HOST,
            new_accept_timeout_ms,
        );
    }

    /// Загружает общий allow-list до `Host` либо во время Auth reload.
    pub fn load_allowed_clients(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        self.base.load_allowed_clients(path)
    }

    /// Сохраняет две исходные local-address записи после успешного `Host`.
    pub fn set_local_identity(&mut self, ip: &[u8], ipv4_word: u32) {
        self.base.set_local_identity(ip, ipv4_word);
    }

    /// Возвращает текущий dotted IPv4 унаследованного socket-state.
    pub fn local_ip(&self) -> &[u8] {
        self.base.local_ip()
    }

    /// Возвращает текущий IPv4 как исходный x86 `unsigned long`.
    pub const fn local_ipv4_word(&self) -> u32 {
        self.base.local_ipv4_word()
    }

    /// Выполняет admission через Auth virtual-фабрику и ставит общий `ADD`.
    pub fn queue_accepted(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
    ) -> AdmissionOutcome {
        let messages = &self.received_messages;
        self.base
            .queue_accepted_with(stream, peer, now_ms, |socket_id, peer_ipv4, tick| {
                let mut client = CMyNetServerClientAuth::new_state(socket_id, peer_ipv4, tick);
                CMyNetServerClientAuth::on_accept(&mut client, messages);
                client
            })
    }

    /// Начинает одну общую accept-operation либо сообщает причину ожидания.
    pub fn begin_accept(&self) -> AcceptStart {
        self.base.begin_accept()
    }

    /// Применяет один общий net-thread snapshot с конкретными Auth callbacks.
    pub fn process_network_snapshot(&mut self, now_ms: u32) -> ServerSnapshot<AuthReceiveError> {
        let mut callbacks = AuthNetworkCallbacks {
            messages: &self.received_messages,
        };
        self.base.process_command_snapshot(&mut callbacks, now_ms)
    }

    /// Возвращает producer handle для доказанного Auth `SendToLogin`.
    pub fn command_handle(&self) -> ServerCommandHandle {
        self.base.command_handle()
    }

    /// Возвращает число сообщений, ожидающих следующего snapshot-прохода.
    pub fn pending_messages(&self) -> i32 {
        self.received_messages.get_size()
    }

    /// Сообщает, остались ли записи в `m_Clients` для `ExitWorkerThread`.
    pub fn has_clients(&self) -> bool {
        self.base.has_clients()
    }

    /// Передаёт старейшее Auth-сообщение фактическому владельцу `CGame`.
    pub fn pop_received_message(&self) -> Option<CMessage> {
        self.received_messages.pop()
    }
}

struct AuthNetworkCallbacks<'a> {
    messages: &'a CMsgQueue<CMessage>,
}

impl ServerComponentCallbacks for AuthNetworkCallbacks<'_> {
    type Error = AuthReceiveError;

    fn on_receive(
        &mut self,
        client: &mut CServerClient,
        recv_time_ms: u32,
    ) -> Result<(), Self::Error> {
        CMyNetServerClientAuth::on_receive(client, recv_time_ms, self.messages).map(|_| ())
    }

    fn on_receive_error(
        &mut self,
        _client: &mut CServerClient,
        _error: &Self::Error,
    ) -> ComponentReceiveErrorAction {
        ComponentReceiveErrorAction::None
    }

    fn on_close(&mut self, client: &mut CServerClient) {
        CMyNetServerClientAuth::on_close(client, self.messages);
    }

    fn on_receive_rate_exceeded(
        &mut self,
        _client: &mut CServerClient,
        _actual: i32,
        _permitted: i32,
    ) {
        // Auth наследует пустой `CServerClient::OnOneMessageSizeOver`;
        // последующие forbid/QUIT выполняет общий owner.
    }

    fn on_missing_map_id_client(&mut self, _map_id: i32, _socket_id: i32) {
        // Auth-вариант не emitted producer `SetClientMapID`.
    }

    fn on_missing_map_name_client(&mut self, _map_name: &[u8], _socket_id: i32) {
        // Auth-вариант не emitted producer строковой identity.
    }
}

impl Default for CMyNetServerAuth {
    fn default() -> Self {
        Self::new(0)
    }
}

// Поле Auth-конструктора `+0x120 = 0` не связано с живым именованным
// состоянием. Оно не получает пустого поля Rust только ради совпадения layout.

// Accept-delay, net snapshot cadence и JoinHandle долгоживущих read-задач
// принадлежат `authserver/src/cgame.rs`; здесь остаётся только network state.
