//! Владелец runtime из `miscserver/game.cpp`, подтверждённый `miscserver.exe` и
//! `miscserver.pdb`. Он управляет World-соединением, auction room, FIFO сообщений,
//! статистикой и жизненным циклом процесса.
//! В составе Realm `app/` — runtime-владелец роли MiscServer.
//!
//! Первичное соединение и reconnect удаляют прежний client и используют общий
//! десятисекундный IPv4 connect. Оба отправляют регистрацию; первый добавляет
//! sync и берёт порт из setup, второй использует `0x092F`. Ошибка reconnect
//! оставляет client отсутствующим. Системный resolver заменяет старый поиск
//! адреса, сохраняя выбор первого IPv4.
//!
//! FIFO снимается целиком. Opcode записывается до обработчика и сбрасывается
//! после него, а reconnect завершается до следующего элемента. Семейство
//! `0x14EC00` остаётся no-op с результатом `1`, прочие сообщения — с `0`.
//!
//! `Init` очищает `debug.txt`, читает setup и повторяет connect каждые 8000 ms;
//! его исторический результат всегда `0`. Shutdown может прервать ожидание.
//! `ReFlushLog` проверяет память, сохраняет первые десять счётчиков и очищает
//! карту. `/proc/self/status` заменяет Windows API; только `VmRSS` может сбросить
//! sync-флаг. Runtime сохраняет порядок `AI -> ProcessMessage -> reconnect ->
//! confirm`, паузу 50 ms и shutdown-tail `2005 ms -> notice -> auction Clear`.

use std::collections::BTreeMap;
use std::fs::File;
use std::future::{Future, poll_fn};
use std::io;
use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::Poll;
use std::time::Duration;

use procfs::ProcError;
use procfs::process::Process;
use rustix::time::{ClockId, clock_gettime};

use super::miscservermessage::{WorldAuctionOutcome, on_msg_w2m_auction};
use super::onbillserver::{MiscFunctionOutcome, on_msg_m2m_function};
use super::othermessage::{OtherMessageOutcome, on_other_msg};
use crate::app::setup::{CSetup, SetupLoadReport, SetupOpenError};
use nebokrai_shared::network::{ClientConnectError, ClientSendQueue};
use crate::app::misc_message::{CMessage, MessageHandlers, MessageSender, SendMessageError};
use crate::app::misc_client::{CMyNetClient, MiscClientIoError, MiscClientIoStep};
use crate::auction::aucitionroom::{AddAuctionItemMissingGoodsType, CAuctionRoom};
use crate::auction::auctionnode::CGoodsNode;
use nebokrai_shared::network::bind_tcp_ipv4;

const INITIAL_REGISTRATION_TYPE: i32 = 0x0005_FA01;
const INITIAL_SYNC_TYPE: i32 = 0x0015_EB05;
const RECONNECT_LISTEN_PORT: u16 = 0x092F;
const KIB_PER_MIB: u64 = 1_024;
const MEMORY_SYNC_RESET_THRESHOLD_KIB: u64 = 0x0280_0000 / 1_024;
const LOG_REFRESH_INTERVAL_MS: u32 = 10_000;
const GAME_THREAD_DELAY: Duration = Duration::from_millis(50);
const GAME_THREAD_SHUTDOWN_DELAY: Duration = Duration::from_millis(2_005);
const RECONNECT_CONFIRMATION_TYPE: i32 = 0x0010_EF00;

#[derive(Debug)]
pub enum MiscClientConnectFailure {
    MissingSetupField(&'static str),
    WorldAddressResolution { host: Vec<u8> },
    Bind(io::Error),
    Connect(ClientConnectError),
}

#[derive(Debug)]
pub struct MiscRegistrationReport {
    pub registration: Result<i32, SendMessageError>,
    pub initial_sync: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug)]
pub enum MiscClientConnectOutcome {
    Connected {
        endpoint: SocketAddrV4,
        sends: MiscRegistrationReport,
    },
    Failed(MiscClientConnectFailure),
}

#[derive(Debug)]
pub enum MiscInitializationEnd {
    DebugFileUnavailable { path: PathBuf, source: io::Error },
    Connected,
    Cancelled,
}

