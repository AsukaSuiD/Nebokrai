//! Сервер принятых GameServer-соединений WorldServer из `mynetserver.cpp`.
//! Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Owner задаёт World limits, создаёт `CMyServerClient` и хранит единственную
//! FIFO обычных сообщений и reconnect handoff. Close публикует `0x3FC02` с
//! map ID; parser error не добавляет ban или `QUIT`. Общий `CServer` владеет
//! I/O/runtime, а этот слой сохраняет только World component state и callbacks.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use tokio::net::TcpStream;

use crate::nets::msgqueue::CMsgQueue;
use crate::nets::serverclient::CServerClient;
use crate::nets::servers::{
    AcceptStart, AdmissionOutcome, CServer, ComponentReceiveErrorAction, ServerCommandHandle,
    ServerComponentCallbacks, ServerHostError, ServerSnapshot,
};

use super::message::CMessage;
use super::mynetclient::CMyNetClient;
use super::myserverclient::{CMyServerClient, GameServerReceiveError, WorldMessageSink};

const WORLD_DEFAULT_MAX_IN_FLIGHT_SENDS: i32 = 100;
const WORLD_DEFAULT_PERMITTED_SEND_BYTES: i32 = 0x200_0000;

/// Одно событие исходной FIFO World `CMyNetServer`.
pub(crate) enum WorldServerEvent {
    /// Обычное wire-сообщение либо synthetic disconnect `0x3FC02`.
    Message(CMessage),
    /// Типизированная замена внутрипроцессного `0x3FC03 + CMyNetClient*`.
    LoginClientReconnected(CMyNetClient),
}

/// Клонируемый отправитель единственной World FIFO.
///
/// Сетевые обратные вызовы и reconnect-worker LoginServer публикуют события
/// через один mutex-защищённый хвост. Отправитель не открывает доступ к
/// `CServer` и не позволяет обойти доменный snapshot `CGame::process_message`.
#[derive(Clone)]
pub(crate) struct WorldServerEventSender {
    events: Arc<CMsgQueue<WorldServerEvent>>,
}

impl WorldServerEventSender {
    /// Передаёт FIFO владение новым подключённым LoginServer client.
    pub(crate) fn publish_reconnected_login_client(&self, client: CMyNetClient) {
        self.events
            .push(WorldServerEvent::LoginClientReconnected(client));
    }
}

/// Владелец общего `CServer` и FIFO событий принятых GameServer.
pub(crate) struct CMyNetServer {
    base: CServer,
    event_sender: WorldServerEventSender,
}

impl CMyNetServer {
    /// Создаёт World server с исходными component defaults.
    pub(crate) fn new(now_ms: u32) -> Self {
        let mut base = CServer::new(now_ms);
        base.configure_send_limits(
            WORLD_DEFAULT_MAX_IN_FLIGHT_SENDS,
            WORLD_DEFAULT_PERMITTED_SEND_BYTES,
        );
        Self {
            base,
            event_sender: WorldServerEventSender {
                events: Arc::new(CMsgQueue::new()),
            },
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

    /// Применяет восемь setup-записей в точном порядке World `InitNetServer`.
    #[allow(
        clippy::too_many_arguments,
        reason = "сигнатура сохраняет восемь последовательных записей исходного owner-а"
    )]
    pub(crate) fn configure_after_host(
        &mut self,
        check_receive_rate: bool,
        max_in_flight_sends: i32,
        maximum_bytes_per_second: u32,
        maximum_clients: i32,
        check_message_content: bool,
        forbid_time_ms: u32,
        maximum_message_length: u32,
        permitted_send_bytes: i32,
    ) {
        self.base.configure_transport_after_host(
            check_receive_rate,
            max_in_flight_sends,
            maximum_bytes_per_second,
            maximum_clients,
            check_message_content,
            forbid_time_ms,
            maximum_message_length,
            permitted_send_bytes,
        );
    }

