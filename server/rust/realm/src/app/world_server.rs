//! Сервер принятых GameServer-соединений WorldServer из
//! `nets/networld/mynetserver.cpp`; Realm — общий обладатель принятого
//! server-направления World. Источник контракта — та же точная
//! пара, что у [`crate::app::world_message`].
//!
//! Машинно подтверждённые точки (первая секция `.exe/Nworldserver.exe`):
//! - ctor `CMyNetServer` `0x428250`: базовый `CServer` ctor `0x427370`,
//!   vtable `0x54107C`, поле `+0x120 = 0`, limits `+0x14C = 100` и
//!   `+0x150 = 0x2000000` — ровно `WORLD_DEFAULT_MAX_IN_FLIGHT_SENDS` и
//!   `WORLD_DEFAULT_PERMITTED_SEND_BYTES`;
//! - `CreateServerClient` `0x428290`: `new` объект `0xC8` байт и ctor
//!   `CMyServerClient` `0x42BBA0` с владельцем `this`.
//!
//! Owner задаёт World limits, создаёт принятое client-состояние и хранит
//! единственную FIFO обычных сообщений и reconnect handoff. Close публикует
//! `0x3FC02` с map ID; parser error не добавляет ban или `QUIT` (см.
//! `OnReceive` `0x42BD00` в [`crate::app::world_server_client`]). Общий
//! `CServer` владеет I/O/runtime, а этот слой сохраняет только World
//! component state и callbacks. Типизированное `LoginClientReconnected`
//! заменяет исходную публикацию `0x3FC03 + CMyNetClient*`: тот же тип
//! обрабатывает сам диспетчер `OnServerMessage` (`cmp eax,0x3FC03` по
//! адресу `0x4ADD4A`; ранняя запись ошибочно относила сравнение к
//! `OnTeamMessage` — в его теле ссылок на тип нет), а wire-передача
//! указателя не получает Rust-значения.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::net::TcpStream;

use nebokrai_shared::network::{
    AcceptStart, AdmissionOutcome, CMsgQueue, CServer, CServerClient, ComponentReceiveErrorAction,
    ServerCommandHandle, ServerComponentCallbacks, ServerHostError, ServerSnapshot,
};

use super::world_client::CMyNetClient;
use super::world_message::CMessage;
use super::world_server_client::{CMyServerClient, GameServerReceiveError, WorldMessageSink};

const WORLD_DEFAULT_MAX_IN_FLIGHT_SENDS: i32 = 100;
const WORLD_DEFAULT_PERMITTED_SEND_BYTES: i32 = 0x200_0000;

/// Одно событие исходной FIFO World `CMyNetServer`.
pub enum WorldServerEvent {
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
pub struct WorldServerEventSender {
    events: Arc<CMsgQueue<WorldServerEvent>>,
}

impl WorldServerEventSender {
    /// Передаёт FIFO владение новым подключённым LoginServer client.
    pub fn publish_reconnected_login_client(&self, client: CMyNetClient) {
        self.events
            .push(WorldServerEvent::LoginClientReconnected(client));
    }
}

/// Общий `CServer` и FIFO событий принятых GameServer.
/// Клон для сетевой задачи разделяет состояние; доменный цикл снимает FIFO.
/// Короткий lock защищает транспортное состояние и не удерживается через await.
#[derive(Clone)]
pub struct CMyNetServer {
    base: Arc<Mutex<CServer>>,
    event_sender: WorldServerEventSender,
}

impl CMyNetServer {
    /// Создаёт World server с исходными component defaults.
    pub fn new(now_ms: u32) -> Self {
        let mut base = CServer::new(now_ms);
        base.configure_send_limits(
            WORLD_DEFAULT_MAX_IN_FLIGHT_SENDS,
            WORLD_DEFAULT_PERMITTED_SEND_BYTES,
        );
        Self {
            base: Arc::new(Mutex::new(base)),
            event_sender: WorldServerEventSender {
                events: Arc::new(CMsgQueue::new()),
            },
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
        self.base
            .lock()
            .host(port, address, socket_type, legacy_flag)
    }

    /// Сохраняет две исходные local-address записи после успешного `Host`.
    pub fn set_local_identity(&mut self, ip: &[u8], ipv4_word: u32) {
        self.base.lock().set_local_identity(ip, ipv4_word);
    }

    /// Возвращает текущий dotted IPv4 унаследованного socket-state.
    pub fn local_ip(&self) -> Vec<u8> {
        self.base.lock().local_ip().to_vec()
    }

    /// Возвращает текущий IPv4 как исходный x86 `unsigned long`.
    pub fn local_ipv4_word(&self) -> u32 {
        self.base.lock().local_ipv4_word()
    }

    /// Применяет восемь setup-записей в точном порядке World `InitNetServer`.
    #[allow(
        clippy::too_many_arguments,
        reason = "сигнатура сохраняет восемь последовательных записей исходного owner-а"
    )]
    pub fn configure_after_host(
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
        self.base.lock().configure_transport_after_host(
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
    pub fn begin_accept(&self) -> AcceptStart {
        self.base.lock().begin_accept()
    }

    /// Выполняет admission через World virtual-фабрику и ставит общий `ADD`.
    pub fn queue_accepted(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
    ) -> AdmissionOutcome {
        self.base
            .lock()
            .queue_accepted_with(stream, peer, now_ms, CMyServerClient::new_state)
    }

    /// Применяет один net-thread snapshot с конкретными World callbacks.
    pub fn process_network_snapshot(
        &mut self,
        now_ms: u32,
    ) -> ServerSnapshot<GameServerReceiveError> {
        let mut callbacks = WorldNetworkCallbacks {
            events: self.event_sender.events.as_ref(),
        };
        self.base
            .lock()
            .process_command_snapshot(&mut callbacks, now_ms)
    }

    /// Возвращает producer handle доказанных World server-команд.
    pub fn command_handle(&self) -> ServerCommandHandle {
        self.base.lock().command_handle()
    }

    /// Возвращает число событий, ожидающих доменного snapshot-прохода.
    pub fn pending_events(&self) -> i32 {
        self.event_sender.events.get_size()
    }

    /// Передаёт старейшее World-событие фактическому владельцу `CGame`.
    pub fn pop_received_event(&self) -> Option<WorldServerEvent> {
        self.event_sender.events.pop()
    }

    /// Возвращает producer для фонового owner-а повторного подключения.
    pub fn event_sender(&self) -> WorldServerEventSender {
        self.event_sender.clone()
    }

    /// Передаёт FIFO владение новым подключённым LoginServer client.
    pub fn publish_reconnected_login_client(&self, client: CMyNetClient) {
        self.event_sender.publish_reconnected_login_client(client);
    }

    /// Ставит локально созданное сообщение в ту же FIFO, что и network receive.
    pub fn publish_local_message(&self, message: CMessage) {
        self.event_sender
            .events
            .push(WorldServerEvent::Message(message));
    }

    /// Сообщает, остались ли записи в исходном `m_Clients` map.
    pub fn has_clients(&self) -> bool {
        self.base.lock().has_clients()
    }

    /// Возвращает exact signed `CServer::m_lClientNum` для World info-owner-а.
    pub fn client_count(&self) -> i32 {
        self.base.lock().client_count()
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