#[derive(Debug)]
pub struct MiscInitializationReport {
    pub legacy_result: i32,
    pub setup: Option<Result<SetupLoadReport, SetupOpenError>>,
    pub attempt_count: u64,
    pub last_attempt: Option<MiscClientConnectOutcome>,
    pub end: MiscInitializationEnd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MiscProcessMemorySnapshot {
    pub working_set_kib: Option<u64>,
    pub pagefile_usage_kib: Option<u64>,
}

#[derive(Debug)]
pub enum MiscProcessMemoryQuery {
    Read(MiscProcessMemorySnapshot),
    Unavailable(ProcError),
}
/// Результат проверки лимита памяти и связанного сброса синхронизации.
///
/// Основная строка имела точный формат
/// `WorkingSetSize = %u(M)  PagefileUsage = %u(M) \n`; при reset следовала
/// CP936-строка `内存大于40M,暂停同步消息`.
#[derive(Debug)]
pub struct MiscMemoryConditionReport {
    pub query: MiscProcessMemoryQuery,
    pub working_set_mib: u32,
    pub pagefile_usage_mib: u32,
    pub auction_sync_reset: bool,
}

/// Первая строка старого `ReFlushLog` до memory-проверки с точным форматом
/// `AuctionGoodsCount = %ld  SyscSign = %d AddMsg %u DisMsg %u \r\n`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MiscAuctionStatusSnapshot {
    pub auction_goods_count: i32,
    pub sync_sign: i32,
    pub added_messages: u32,
    pub discarded_messages: u32,
}

/// Одна из не более десяти строк ordered message-map с точным форматом
/// ` MsgType = %u   MsgCount = %u \n`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MiscMessageCountSnapshot {
    pub message_type: u32,
    pub message_count: u32,
}

#[derive(Debug)]
pub struct MiscLogRefreshReport {
    pub status: MiscAuctionStatusSnapshot,
    pub memory: MiscMemoryConditionReport,
    pub message_counts: Vec<MiscMessageCountSnapshot>,
}

impl MiscClientConnectOutcome {
    pub const fn legacy_result(&self) -> i32 {
        match self {
            Self::Connected { .. } => 1,
            Self::Failed(_) => 0,
        }
    }
}

#[derive(Debug)]
pub enum MiscComponentHandlerOutcome {
    GuidFamilyNoOp,
    WorldAuction(WorldAuctionOutcome),
    MiscFunction(MiscFunctionOutcome),
    Other(OtherMessageOutcome),
}

#[derive(Debug)]
pub struct MiscComponentMessageOutcome {
    pub message_type: i32,
    pub legacy_run_result: i32,
    pub handler: MiscComponentHandlerOutcome,
}

#[derive(Debug)]
pub struct MiscProcessMessageOutcome {
    pub legacy_result: i32,
    pub messages: Vec<MiscComponentMessageOutcome>,
}

#[derive(Debug)]
pub struct MiscReconnectTurn {
    pub reconnect_notice: bool,
    pub connection: MiscClientConnectOutcome,
    pub confirmation: Result<i32, SendMessageError>,
}

pub struct MiscGameThreadTurn {
    pub network: Option<Result<MiscClientIoStep, MiscClientIoError>>,
    pub refresh: Option<MiscLogRefreshReport>,
    pub auction: crate::auction::aucitionroom::AuctionAiOutcome,
    pub messages: MiscProcessMessageOutcome,
    pub reconnect: Option<MiscReconnectTurn>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MiscGameThreadRelease {
    pub exit_notice: bool,
    pub legacy_exit_status: i32,
}

pub struct MiscGameThreadReport {
    pub initialization: MiscInitializationReport,
    pub completed_turns: u64,
    pub release: MiscGameThreadRelease,
}

pub struct CGame {
    setup: CSetup,
    auction_room: CAuctionRoom<CGoodsNode>,
    net_client: Option<CMyNetClient>,
    client_close: bool,
    current_message: u32,
    message_record: BTreeMap<i32, i32>,
    add_new_count: u32,
    deleted_new_count: u32,
    done_sync_message: bool,
    sync_start_time: u32,
    done_sync_count: u32,
    last_log_refresh_time: u32,
}

impl CGame {
    pub fn with_setup(setup: CSetup) -> Self {
        Self {
            setup,
            auction_room: CAuctionRoom::new(),
            net_client: None,
            client_close: false,
            current_message: 0,
            message_record: BTreeMap::new(),
            add_new_count: 0,
            deleted_new_count: 0,
            done_sync_message: false,
            sync_start_time: legacy_tick_ms(),
            done_sync_count: 0,
            last_log_refresh_time: 0,
        }
    }

