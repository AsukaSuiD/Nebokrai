//! Достигнутый network/send owner GameServer `CMyNetServer`.
//!
//! Constructor RVA `0x00018E70` имеет статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходник
//! `nets/netserver/mynetserver.cpp`. Он строит общий `CServer`, затем задаёт
//! GameServer send limits `3` и `0x400000`; нулевой slot `+0x120` остаётся у
//! ещё не достигнутого derived runtime и не получает придуманной трактовки.
//!
//! `CMessage::SendToSocket` вызывает virtual slot `+0x38`, подтверждённый
//! vtable/PDB как `CServer::SendBySocketID`; `SendToPlayer`, region/area/around
//! используют slot `+0x3C`, то есть `SendByMapID`. `SendAll` вызывает общий
//! owner напрямую. `ServerCommandHandle` во всех трёх случаях синхронно
//! копирует payload в owned-команду до возврата, поэтому старый общий RLE
//! scratch-buffer не требует Rust lifetime-lock. Реальный net-owner и порядок
//! команд остаются общими `CServer`. Virtual `CreateServerClient` RVA
//! `0x00018EB0`, `OnMapIDError` RVA `0x00018F10`, client callback-chain и
//! concrete FIFO имеют статус `IMPLEMENTED`; та же FIFO типизированно заменяет
//! внутрипроцессные reconnect pointer-сообщения. Linux transport выполняет
//! общий owner, а доменные сообщения здесь не исполняются. Достигнутый
//! `OnGMMessage 0x7FC0F` читает inherited local-IP bytes через тонкий getter,
//! не перенося доменную сборку ответа в network owner.
//!
//! Oversized `SendAll` до отправки печатал inherited
//! `CMySocket::m_lIndexID +0x34`. Exact `CMySocket` constructor RVA
//! `0x0001AB60` это поле не
//! инициализирует, а достигнутый `CGame::InitNetServer` writer-а не содержит.
//! Поэтому Rust хранит только явно наблюдённое позднее значение как `Option`;
//! сама локальная logging-граница отмечена в message-owner-е, не заменена
//! придуманным нулём.

use std::collections::VecDeque;
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
use super::myserverclient::{
    CMyServerClient, ClientMessageSink, ClientNetworkNotice, ClientReceiveError,
    ClientReceiveSettings,
};

const MAP_ID_ERROR_MESSAGE: i32 = 0x0006_FA01;

/// Один элемент исходной Game `CMyNetServer` FIFO.
pub(crate) enum GameServerEvent {
    Message(CMessage),
    WorldClientReconnected(CMyNetClient),
    BillingClientReconnected(CMyNetClient),
}

#[derive(Clone)]
pub(crate) struct GameServerEventPublisher {
    events: Arc<CMsgQueue<GameServerEvent>>,
}

impl GameServerEventPublisher {
    pub(crate) fn publish_message(&self, message: CMessage) {
        self.events.push(GameServerEvent::Message(message));
    }

    pub(crate) fn publish_reconnected_world_client(&self, client: CMyNetClient) {
        self.events
            .push(GameServerEvent::WorldClientReconnected(client));
    }

    pub(crate) fn publish_reconnected_billing_client(&self, client: CMyNetClient) {
        self.events
            .push(GameServerEvent::BillingClientReconnected(client));
    }
}

pub(crate) struct CMyNetServer {
    base: CServer,
    received_events: Arc<CMsgQueue<GameServerEvent>>,
    receive_settings: ClientReceiveSettings,
    notices: VecDeque<ClientNetworkNotice>,
    legacy_index_id: Option<i32>,
}

impl CMyNetServer {
    pub(crate) fn new(now_ms: u32) -> Self {
        let mut base = CServer::new(now_ms);
        base.configure_send_limits(3, 0x400000);
        Self {
            base,
            received_events: Arc::new(CMsgQueue::new()),
            receive_settings: ClientReceiveSettings::new(false, false, 0),
            notices: VecDeque::new(),
            legacy_index_id: None,
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

    /// Повторяет восемь последовательных setup-записей после `Host`.
    #[allow(
        clippy::too_many_arguments,
        reason = "сигнатура сохраняет одну точную последовательность Game CServer"
    )]
    pub(crate) fn configure_after_host(
        &mut self,
        check_network: bool,
        maximum_in_flight_sends: i32,
        maximum_bytes_per_second: u32,
        maximum_clients: i32,
        check_content_crc: bool,
        forbid_time_ms: u32,
        maximum_message_length: u32,
        permitted_send_bytes: i32,
    ) {
        self.base.configure_transport_after_host(
            check_network,
            maximum_in_flight_sends,
            maximum_bytes_per_second,
            maximum_clients,
            check_content_crc,
            forbid_time_ms,
            maximum_message_length,
            permitted_send_bytes,
        );
        self.receive_settings =
            ClientReceiveSettings::new(check_network, check_content_crc, maximum_message_length);
    }