    /// Начинает одну общую accept-operation либо сообщает причину ожидания.
    pub(crate) fn begin_accept(&self) -> AcceptStart {
        self.base.begin_accept()
    }

    /// Выполняет admission через World virtual-фабрику и ставит общий `ADD`.
    pub(crate) fn queue_accepted(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
    ) -> AdmissionOutcome {
        self.base
            .queue_accepted_with(stream, peer, now_ms, CMyServerClient::new_state)
    }

    /// Применяет один net-thread snapshot с конкретными World callbacks.
    pub(crate) fn process_network_snapshot(
        &mut self,
        now_ms: u32,
    ) -> ServerSnapshot<GameServerReceiveError> {
        let mut callbacks = WorldNetworkCallbacks {
            events: self.event_sender.events.as_ref(),
        };
        self.base.process_command_snapshot(&mut callbacks, now_ms)
    }

    /// Возвращает producer handle доказанных World server-команд.
    pub(crate) fn command_handle(&self) -> ServerCommandHandle {
        self.base.command_handle()
    }

    /// Возвращает число событий, ожидающих доменного snapshot-прохода.
    pub(crate) fn pending_events(&self) -> i32 {
        self.event_sender.events.get_size()
    }

    /// Передаёт старейшее World-событие фактическому владельцу `CGame`.
    pub(crate) fn pop_received_event(&self) -> Option<WorldServerEvent> {
        self.event_sender.events.pop()
    }

    /// Возвращает producer для фонового owner-а повторного подключения.
    pub(crate) fn event_sender(&self) -> WorldServerEventSender {
        self.event_sender.clone()
    }

    /// Передаёт FIFO владение новым подключённым LoginServer client.
    pub(crate) fn publish_reconnected_login_client(&self, client: CMyNetClient) {
        self.event_sender.publish_reconnected_login_client(client);
    }

    /// Ставит локально созданное сообщение в ту же FIFO, что и network receive.
    pub(crate) fn publish_local_message(&self, message: CMessage) {
        self.event_sender.events.push(WorldServerEvent::Message(message));
    }

    /// Сообщает, остались ли записи в исходном `m_Clients` map.
    pub(crate) fn has_clients(&self) -> bool {
        self.base.has_clients()
    }

    /// Возвращает exact signed `CServer::m_lClientNum` для World info-owner-а.
    pub(crate) const fn client_count(&self) -> i32 {
        self.base.client_count()
    }
}

struct WorldNetworkCallbacks<'a> {
    events: &'a CMsgQueue<WorldServerEvent>,
}

impl WorldMessageSink for WorldNetworkCallbacks<'_> {
    fn publish_message(&self, message: CMessage) {
        self.events.push(WorldServerEvent::Message(message));
    }
}

impl ServerComponentCallbacks for WorldNetworkCallbacks<'_> {
    type Error = GameServerReceiveError;

    fn on_receive(
        &mut self,
        client: &mut CServerClient,
        recv_time_ms: u32,
    ) -> Result<(), Self::Error> {
        CMyServerClient::on_receive(client, recv_time_ms, self).map(|_| ())
    }

    fn on_receive_error(
        &mut self,
        _client: &mut CServerClient,
        _error: &Self::Error,
    ) -> ComponentReceiveErrorAction {
        ComponentReceiveErrorAction::None
    }

    fn on_close(&mut self, client: &mut CServerClient) {
        CMyServerClient::on_close(client, self);
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
        // World networld-вариант не emitted отдельный OnMapIDError.
    }

    fn on_missing_map_name_client(&mut self, _map_name: &[u8], _socket_id: i32) {
        // World networld-вариант не emitted producer строковой identity.
    }
}

impl Default for CMyNetServer {
    fn default() -> Self {
        Self::new(0)
    }
}

// Поле World-конструктора `+0x120 = 0` не связано с наблюдаемым состоянием и
// не получает пустого Rust-поля только ради совпадения layout.