    pub fn new() -> Self {
        Self::with_setup(CSetup::new())
    }

    pub const fn setup(&self) -> &CSetup {
        &self.setup
    }

    pub fn setup_mut(&mut self) -> &mut CSetup {
        &mut self.setup
    }

    /// Выполняет исходный file/setup/connect init до успеха либо shutdown.
    ///
    /// `shutdown` остаётся пригодным caller-у после успешного init.
    pub async fn initialize<Shutdown>(
        &mut self,
        runtime_directory: &Path,
        mut shutdown: Pin<&mut Shutdown>,
        mut observe_attempt: impl FnMut(&MiscClientConnectOutcome),
    ) -> MiscInitializationReport
    where
        Shutdown: Future<Output = ()> + ?Sized,
    {
        let debug_path = runtime_directory.join("debug.txt");
        let debug_file = match File::create(&debug_path) {
            Ok(file) => file,
            Err(source) => {
                return MiscInitializationReport {
                    legacy_result: 0,
                    setup: None,
                    attempt_count: 0,
                    last_attempt: None,
                    end: MiscInitializationEnd::DebugFileUnavailable {
                        path: debug_path,
                        source,
                    },
                };
            }
        };
        drop(debug_file);

        self.client_close = true;
        let setup = self.setup.load_setup(runtime_directory.join("setup.ini"));
        let mut attempt_count = 0_u64;
        let mut last_attempt = None;

        loop {
            let attempt = tokio::select! {
                biased;
                () = shutdown.as_mut() => {
                    return MiscInitializationReport {
                        legacy_result: 0,
                        setup: Some(setup),
                        attempt_count,
                        last_attempt,
                        end: MiscInitializationEnd::Cancelled,
                    };
                }
                attempt = self.init_net_client() => attempt,
            };
            let connected = attempt.legacy_result() != 0;
            attempt_count = attempt_count.saturating_add(1);
            observe_attempt(&attempt);
            last_attempt = Some(attempt);
            if connected {
                return MiscInitializationReport {
                    legacy_result: 0,
                    setup: Some(setup),
                    attempt_count,
                    last_attempt,
                    end: MiscInitializationEnd::Connected,
                };
            }

            tokio::select! {
                biased;
                () = shutdown.as_mut() => {
                    return MiscInitializationReport {
                        legacy_result: 0,
                        setup: Some(setup),
                        attempt_count,
                        last_attempt,
                        end: MiscInitializationEnd::Cancelled,
                    };
                }
                () = tokio::time::sleep(Duration::from_millis(8_000)) => {}
            }
        }
    }

    pub const fn client_close(&self) -> bool {
        self.client_close
    }

    pub fn auction_room_mut(&mut self) -> &mut CAuctionRoom<CGoodsNode> {
        &mut self.auction_room
    }

    pub const fn auction_room(&self) -> &CAuctionRoom<CGoodsNode> {
        &self.auction_room
    }

    pub fn count_new_auction_item(&mut self) {
        self.add_new_count = self.add_new_count.wrapping_add(1);
    }

    pub fn add_auction_item(
        &mut self,
        item: Box<CGoodsNode>,
    ) -> Result<bool, AddAuctionItemMissingGoodsType> {
        self.auction_room
            .add_item_to_auction_room(item, &mut self.deleted_new_count)
    }

    pub const fn add_new_count(&self) -> u32 {
        self.add_new_count
    }

    pub const fn deleted_new_count(&self) -> u32 {
        self.deleted_new_count
    }

    pub fn begin_auction_sync_turn(&mut self) {
        self.done_sync_count = 0;
    }

    pub const fn auction_sync_count(&self) -> u32 {
        self.done_sync_count
    }

    pub const fn auction_sync_enabled(&self) -> bool {
        self.done_sync_message
    }

    pub const fn auction_sync_start_time(&self) -> u32 {
        self.sync_start_time
    }

    pub fn enable_auction_sync(&mut self) {
        self.done_sync_message = true;
    }

    pub fn finish_auction_sync_turn(&mut self) {
        self.done_sync_count = 1;
    }

    pub fn mark_client_closed(&mut self) {
        self.client_close = true;
    }

    pub const fn net_client(&self) -> Option<&CMyNetClient> {
        self.net_client.as_ref()
    }

    pub async fn init_net_client(&mut self) -> MiscClientConnectOutcome {
        self.connect_world(None, true).await
    }

