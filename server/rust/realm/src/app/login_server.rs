//! Производный владелец игровых client-соединений LoginServer из
//! `nets/netlogin/mynetserver_client.cpp`; Realm — listener игровых клиентов
//! Login-направления. Источник контракта — та же точная пара,
//! что у [`crate::app::login_message`]; доменные call sites `CGame` остаются
//! у владельца процесса.
//!
//! Owner создаёт accepted client через virtual-фабрику, применяет условные
//! receive-проверки и общий command snapshot, обрабатывает `OnMapStrError`,
//! структурные limit-события и выдаёт конкретную FIFO; второй transport и
//! доменные сообщения — не его граница. Константы component: максимум
//! незавершённых send — `5`, per-client send-buffer limit — `0x400000`;
//! receive-rate/ban, обе условные CRC, предел кадра, max client count,
//! backlog и таймаут первого пакета `CGame` записывал после `Host` — Rust
//! сохраняет их отдельными config-методами и не читает setup внутри сети.
//!
//! Ошибка client parser с доказанной реакцией вызывает component diagnostic,
//! затем общий `CServer` немедленно добавляет peer в forbid-map и ставит
//! `QUIT` до следующей команды текущего snapshot; четыре доказанных пути
//! извнутри OnReceive детально описаны в `login_server_client.rs`. Остальные
//! локальные неизвестности только возвращаются в `ServerSnapshotError` и не
//! получают придуманной сетевой политики. Превышение суммарного receive-rate
//! остаётся общим guard-путём с тем же ban/QUIT.
//!
//! `OnMapStrError` (`0x46AA10`) публикует то же synthetic сообщение `0x10001`
//! с NUL-terminated строковой identity, что и disconnect клиента. Старые
//! `PutDebugString` limit-callbacks представлены локальным FIFO структурных
//! notices без GUI, HTTP или общей административной плоскости.
//!
//! `field +0x120 = 0` конструктора не связан с именованным живым состоянием и
//! не получает пустого Rust-поля.
//!
//! Доказательства: docs/reconstruction/realm-services.md#login-listeners

use std::collections::VecDeque;
use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::net::TcpStream;

use nebokrai_shared::network::{
    AcceptStart, AdmissionOutcome, CMsgQueue, CServer, CServerClient, ComponentReceiveErrorAction,
    ServerCommandHandle, ServerComponentCallbacks, ServerHostError, ServerSnapshot,
};

use super::login_message::CMessage;
use super::login_server_client::{
    CMyNetServerClientClient, ClientReceiveError, ClientReceiveSettings,
};

const CLIENT_DEFAULT_MAX_IN_FLIGHT_SENDS: i32 = 5;
const CLIENT_DEFAULT_PERMITTED_SEND_BYTES: i32 = 0x40_0000;
const CLIENT_IDENTITY_ERROR: i32 = 0x0001_0001;

/// Структурный эквивалент двух исходных limit diagnostics игрового клиента.
#[derive(Debug, Eq, PartialEq)]
pub enum ClientNetworkNotice {
    /// Один объявленный frame превысил `dwMaxMsgLen`.
    OneMessageSizeExceeded {
        socket_id: i32,
        cdkey: Vec<u8>,
        peer_ipv4: u32,
        declared: i32,
        permitted: i32,
    },
    /// Сумма принятых bytes за окно превысила `dwMaxByteNum`.
    TotalMessageSizeExceeded {
        socket_id: i32,
        cdkey: Vec<u8>,
        peer_ipv4: u32,
        actual: i32,
        permitted: i32,
    },
}

/// Владелец общего `CServer`, client receive-настроек и FIFO `CMessage`.
pub struct CMyNetServerClient {
    base: CServer,
    received_messages: CMsgQueue<CMessage>,
    receive_settings: ClientReceiveSettings,
    notices: VecDeque<ClientNetworkNotice>,
}