    /// Сохраняет поздние backlog/first-packet timeout записи после `Host`.
    pub(crate) fn configure_accept_limits_after_host(
        &mut self,
        maximum_block_connections: i32,
        first_receive_timeout_ms: i32,
    ) {
        self.base.configure_accept_limits_after_host(
            maximum_block_connections,
            first_receive_timeout_ms,
        );
    }

    /// Сохраняет dotted IPv4 и его исходное `unsigned long` представление.
    pub(crate) fn set_local_identity(&mut self, ip: &[u8], ipv4_word: u32) {
        self.base.set_local_identity(ip, ipv4_word);
    }

    /// Возвращает inherited `CMySocket::m_strLocalIP`, нужный точному
    /// GameServer suffix в адресном GM-ответе.
    pub(crate) fn local_ip(&self) -> &[u8] {
        self.base.local_ip()
    }

    pub(crate) fn begin_accept(&self) -> AcceptStart {
        self.base.begin_accept()
    }

    /// Выполняет admission через точную virtual-фабрику `CMyServerClient`.
    pub(crate) fn queue_accepted(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
    ) -> AdmissionOutcome {
        self.base
            .queue_accepted_with(stream, peer, now_ms, CMyServerClient::new_state)
    }

    /// Применяет один общий command snapshot с Game client callbacks.
    pub(crate) fn process_network_snapshot(
        &mut self,
        now_ms: u32,
    ) -> ServerSnapshot<ClientReceiveError> {
        let mut callbacks = GameNetworkCallbacks {
            events: &self.received_events,
            receive_settings: self.receive_settings,
            notices: &mut self.notices,
        };
        self.base.process_command_snapshot(&mut callbacks, now_ms)
    }

    pub(crate) fn command_handle(&self) -> ServerCommandHandle {
        self.base.command_handle()
    }

    /// Копирует готовый legacy frame в очередь конкретного transport socket ID.
    pub(crate) fn send_to_socket(&self, socket_id: i32, frame: &[u8]) -> i32 {
        self.base
            .command_handle()
            .send_by_socket_id(socket_id, frame)
    }

    /// Копирует frame игроку по numeric map/player identity.
    pub(crate) fn send_to_player(&self, player_id: i32, frame: &[u8]) -> i32 {
        self.base.command_handle().send_by_map_id(player_id, frame)
    }

    /// Копирует один frame всем текущим server clients.
    pub(crate) fn send_all(&self, frame: &[u8]) -> i32 {
        self.base.command_handle().send_all(frame)
    }

    /// Exact successful cross-server handoff: resolve the client by its old
    /// player map ID and clear that route before the destination reconnects.
    pub(crate) fn clear_player_map_id(&self, player_id: i32) -> i32 {
        let socket_id = self.base.get_socket_id_by_map_id(player_id);
        self.base.command_handle().set_client_map_id(socket_id, 0)
    }

    pub(crate) fn has_player_map_id(&self, player_id: i32) -> bool {
        self.base.get_socket_id_by_map_id(player_id) != 0
    }

    /// Публикует exact synthetic `0x6FA01 + map ID + empty C-string`.
    pub(crate) fn on_map_id_error(&self, map_id: i32) {
        let mut message = CMessage::new(MAP_ID_ERROR_MESSAGE);
        message.base_mut().add_long(map_id);
        message.base_mut().add_byte(0);
        self.received_events.push(GameServerEvent::Message(message));
    }

    /// Публикует локально созданное Game-message в тот же FIFO, что и
    /// network callbacks. Используется legacy intra-process message routes.
    pub(crate) fn publish_local_message(&self, message: CMessage) {
        self.received_events.push(GameServerEvent::Message(message));
    }

    /// Передаёт весь текущий FIFO доменному snapshot-владельцу `CGame`.
    pub(crate) fn take_all_events(&self) -> VecDeque<GameServerEvent> {
        self.received_events.take_all()
    }