    pub async fn reconnect(&mut self) -> MiscClientConnectOutcome {
        self.connect_world(Some(RECONNECT_LISTEN_PORT), false).await
    }

    pub async fn run_client_io_once(
        &mut self,
    ) -> Result<MiscClientIoStep, MiscClientIoError> {
        let client = self
            .net_client
            .as_mut()
            .ok_or(MiscClientIoError::NotConnected)?;
        client.run_io_once(legacy_tick_ms).await
    }

    pub async fn poll_client_io_once(
        &mut self,
    ) -> Option<Result<MiscClientIoStep, MiscClientIoError>> {
        match self.net_client.as_mut() {
            Some(client) if client.is_connected() => {
                poll_once(client.run_io_once(legacy_tick_ms)).await
            }
            Some(_) | None => None,
        }
    }

    pub fn client_send_queue(&self) -> Option<&ClientSendQueue> {
        self.net_client.as_ref().map(CMyNetClient::send_queue)
    }

    /// Снимает и обрабатывает один атомарный snapshot `CGame::ProcessMessage`.
    ///
    /// Сообщения, опубликованные callback-ами после снятия FIFO, остаются
    /// следующему проходу. Reconnect полностью завершается до следующего
    /// элемента; отчёт сохраняет исходный безусловный return `0`.
    pub async fn process_message(&mut self) -> MiscProcessMessageOutcome {
        let messages = self
            .net_client
            .as_ref()
            .map(CMyNetClient::take_all_messages)
            .unwrap_or_default();
        let mut outcomes = Vec::with_capacity(messages.len());

        for mut message in messages {
            let message_type = message.message_type();
            self.current_message = message_type as u32;
            self.message_record
                .entry(message_type)
                .and_modify(|count| *count = count.wrapping_add(1))
                .or_insert(1);

            let mut selector = MiscOwnerSelector::default();
            let legacy_run_result = message.run(&mut selector);
            let handler = match selector.owner {
                None => MiscComponentHandlerOutcome::GuidFamilyNoOp,
                Some(MiscMessageOwner::WorldAuction) => MiscComponentHandlerOutcome::WorldAuction(
                    on_msg_w2m_auction(&mut message, self),
                ),
                Some(MiscMessageOwner::MiscFunction) => MiscComponentHandlerOutcome::MiscFunction(
                    on_msg_m2m_function(&message, self).await,
                ),
                Some(MiscMessageOwner::Other) => {
                    let sender = self
                        .net_client
                        .as_ref()
                        .map(|client| client as &dyn MessageSender);
                    MiscComponentHandlerOutcome::Other(on_other_msg(&message, sender))
                }
            };
            self.current_message = 0;
            outcomes.push(MiscComponentMessageOutcome {
                message_type,
                legacy_run_result,
                handler,
            });
        }
        MiscProcessMessageOutcome {
            legacy_result: 0,
            messages: outcomes,
        }
    }

    pub const fn current_message(&self) -> u32 {
        self.current_message
    }

    pub const fn message_record(&self) -> &BTreeMap<i32, i32> {
        &self.message_record
    }

    /// Снимает process memory и при строгом превышении 40 MiB сбрасывает sync.
    ///
    /// Неиспользуемый `legacy_argument` сохраняет форму исходного `ulong`.
    pub fn put_mem_condition(&mut self, _legacy_argument: u32) -> MiscMemoryConditionReport {
        let query = match Process::myself().and_then(|process| process.status()) {
            Ok(status) => MiscProcessMemoryQuery::Read(MiscProcessMemorySnapshot {
                working_set_kib: status.vmrss,
                pagefile_usage_kib: status.vmsize,
            }),
            Err(error) => MiscProcessMemoryQuery::Unavailable(error),
        };
        let (working_set_kib, pagefile_usage_kib) = match &query {
            MiscProcessMemoryQuery::Read(snapshot) => (
                snapshot.working_set_kib.unwrap_or(0),
                snapshot.pagefile_usage_kib.unwrap_or(0),
            ),
            MiscProcessMemoryQuery::Unavailable(_) => (0, 0),
        };
        let auction_sync_reset = MEMORY_SYNC_RESET_THRESHOLD_KIB < working_set_kib;
        if auction_sync_reset {
            self.done_sync_message = false;
            self.sync_start_time = legacy_tick_ms();
        }

        MiscMemoryConditionReport {
            query,
            working_set_mib: (working_set_kib / KIB_PER_MIB) as u32,
            pagefile_usage_mib: (pagefile_usage_kib / KIB_PER_MIB) as u32,
            auction_sync_reset,
        }
    }

