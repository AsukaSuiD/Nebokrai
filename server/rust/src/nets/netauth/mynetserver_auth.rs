//! Производный сетевой владелец AuthServer, восстановленный из
//! `nets/netauth/mynetserver_auth.cpp` и доказанных call sites `CGame`.
//!
//! Owner создаёт принятых клиентов, публикует synthetic accept/close сообщения,
//! обрабатывает общий command snapshot и передаёт конкретную FIFO владельцу
//! `CGame::ProcessMessage`. Контракт подтверждён точной парой AuthServer EXE/PDB.
//! Долгоживущие Linux read-задачи и send-operation возвращаются runtime как
//! типизированные `ServerIoAction`; этот owner не создаёт второй transport
//! runtime и не исполняет доменные сообщения.
//! Производный конструктор менял максимум незавершённых send-операций на `100`
//! и per-client send-buffer limit на `0x1000000`. Позднее `CGame` заменял их
//! значениями setup вместе с max LoginServer count, записывал поздний backlog
//! `10` и config timeout первого пакета; Rust оставляет для этого явный config-
//! метод, но не читает `setup.ini` внутри сети.
//!
//! Старый `OnAccept` вызывался до постановки `ADD`, поэтому Rust-фабрика сначала
//! создаёт Auth-state и публикует `0x0CF401`, а затем общий `CServer` ставит
//! owned client-команду. Snapshot message loop берёт исходный размер один раз:
//! сообщения, добавленные обработчиком во время прохода, остаются следующей
//! итерации. Владение Rust заменяет виртуальное удаление после каждого `Run`.

use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;

use tokio::net::TcpStream;

use crate::nets::msgqueue::CMsgQueue;
use crate::nets::netauth::message::CMessage;
use crate::nets::netauth::mynetserverclient_auth::{AuthReceiveError, CMyNetServerClientAuth};
use crate::nets::serverclient::CServerClient;
use crate::nets::servers::{
    AcceptStart, AdmissionOutcome, CServer, ComponentReceiveErrorAction, ServerCommandHandle,
    ServerComponentCallbacks, ServerHostError, ServerSnapshot,
};

const AUTH_MAX_BLOCK_CONNECTIONS_AFTER_HOST: i32 = 10;

const AUTH_DEFAULT_MAX_IN_FLIGHT_SENDS: i32 = 100;
const AUTH_DEFAULT_PERMITTED_SEND_BYTES: i32 = 0x100_0000;

/// Владелец общего `CServer` и его конкретной FIFO `CMessage`.
pub(crate) struct CMyNetServerAuth {
    base: CServer,
    received_messages: CMsgQueue<CMessage>,
}

impl CMyNetServerAuth {
    /// Создаёт Auth server с исходными component defaults.
    pub(crate) fn new(now_ms: u32) -> Self {
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
    pub(crate) fn host(
        &mut self,
        port: u32,
        address: Option<Ipv4Addr>,
        socket_type: i32,
        legacy_flag: bool,
    ) -> Result<(), ServerHostError> {
        self.base.host(port, address, socket_type, legacy_flag)
    }

    /// Применяет Auth setup и две поздние записи исходного `CGame`.
    pub(crate) fn configure_limits(
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
    pub(crate) fn load_allowed_clients(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        self.base.load_allowed_clients(path)
    }

    /// Сохраняет две исходные local-address записи после успешного `Host`.
    pub(crate) fn set_local_identity(&mut self, ip: &[u8], ipv4_word: u32) {
        self.base.set_local_identity(ip, ipv4_word);
    }

    /// Возвращает текущий dotted IPv4 унаследованного socket-state.
    pub(crate) fn local_ip(&self) -> &[u8] {
        self.base.local_ip()
    }

    /// Возвращает текущий IPv4 как исходный x86 `unsigned long`.
    pub(crate) const fn local_ipv4_word(&self) -> u32 {
        self.base.local_ipv4_word()
    }

    /// Выполняет admission через Auth virtual-фабрику и ставит общий `ADD`.
    pub(crate) fn queue_accepted(
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
    pub(crate) fn begin_accept(&self) -> AcceptStart {
        self.base.begin_accept()
    }

    /// Применяет один общий net-thread snapshot с конкретными Auth callbacks.
    pub(crate) fn process_network_snapshot(
        &mut self,
        now_ms: u32,
    ) -> ServerSnapshot<AuthReceiveError> {
        let mut callbacks = AuthNetworkCallbacks {
            messages: &self.received_messages,
        };
        self.base.process_command_snapshot(&mut callbacks, now_ms)
    }

    /// Возвращает producer handle для доказанного Auth `SendToLogin`.
    pub(crate) fn command_handle(&self) -> ServerCommandHandle {
        self.base.command_handle()
    }

    /// Возвращает число сообщений, ожидающих следующего snapshot-прохода.
    pub(crate) fn pending_messages(&self) -> i32 {
        self.received_messages.get_size()
    }

    /// Сообщает, остались ли записи в `m_Clients` для `ExitWorkerThread`.
    pub(crate) fn has_clients(&self) -> bool {
        self.base.has_clients()
    }

    /// Передаёт старейшее Auth-сообщение фактическому владельцу `CGame`.
    pub(crate) fn pop_received_message(&self) -> Option<CMessage> {
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