impl CMyNetServerClient {
    /// Создаёт client server с исходными component defaults.
    pub fn new(now_ms: u32) -> Self {
        let mut base = CServer::new(now_ms);
        base.configure_send_limits(
            CLIENT_DEFAULT_MAX_IN_FLIGHT_SENDS,
            CLIENT_DEFAULT_PERMITTED_SEND_BYTES,
        );
        Self {
            base,
            received_messages: CMsgQueue::new(),
            receive_settings: ClientReceiveSettings::new(false, false, 0),
            notices: VecDeque::new(),
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

    /// Применяет receive-rate, ban и две client CRC-настройки из setup.
    pub fn configure_receive(
        &mut self,
        check_length_crc: bool,
        check_content_crc: bool,
        maximum_bytes_per_second: u32,
        forbid_time_ms: u32,
        maximum_message_length: u32,
    ) {
        self.base.configure_receive_guard(
            check_length_crc,
            maximum_bytes_per_second as i32,
            forbid_time_ms as i32,
        );
        self.receive_settings =
            ClientReceiveSettings::new(check_length_crc, check_content_crc, maximum_message_length);
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

    /// Начинает одну общую accept-operation либо сообщает причину ожидания.
    pub fn begin_accept(&self) -> AcceptStart {
        self.base.begin_accept()
    }

    /// Выполняет admission через client virtual-фабрику и ставит общий `ADD`.
    pub fn queue_accepted(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
    ) -> AdmissionOutcome {
        self.base
            .queue_accepted_with(stream, peer, now_ms, CMyNetServerClientClient::new_state)
    }

    /// Применяет один net-thread snapshot с конкретными client callbacks.
    pub fn process_network_snapshot(&mut self, now_ms: u32) -> ServerSnapshot<ClientReceiveError> {
        let mut callbacks = ClientNetworkCallbacks {
            messages: &self.received_messages,
            receive_settings: self.receive_settings,
            notices: &mut self.notices,
        };
        self.base.process_command_snapshot(&mut callbacks, now_ms)
    }

    /// Возвращает producer handle доказанных client server-команд.
    pub fn command_handle(&self) -> ServerCommandHandle {
        self.base.command_handle()
    }

    /// Возвращает число сообщений, ожидающих доменного snapshot-прохода.
    pub fn pending_messages(&self) -> i32 {
        self.received_messages.get_size()
    }

    /// Передаёт старейшее client-сообщение фактическому владельцу `CGame`.
    pub fn pop_received_message(&self) -> Option<CMessage> {
        self.received_messages.pop()
    }

    /// Передаёт старейшее структурное limit-событие операторскому владельцу.
    pub fn pop_notice(&mut self) -> Option<ClientNetworkNotice> {
        self.notices.pop_front()
    }

    /// Сообщает, остались ли записи в исходном `m_Clients` map.
    pub fn has_clients(&self) -> bool {
        self.base.has_clients()
    }
}

struct ClientNetworkCallbacks<'a> {
    messages: &'a CMsgQueue<CMessage>,
    receive_settings: ClientReceiveSettings,
    notices: &'a mut VecDeque<ClientNetworkNotice>,
}

impl ServerComponentCallbacks for ClientNetworkCallbacks<'_> {
    type Error = ClientReceiveError;

    fn on_receive(
        &mut self,
        client: &mut CServerClient,
        recv_time_ms: u32,
    ) -> Result<(), Self::Error> {
        CMyNetServerClientClient::on_receive(
            client,
            recv_time_ms,
            self.receive_settings,
            self.messages,
        )
        .map(|_| ())
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
                    socket_id: context.socket_id,
                    cdkey: context.map_name.to_vec(),
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
        CMyNetServerClientClient::on_close(client, self.messages);
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
                socket_id: context.socket_id,
                cdkey: context.map_name.to_vec(),
                peer_ipv4: context.peer_ipv4,
                actual,
                permitted,
            });
    }

    fn on_missing_map_id_client(&mut self, _map_id: i32, _socket_id: i32) {
        // Client-вариант не emitted producer `SetClientMapID`.
    }

    fn on_missing_map_name_client(&mut self, map_name: &[u8], _socket_id: i32) {
        let mut message = CMessage::new(CLIENT_IDENTITY_ERROR);
        message.base_mut().add(map_name);
        message.base_mut().add_byte(0);
        self.messages.push(message);
    }
}

impl Default for CMyNetServerClient {
    fn default() -> Self {
        Self::new(0)
    }
}

// Поле Login-конструктора `+0x120 = 0` не связано с живым именованным
// состоянием и не получает пустого Rust-поля только ради layout.