    pub fn reflush_log(&mut self) -> MiscLogRefreshReport {
        let status = MiscAuctionStatusSnapshot {
            auction_goods_count: self.auction_room.legacy_auction_goods_count() as i32,
            sync_sign: i32::from(self.done_sync_message),
            added_messages: self.add_new_count,
            discarded_messages: self.deleted_new_count,
        };
        let memory = self.put_mem_condition(0);
        let message_counts = self
            .message_record
            .iter()
            .take(10)
            .map(|(&message_type, &message_count)| MiscMessageCountSnapshot {
                message_type: message_type as u32,
                message_count: message_count as u32,
            })
            .collect();
        self.message_record.clear();

        MiscLogRefreshReport {
            status,
            memory,
            message_counts,
        }
    }

    pub async fn run_game_thread_turn(
        &mut self,
    ) -> MiscGameThreadTurn {
        self.begin_auction_sync_turn();

        let now = legacy_tick_ms();
        let refresh = if self
            .last_log_refresh_time
            .wrapping_add(LOG_REFRESH_INTERVAL_MS)
            < now
        {
            self.last_log_refresh_time = legacy_tick_ms();
            Some(self.reflush_log())
        } else {
            None
        };

        let network = self.poll_client_io_once().await;
        if matches!(
            network.as_ref(),
            Some(Err(MiscClientIoError::Io(_) | MiscClientIoError::Send(_)))
        ) {
            // Общий CClient на постоянной transport-ошибке вызывал OnClose;
            // concrete HandleClose публикует 0x16EA01, которое текущий FIFO
            // snapshot переводит в обычный reconnect lifecycle. Malformed
            // frame сохраняет отдельную подтверждённую политику очистки входа.
            self.net_client
                .as_mut()
                .expect("Misc client существовал при выполнении I/O")
                .handle_transport_close();
        }
        let auction = {
            let sender = self
                .net_client
                .as_ref()
                .map(|client| client as &dyn MessageSender);
            self.auction_room.ai(sender)
        };
        let messages = self.process_message().await;
        let reconnect = if self.client_close {
                let connection = self.reconnect().await;
                let sender = self
                    .net_client
                    .as_ref()
                    .map(|client| client as &dyn MessageSender);
                let confirmation = CMessage::new(RECONNECT_CONFIRMATION_TYPE).send(sender, false);
                Some(MiscReconnectTurn {
                    reconnect_notice: true,
                    connection,
                    confirmation,
                })
        } else {
            None
        };

        MiscGameThreadTurn {
            network,
            refresh,
            auction,
            messages,
            reconnect,
        }
    }