    /// Typed-замена старого внутрипроцессного `0x6F902 + CMyNetClient*`.
    pub(crate) fn publish_reconnected_world_client(&self, client: CMyNetClient) {
        self.event_publisher()
            .publish_reconnected_world_client(client);
    }

    /// Typed-замена старого внутрипроцессного `0x6F904 + CMyNetClient*`.
    pub(crate) fn publish_reconnected_billing_client(&self, client: CMyNetClient) {
        self.event_publisher()
            .publish_reconnected_billing_client(client);
    }

    pub(crate) fn event_publisher(&self) -> GameServerEventPublisher {
        GameServerEventPublisher {
            events: Arc::clone(&self.received_events),
        }
    }

    pub(crate) fn pop_notice(&mut self) -> Option<ClientNetworkNotice> {
        self.notices.pop_front()
    }

    /// Материализует только значение, подтверждённое внешним startup writer-ом.
    pub(crate) const fn set_legacy_index_id(&mut self, index_id: i32) {
        self.legacy_index_id = Some(index_id);
    }

    pub(crate) const fn legacy_index_id(&self) -> Option<i32> {
        self.legacy_index_id
    }

    pub(crate) const fn client_count(&self) -> i32 {
        self.base.client_count()
    }
}

struct GameNetworkCallbacks<'a> {
    events: &'a CMsgQueue<GameServerEvent>,
    receive_settings: ClientReceiveSettings,
    notices: &'a mut VecDeque<ClientNetworkNotice>,
}

impl ServerComponentCallbacks for GameNetworkCallbacks<'_> {
    type Error = ClientReceiveError;

    fn on_receive(
        &mut self,
        client: &mut CServerClient,
        _recv_time_ms: u32,
    ) -> Result<(), Self::Error> {
        CMyServerClient::on_receive(client, self.receive_settings, self).map(|_| ())
    }

    fn on_receive_error(
        &mut self,
        client: &mut CServerClient,
        error: &Self::Error,
    ) -> ComponentReceiveErrorAction {
        if let ClientReceiveError::MessageLengthExceeded {
            declared,
            permitted,
        } = error
        {
            let context = client.message_context();
            self.notices
                .push_back(ClientNetworkNotice::OneMessageSizeExceeded {
                    player_id: context.map_id,
                    peer_ipv4: context.peer_ipv4,
                    declared: *declared as i32,
                    permitted: *permitted as i32,
                });
        }

        if error.requires_forbid_and_quit() {
            ComponentReceiveErrorAction::ForbidAndQuit
        } else {
            ComponentReceiveErrorAction::None
        }
    }

    fn on_close(&mut self, client: &mut CServerClient) {
        CMyServerClient::on_close(client, self);
    }

    fn on_receive_rate_exceeded(
        &mut self,
        client: &mut CServerClient,
        actual: i32,
        permitted: i32,
    ) {
        let context = client.message_context();
        self.notices
            .push_back(ClientNetworkNotice::TotalMessageSizeExceeded {
                player_id: context.map_id,
                peer_ipv4: context.peer_ipv4,
                actual,
                permitted,
            });
    }

    fn on_missing_map_id_client(&mut self, map_id: i32, _socket_id: i32) {
        let mut message = CMessage::new(MAP_ID_ERROR_MESSAGE);
        message.base_mut().add_long(map_id);
        message.base_mut().add_byte(0);
        self.events.push(GameServerEvent::Message(message));
    }

    fn on_missing_map_name_client(&mut self, _map_name: &[u8], _socket_id: i32) {
        // Game netserver использует numeric identity и не emitted string variant.
    }
}

impl ClientMessageSink for GameNetworkCallbacks<'_> {
    fn publish_message(&self, message: CMessage) {
        self.events.push(GameServerEvent::Message(message));
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\nets\netserver\mynetserver.cpp

// IMPLEMENTED: constructor материализован выше; base destructor выражен
// обычным `Drop` Rust, покрытые raw-блоки удалены.

// IMPLEMENTED: `CreateServerClient` и `OnMapIDError` материализованы выше.

// CLASSIFIED_TECHNICAL_NOISE: deleting thunk и allocation unwind
// `CMyServerClient` покрываются Rust ownership после materialization owner-а.

// COMPONENT_VARIANT_END: GameServer
