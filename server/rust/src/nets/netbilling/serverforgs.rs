//! Производный владелец GameServer-соединений BillingServer из
//! `nets/netbilling/serverforgs.cpp` и доказанных call sites `CGame`.
//!
//! Owner создаёт `CClientForGS`, обрабатывает общий command snapshot и выдаёт
//! конкретную FIFO. Контракт подтверждён точной парой BillingServer EXE/PDB.
//! Долгоживущие Linux I/O actions возвращаются runtime через общий `CServer`;
//! этот owner не исполняет доменные Billing-сообщения и не создаёт второй
//! transport runtime.
//! Производный конструктор менял общий максимум незавершённых send-операций на
//! `100` и per-client send-buffer limit на `0x1000000`. В отличие от Auth и
//! Login World, Billing `CGame` не перезаписывал эти лимиты из setup. После
//! `Host` он загружал общий allow-list `GSInfoSetup.ini` и сохранял local IPv4;
//! Rust оставляет обе операции явными и не читает setup внутри сети.
//!
//! Virtual-фабрика выделяла `CClientForGS`; Rust создаёт тот же component-state
//! непосредственно в admission closure. Старые `operator new`/null и
//! deleting-destructor заменены обычным владением Rust и `Drop`. Receive-ошибка
//! Billing parser не добавляет IP-ban и не ставит `QUIT`; close callback
//! публикует доказанное `0x10EF01` через FIFO. Отсутствующие у производного
//! класса diagnostics для rate guard и identity lookup остаются no-op.
//!
//! Windows `SetSendRevBuf/SO_SNDBUF=0` принятого клиента не переносится в
//! Linux как внешне похожая socket option: требуемая backpressure-семантика не
//! доказана. Неименованное поле конструктора `+0x120 = 0` также не получает
//! фиктивного Rust-state только ради старого layout.

use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;

use tokio::net::TcpStream;

use crate::nets::msgqueue::CMsgQueue;
use crate::nets::netbilling::clientforgs::{BillingReceiveError, CClientForGS};
use crate::nets::netbilling::message::CMessage;
use crate::nets::serverclient::CServerClient;
use crate::nets::servers::{
    AcceptStart, AdmissionOutcome, CServer, ComponentReceiveErrorAction, ServerCommandHandle,
    ServerComponentCallbacks, ServerHostError, ServerSnapshot,
};

const BILLING_DEFAULT_MAX_IN_FLIGHT_SENDS: i32 = 100;
const BILLING_DEFAULT_PERMITTED_SEND_BYTES: i32 = 0x100_0000;

/// Владелец общего `CServer` и Billing FIFO `CMessage`.
pub(crate) struct CServerForGS {
    base: CServer,
    received_messages: CMsgQueue<CMessage>,
}

impl CServerForGS {
    /// Создаёт Billing server с исходными component defaults.
    pub(crate) fn new(now_ms: u32) -> Self {
        let mut base = CServer::new(now_ms);
        base.configure_send_limits(
            BILLING_DEFAULT_MAX_IN_FLIGHT_SENDS,
            BILLING_DEFAULT_PERMITTED_SEND_BYTES,
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

    /// Загружает исходный allow-list GameServer до начала приёма соединений.
    pub(crate) fn load_allowed_clients(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        self.base.load_allowed_clients(path)
    }

    /// Проверяет адрес отключившегося GameServer по загруженному allow-list.
    pub(crate) fn is_allowed_address(&self, peer_ip: &[u8], peer_port: u16) -> bool {
        self.base.is_allowed_address(peer_ip, peer_port)
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

    /// Начинает одну общую accept-operation либо сообщает причину ожидания.
    pub(crate) fn begin_accept(&self) -> AcceptStart {
        self.base.begin_accept()
    }

    /// Выполняет admission через Billing virtual-фабрику и ставит общий `ADD`.
    pub(crate) fn queue_accepted(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
    ) -> AdmissionOutcome {
        self.base
            .queue_accepted_with(stream, peer, now_ms, CClientForGS::new_state)
    }

    /// Применяет один net-thread snapshot с конкретными Billing callbacks.
    pub(crate) fn process_network_snapshot(
        &mut self,
        now_ms: u32,
    ) -> ServerSnapshot<BillingReceiveError> {
        let mut callbacks = BillingNetworkCallbacks {
            messages: &self.received_messages,
        };
        self.base.process_command_snapshot(&mut callbacks, now_ms)
    }

    /// Возвращает producer handle доказанных Billing server-команд.
    pub(crate) fn command_handle(&self) -> ServerCommandHandle {
        self.base.command_handle()
    }

    /// Возвращает число сообщений, ожидающих доменного snapshot-прохода.
    pub(crate) fn pending_messages(&self) -> i32 {
        self.received_messages.get_size()
    }

    /// Передаёт старейшее Billing-сообщение фактическому владельцу `CGame`.
    pub(crate) fn pop_received_message(&self) -> Option<CMessage> {
        self.received_messages.pop()
    }

    /// Сообщает, остались ли записи в исходном `m_Clients` map.
    pub(crate) fn has_clients(&self) -> bool {
        self.base.has_clients()
    }
}

struct BillingNetworkCallbacks<'a> {
    messages: &'a CMsgQueue<CMessage>,
}

impl ServerComponentCallbacks for BillingNetworkCallbacks<'_> {
    type Error = BillingReceiveError;

    fn on_receive(
        &mut self,
        client: &mut CServerClient,
        recv_time_ms: u32,
    ) -> Result<(), Self::Error> {
        CClientForGS::on_receive(client, recv_time_ms, self.messages).map(|_| ())
    }

    fn on_receive_error(
        &mut self,
        _client: &mut CServerClient,
        _error: &Self::Error,
    ) -> ComponentReceiveErrorAction {
        ComponentReceiveErrorAction::None
    }

    fn on_close(&mut self, client: &mut CServerClient) {
        CClientForGS::on_close(client, self.messages);
    }

    fn on_receive_rate_exceeded(
        &mut self,
        _client: &mut CServerClient,
        _actual: i32,
        _permitted: i32,
    ) {
        // Billing не переопределял пустой общий diagnostic callback.
    }

    fn on_missing_map_id_client(&mut self, _map_id: i32, _socket_id: i32) {
        // Billing server не emitted отдельный OnMapIDError.
    }

    fn on_missing_map_name_client(&mut self, _map_name: &[u8], _socket_id: i32) {
        // Billing-вариант не emitted producer строковой identity.
    }
}

impl Default for CServerForGS {
    fn default() -> Self {
        Self::new(0)
    }
}

// `CClientForGS::SetSendRevBuf` задавал Windows `SO_SNDBUF=0`, а значение
// constructor field `+0x120 = 0` не связано с именованным состоянием.
// Совместимые Linux/Rust-аналоги для этих границ неизвестны и не назначены.