    async fn connect_world(
        &mut self,
        registration_port: Option<u16>,
        send_initial_sync: bool,
    ) -> MiscClientConnectOutcome {
        self.net_client.take();
        let mut client = CMyNetClient::new();

        let socket = match bind_tcp_ipv4(None, 0) {
            Ok(socket) => socket,
            Err(error) => {
                return MiscClientConnectOutcome::Failed(MiscClientConnectFailure::Bind(error));
            }
        };
        let endpoint = match resolve_world_endpoint(self.setup.ip_port()) {
            Ok(endpoint) => endpoint,
            Err(failure) => return MiscClientConnectOutcome::Failed(failure),
        };
        if let Err(error) = client.connect(socket, endpoint).await {
            let _legacy_close = client.close();
            return MiscClientConnectOutcome::Failed(MiscClientConnectFailure::Connect(error));
        }

        client.enable_control_send();
        let registration_port = match registration_port.or(self.setup.ip_port().listen_port()) {
            Some(port) => port,
            None => {
                let _legacy_close = client.close();
                return MiscClientConnectOutcome::Failed(
                    MiscClientConnectFailure::MissingSetupField("wListenPort"),
                );
            }
        };
        let local_ip = match self.setup.ip_port().local_ip() {
            Some(local_ip) => local_ip,
            None => {
                let _legacy_close = client.close();
                return MiscClientConnectOutcome::Failed(
                    MiscClientConnectFailure::MissingSetupField("strLocalIp"),
                );
            }
        };
        let mut registration = CMessage::new(INITIAL_REGISTRATION_TYPE);
        registration.base_mut().add_byte(0);
        registration
            .base_mut()
            .add_long(i32::from(registration_port));
        add_legacy_c_string(registration.base_mut(), local_ip);
        let registration = registration.send(Some(&client), false);
        let initial_sync =
            send_initial_sync.then(|| CMessage::new(INITIAL_SYNC_TYPE).send(Some(&client), false));

        self.client_close = false;
        self.net_client = Some(client);
        MiscClientConnectOutcome::Connected {
            endpoint,
            sends: MiscRegistrationReport {
                registration,
                initial_sync,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum MiscMessageOwner {
    WorldAuction,
    MiscFunction,
    Other,
}

#[derive(Default)]
struct MiscOwnerSelector {
    owner: Option<MiscMessageOwner>,
}

impl MessageHandlers for MiscOwnerSelector {
    fn on_world_auction(&mut self, _message: &mut CMessage) {
        self.owner = Some(MiscMessageOwner::WorldAuction);
    }

    fn on_misc_function(&mut self, _message: &mut CMessage) {
        self.owner = Some(MiscMessageOwner::MiscFunction);
    }

    fn on_other(&mut self, _message: &mut CMessage) {
        self.owner = Some(MiscMessageOwner::Other);
    }
}

fn resolve_world_endpoint(
    setup: &crate::app::setup::IpPortSetup,
) -> Result<SocketAddrV4, MiscClientConnectFailure> {
    let host = setup
        .world_ip()
        .ok_or(MiscClientConnectFailure::MissingSetupField("strWorldIp"))?;
    let port = setup
        .world_port()
        .ok_or(MiscClientConnectFailure::MissingSetupField("wPort"))?;
    let host = legacy_c_string_prefix(host);
    let host_text = std::str::from_utf8(host).map_err(|_| {
        MiscClientConnectFailure::WorldAddressResolution {
            host: host.to_vec(),
        }
    })?;
    (host_text, port)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addresses| {
            addresses.find_map(|address| match address {
                SocketAddr::V4(address) => Some(address),
                SocketAddr::V6(_) => None,
            })
        })
        .ok_or_else(|| MiscClientConnectFailure::WorldAddressResolution {
            host: host.to_vec(),
        })
}

fn add_legacy_c_string(message: &mut nebokrai_shared::network::CBaseMessage, value: &[u8]) {
    message.add(legacy_c_string_prefix(value));
    message.add_byte(0);
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

pub fn legacy_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u64).wrapping_mul(1000);
    let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

async fn poll_once<Output>(future: impl Future<Output = Output>) -> Option<Output> {
    let mut future = Box::pin(future);
    poll_fn(move |context| {
        Poll::Ready(match future.as_mut().poll(context) {
            Poll::Ready(output) => Some(output),
            Poll::Pending => None,
        })
    })
    .await
}
/// Полный аналог исходного `GameThreadFunc` с явным владением ресурсами.
///
/// `shutdown` заменяет только внешний `g_bMainThreadExit`. Локальный владелец
/// заменяет `g_pGame`; после любой достигнутой границы аукционная комната
/// очищается в исходной позиции общего shutdown-tail.
pub async fn game_thread_func<Shutdown>(
    runtime_directory: &Path,
    shutdown: Shutdown,
    observe_attempt: impl FnMut(&MiscClientConnectOutcome),
    mut observe_turn: impl FnMut(&MiscGameThreadTurn),
) -> MiscGameThreadReport
where
    Shutdown: Future<Output = ()>,
{
    let mut game = CGame::new();
    tokio::pin!(shutdown);
    let initialization = game
        .initialize(runtime_directory, shutdown.as_mut(), observe_attempt)
        .await;
    let initialized = matches!(&initialization.end, MiscInitializationEnd::Connected);
    let mut completed_turns = 0_u64;

    if initialized {
        'main_loop: loop {
            let turn = tokio::select! {
                biased;
                () = shutdown.as_mut() => break 'main_loop,
                result = game.run_game_thread_turn() => result,
            };
            observe_turn(&turn);
            completed_turns = completed_turns.wrapping_add(1);
            tokio::select! {
                biased;
                () = shutdown.as_mut() => break 'main_loop,
                () = tokio::time::sleep(GAME_THREAD_DELAY) => {}
            }
        }
    }

    tokio::time::sleep(GAME_THREAD_SHUTDOWN_DELAY).await;
    let release = MiscGameThreadRelease {
        exit_notice: true,
        legacy_exit_status: 0,
    };
    game.auction_room.clear();
    drop(game);
    MiscGameThreadReport {
        initialization,
        completed_turns,
        release,
    }
}
