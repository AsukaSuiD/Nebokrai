//! Владелец `CGame` исторического MiscServer из `miscserver/game.cpp`.
//!
//! Статус `CGame::InitNetClient` RVA `0x00001210`, `ReConnect` RVA
//! `0x00001480`, `Init` RVA `0x00001690`, `PutMemCondition` RVA `0x00001720`,
//! `ReFlushLog` RVA `0x00001CD0`, `ProcessMessage` RVA `0x000026A0` и достигнутая
//! constructor-инициализация auction/sync-полей RVA `0x000027B0`, а также
//! внешний `GameThreadFunc` RVA `0x00002870` — `IMPLEMENTED`; catch main-loop
//! RVA `0x00002959` дополнительно имеет статус `VERIFIED_DISASSEMBLY`.
//!
//! Точная пара: `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`;
//! SHA-256 EXE
//! `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65`,
//! SHA-256 PDB
//! `ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7`.
//! Исходный путь PDB:
//! `h:\fengyun\fy_russia\src\server\miscserver\miscserver\game.cpp`.
//!
//! Обе network-функции сначала уничтожают прежний `CMyNetClient`, создают новый
//! IPv4 socket через исходные `Create(0, 0, 1)` и синхронно разрешают World
//! host до общего десятисекундного connect. `CreateSocketThread` заменён
//! awaitable Tokio readiness у самого owned client; static message buffers и
//! WinSock startup не получают пустых вызовов. Неуспех закрывает и уничтожает
//! новый owner, оставляя nullable client, а успех включает `m_bControlSend` и
//! только после постановки регистрационных сообщений сбрасывает
//! `m_bClientClose`.
//!
//! Оба пути отправляют `0x5FA01 + byte(0) + word(port) + local_ip\0`, но port
//! различается буквально: initial использует setup listen-port, reconnect —
//! константу `0x092F`. Только initial затем ставит пустой `0x15EB05`.
//! Результаты исходных `Send` игнорировались; Rust сохраняет их в отчёте, не
//! меняя legacy success connect. Byte-oriented setup host/local IP получают
//! C-string границу по первому NUL. Standard `ToSocketAddrs` заменяет
//! `inet_addr/gethostbyname` и выбирает первый IPv4; high-bit/non-UTF8 hostname
//! остаётся отдельной недоказанной resolution-границей.
//!
//! `ProcessMessage` атомарно забирает весь текущий `CMyNetClient` FIFO. Перед
//! каждым `CMessage::Run` он сохраняет полный opcode в `m_dwCurMsg` и
//! увеличивает signed `std::map<long,long>`; первая запись получает `1`, а
//! последующие используют машинное 32-битное wrapping. После `Run` текущий
//! opcode сбрасывается до удаления сообщения. `BTreeMap<i32, i32>` сохраняет
//! ordered-map семантику без STL node/iterator plumbing, а owned очередь не
//! может содержать старый nullable `CBaseMessage*`. Узкий `MessageHandlers`
//! только запоминает владельца, выбранного точным numeric switch `Run`; затем
//! вызываются готовые `OnMSG_W2M_AUCTION`, `OnMSG_M2M_Fuction` либо
//! `OnOtherMsg`. Результат каждого handler-а и старый return `Run` сохраняются
//! в FIFO-отчёте. Process snapshot стал awaitable только потому, что исходный
//! синхронный `ReConnect` заменён async transport: следующий элемент не может
//! обогнать его завершение. Особое семейство `0x14EC00` остаётся доказанным
//! no-op с return `1`, а default `OnOtherMsg` — return `0`.
//!
//! Частичная Rust-форма `CGame` владеет достигнутыми setup/network полями,
//! статистикой сообщений, двумя auction allocation-счётчиками и уже
//! восстановленным Misc `CAuctionRoom<CGoodsNode>`. Оба счётчика constructor
//! задавал нулями; handler `0x14ED01` увеличивает `m_dwAddNewCount`, а комната
//! увеличивает `m_dwDelNewCount` только при invalid/duplicate GUID. Windows
//! `unsigned long` выражен `u32` с явным wrapping.
//!
//! Для ветви `0x14ED05` точный PDB задаёт `m_bDoneSyscMsg: bool` по offset
//! `0x7D`, `m_dwStartTime: unsigned long` по `0x80` и
//! `m_dwDoneSysnCount: unsigned long` по `0x84`. Constructor ставит только
//! первый флаг в `false` и снимает `timeGetTime` в start-time. Sync-count он не
//! инициализирует: `GameThreadFunc` обнуляет его в начале каждого turn перед
//! `ProcessMessage`. Поэтому Rust хранит его как `Option<u32>` и требует явного
//! `begin_auction_sync_turn`; недостигнутый turn не получает придуманного нуля.
//! `CLOCK_BOOTTIME` уже является совместимой suspend-aware заменой
//! `timeGetTime`, а преобразование в `u32` сохраняет wrapping миллисекунд.
//!
//! `Init` первым создаёт либо очищает относительный `debug.txt` через
//! write-open. Ошибка этой операции сохраняет прежние поля и немедленно
//! завершает функцию. Только успешный close пустого файла ставит
//! `m_bClientClose = true`, вызывает готовый positional `LoadSetup` и начинает
//! `InitNetClient`; каждый нулевой результат создаёт operator-visible failed
//! attempt, затем ровно `8000 ms` паузы и новую попытку до первого успеха.
//! Legacy return самой функции равен `0` на всех путях, включая connected.
//!
//! `File::create` заменяет `fopen("debug.txt", "wb")/fclose`, а Tokio timer —
//! `Sleep(8000)`. Будущий Linux shutdown передаётся как повторно используемый
//! pinned future и может отменить connect либо паузу: оригинал не имел safe
//! owned-выхода из бесконечного Init-retry, но оставить будущий Rust lifecycle
//! навсегда заблокированным после сигнала нельзя. Cancellation получает
//! отдельный typed outcome и не выдаётся за старый return. `CBaseMessage::Initial`
//! и `CMySocket::MySocketInit` не имеют пустых вызовов: static message scratch
//! и WinSock startup уже заменены локальным message ownership и Linux
//! transport. Единственные Misc `srand(time)` и проигнорированный
//! `random(100)` не имеют последующего project-caller, поэтому отдельный
//! process-global PRNG не создаётся заранее.
//!
//! `ReFlushLog` сначала снимает summary из размера primary auction-map,
//! sync-флага и двух wrapping auction-счётчиков, затем выполняет
//! `PutMemCondition`, копирует не более первых десяти пар ordered message-map и
//! безусловно очищает весь map. MFC `SetWindowText`, `PutLogInfo` и
//! `AddLogText` заменены одним typed-отчётом в том же порядке; точные старые
//! форматы сохранены в документации его частей. Узкий accessor комнаты
//! возвращает прежний 32-битный `_Mysize`, не открывая её контейнер владельцу.
//!
//! Потерянные декомпилятором varargs имеют статус `VERIFIED_DISASSEMBLY` по
//! exact Misc EXE. `0x00401CEF..0x00401D06` передаёт соответственно offsets
//! `CGame+0x0C/+0x7D/+0x98/+0x9C`, а `0x00401D8C..0x00401D9E` — node key/value
//! `+0x0C/+0x10`. В `PutMemCondition` адреса
//! `0x0040176A..0x0040177F` сдвигают WorkingSet/Pagefile на двадцать бит, а
//! `0x00401784..0x004017A9` строго сравнивают несдвинутый WorkingSet с
//! `0x02800000` и при превышении сбрасывают sync-флаг перед новым boot tick.
//!
//! `procfs 0.18` заменяет `GetProcessMemoryInfo`: Linux `VmRSS` является
//! resident working set, а `VmSize` сохраняет ближайший доступный process-wide
//! address-space показатель для исторической подписи `PagefileUsage`. Только
//! `VmRSS` участвует в серверной мутации; различие Windows commit charge и
//! Linux virtual size остаётся операторской диагностической заменой. Ошибка
//! `/proc/self/status` сохраняет исходные предварительные нули и не сбрасывает
//! sync. Значения `kB` сдвигаются на десять бит, что эквивалентно старому
//! `bytes >> 20` без округления.
//!
//! `GameThreadFunc` владеет единственным локальным `CGame` вместо
//! `CreateGame/GetGame/g_pGame`. Каждый turn сначала обнуляет sync-count, затем
//! проверяет буквально `last.wrapping_add(10000) < now`, делает один
//! cancellation-safe readiness poll старого client I/O непосредственно перед
//! `AI`, после чего сохраняет порядок `AI -> ProcessMessage -> client-close ->
//! ReConnect -> 0x10EF00`. Read и send раньше жили в независимых Windows
//! threads, поэтому неблокирующий poll не вводит между ними новый порядок и не
//! задерживает доменный turn при отсутствии readiness; кадры, принятые poll-ом,
//! попадают в его текущий FIFO snapshot, а исходящие сообщения обслуживаются на
//! следующей cadence не позднее очередной паузы `50 ms`.
//!
//! Ошибка раннего `debug.txt` оставляла `m_bClientClose` неинициализированным.
//! Доказанные refresh/AI/message стадии первого turn выполняются, но safe Rust
//! останавливается ровно на чтении этого поля с
//! `BLOCKED_MISSING_FACT`: выбирать reconnect либо continue за старый UB нельзя.
//! Внешний shutdown является технической owned-заменой global exit flag и может
//! отменить ожидающий connect/I/O turn. После любой достигнутой границы
//! сохраняется `2005 ms -> exit notice -> CAuctionRoom::Clear`; `exit(0)` стал
//! возвращаемым legacy status, потому что доменный owner не завершает будущий
//! общий Linux-процесс самостоятельно.
//!
//! `VERIFIED_DISASSEMBLY`: exact EXE `0x00402959..0x00402977` показывает, что
//! compiler catch читает `CGame+0x88` (`m_dwCurMsg`), передаёт его формату
//! `mainloop... = %d ...` и возвращается на `0x00402978`, то есть сразу в общий
//! shutdown-tail, а не к следующему turn. Safe handlers уже возвращают typed
//! outcomes; SEH/COM, WinSock startup/cleanup, MFC logging, message destructor,
//! STL map allocation и ручные new/delete не получают пустых аналогов. `Drop`,
//! локальные сообщения, `BTreeMap`, Tokio и typed reports выражают их достигнутый
//! эффект. Сырой `AddLogText` ниже остаётся до реализации остальных владельцев,
//! которые ещё ссылаются на общий diagnostic helper.

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

use crate::miscserver::miscserver::miscservermessage::{WorldAuctionOutcome, on_msg_w2m_auction};
use crate::miscserver::miscserver::onbillserver::{MiscFunctionOutcome, on_msg_m2m_function};
use crate::miscserver::miscserver::othermessage::{OtherMessageOutcome, on_other_msg};
use crate::miscserver::miscserver::setup::setup::{CSetup, SetupLoadReport, SetupOpenError};
use crate::nets::clients::{ClientConnectError, ClientSendQueue};
use crate::nets::netmisc::message::{CMessage, MessageHandlers, MessageSender, SendMessageError};
use crate::nets::netmisc::mynetclient::{CMyNetClient, MiscClientIoError, MiscClientIoStep};
use crate::public::aucitionroom::{AddAuctionItemMissingGoodsType, CAuctionRoom};
use crate::public::auctionnode::CGoodsNode;
use crate::transport::bind_tcp_ipv4;

const INITIAL_REGISTRATION_TYPE: i32 = 0x0005_FA01;
const INITIAL_SYNC_TYPE: i32 = 0x0015_EB05;
const RECONNECT_LISTEN_PORT: u16 = 0x092F;
const KIB_PER_MIB: u64 = 1_024;
const MEMORY_SYNC_RESET_THRESHOLD_KIB: u64 = 0x0280_0000 / 1_024;
const LOG_REFRESH_INTERVAL_MS: u32 = 10_000;
const GAME_THREAD_DELAY: Duration = Duration::from_millis(50);
const GAME_THREAD_SHUTDOWN_DELAY: Duration = Duration::from_millis(2_005);
const RECONNECT_CONFIRMATION_TYPE: i32 = 0x0010_EF00;

/// Причина нулевого результата исходного connect-прохода.
#[derive(Debug)]
pub(crate) enum MiscClientConnectFailure {
    MissingSetupField(&'static str),
    WorldAddressResolution { host: Vec<u8> },
    Bind(io::Error),
    Connect(ClientConnectError),
}

/// Два исходно игнорировавшихся результата постановки registration packets.
#[derive(Debug)]
pub(crate) struct MiscRegistrationReport {
    pub(crate) registration: Result<i32, SendMessageError>,
    pub(crate) initial_sync: Option<Result<i32, SendMessageError>>,
}

/// Полный итог одной попытки `InitNetClient` либо `ReConnect`.
#[derive(Debug)]
pub(crate) enum MiscClientConnectOutcome {
    Connected {
        endpoint: SocketAddrV4,
        sends: MiscRegistrationReport,
    },
    Failed(MiscClientConnectFailure),
}

/// Причина завершения исходного init-прохода либо его Linux cancellation.
#[derive(Debug)]
pub(crate) enum MiscInitializationEnd {
    /// Ранний write-open `debug.txt` не состоялся; остальные стадии пропущены.
    DebugFileUnavailable { path: PathBuf, source: io::Error },
    /// `InitNetClient` впервые вернул исходный успех.
    Connected,
    /// Будущий owned lifecycle запросил shutdown во время connect либо паузы.
    Cancelled,
}

/// Полный typed-отчёт исходного `CGame::Init`.
#[derive(Debug)]
pub(crate) struct MiscInitializationReport {
    /// Исходная функция возвращала `0` на любом пути.
    pub(crate) legacy_result: i32,
    /// Результат `LoadSetup`; отсутствует только при ранней ошибке debug-файла.
    pub(crate) setup: Option<Result<SetupLoadReport, SetupOpenError>>,
    /// Connect-попытки в исходном порядке, включая последнюю успешную.
    pub(crate) attempts: Vec<MiscClientConnectOutcome>,
    pub(crate) end: MiscInitializationEnd,
}

/// Значения Linux `/proc/self/status`, соответствующие старой memory diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiscProcessMemorySnapshot {
    /// `VmRSS` в единицах procfs `kB`.
    pub(crate) working_set_kib: Option<u64>,
    /// `VmSize` в единицах procfs `kB` для старой подписи `PagefileUsage`.
    pub(crate) pagefile_usage_kib: Option<u64>,
}

/// Результат безопасной Linux-замены `GetProcessMemoryInfo`.
#[derive(Debug)]
pub(crate) enum MiscProcessMemoryQuery {
    Read(MiscProcessMemorySnapshot),
    /// Старый zero-initialized `PROCESS_MEMORY_COUNTERS` оставался нулевым.
    Unavailable(ProcError),
}

/// Полный результат `CGame::PutMemCondition` RVA `0x00001720`.
///
/// Основная строка имела точный формат
/// `WorkingSetSize = %u(M)  PagefileUsage = %u(M) \n`; при reset следовала
/// CP936-строка `内存大于40M,暂停同步消息`.
#[derive(Debug)]
pub(crate) struct MiscMemoryConditionReport {
    pub(crate) query: MiscProcessMemoryQuery,
    /// Первый `%u(M)`: floor `WorkingSetSize / 2^20`.
    pub(crate) working_set_mib: u32,
    /// Второй `%u(M)`: floor `PagefileUsage / 2^20`.
    pub(crate) pagefile_usage_mib: u32,
    /// Был ли строго превышен порог `0x02800000` и сброшен auction sync.
    pub(crate) auction_sync_reset: bool,
}

/// Первая строка старого `ReFlushLog` до memory-проверки с точным форматом
/// `AuctionGoodsCount = %ld  SyscSign = %d AddMsg %u DisMsg %u \r\n`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiscAuctionStatusSnapshot {
    /// Старый формат: `AuctionGoodsCount = %ld`.
    pub(crate) auction_goods_count: i32,
    /// Старый формат: `SyscSign = %d`, всегда `0` либо `1`.
    pub(crate) sync_sign: i32,
    /// Старый формат: `AddMsg %u`.
    pub(crate) added_messages: u32,
    /// Старый формат: `DisMsg %u`.
    pub(crate) discarded_messages: u32,
}

/// Одна из не более десяти строк ordered message-map с точным форматом
/// ` MsgType = %u   MsgCount = %u \n`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiscMessageCountSnapshot {
    /// Старый формат: `MsgType = %u`.
    pub(crate) message_type: u32,
    /// Старый формат: `MsgCount = %u`.
    pub(crate) message_count: u32,
}

/// Typed-замена GUI/file diagnostics одного `ReFlushLog`.
#[derive(Debug)]
pub(crate) struct MiscLogRefreshReport {
    pub(crate) status: MiscAuctionStatusSnapshot,
    pub(crate) memory: MiscMemoryConditionReport,
    /// Первые десять signed-map ключей в возрастающем порядке, показанные как `%u`.
    pub(crate) message_counts: Vec<MiscMessageCountSnapshot>,
}

impl MiscClientConnectOutcome {
    /// Возвращает исходный `1/0` без потери typed diagnostic.
    pub(crate) const fn legacy_result(&self) -> i32 {
        match self {
            Self::Connected { .. } => 1,
            Self::Failed(_) => 0,
        }
    }
}

/// Результат фактического владельца, выбранного `CMessage::Run`.
#[derive(Debug)]
pub(crate) enum MiscComponentHandlerOutcome {
    /// Семейство `0x14EC00` вызвало пустой деструктор GUID.
    GuidFamilyNoOp,
    WorldAuction(WorldAuctionOutcome),
    MiscFunction(MiscFunctionOutcome),
    Other(OtherMessageOutcome),
}

/// Итог одного сообщения внутри исходного FIFO snapshot.
#[derive(Debug)]
pub(crate) struct MiscComponentMessageOutcome {
    pub(crate) message_type: i32,
    pub(crate) legacy_run_result: i32,
    pub(crate) handler: MiscComponentHandlerOutcome,
}

/// Итог одного полного `CGame::ProcessMessage` snapshot.
#[derive(Debug)]
pub(crate) struct MiscProcessMessageOutcome {
    /// Исходный безусловный return `ProcessMessage` равен `0`.
    pub(crate) legacy_result: i32,
    /// Результаты сообщений в точном порядке снятого WorldServer FIFO.
    pub(crate) messages: Vec<MiscComponentMessageOutcome>,
}

/// Результаты ветви `m_bClientClose != false` одного game-thread turn.
#[derive(Debug)]
pub(crate) struct MiscReconnectTurn {
    /// Исходный operator notice `ReConnect....` достигнут.
    pub(crate) reconnect_notice: bool,
    pub(crate) connection: MiscClientConnectOutcome,
    /// `0x10EF00` ставится после попытки reconnect даже при её неуспехе.
    pub(crate) confirmation: Result<i32, SendMessageError>,
}

/// Полный результат одного доказанного доменного turn до паузы `50 ms`.
pub(crate) struct MiscGameThreadTurn {
    /// Один неблокирующий I/O-шаг; `None` означает отсутствие readiness/client.
    pub(crate) network: Option<Result<MiscClientIoStep, MiscClientIoError>>,
    pub(crate) refresh: Option<MiscLogRefreshReport>,
    pub(crate) auction: crate::public::aucitionroom::AuctionAiOutcome,
    pub(crate) messages: MiscProcessMessageOutcome,
    pub(crate) reconnect: Option<MiscReconnectTurn>,
}

/// Недоказанная safe-граница внешнего main-loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MiscGameThreadRuntimeError {
    /// `Init` не записал старое поле после ошибки создания `debug.txt`.
    ClientCloseUninitializedReactionUnknown,
}

/// Выполненный общий tail оригинального `GameThreadFunc`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MiscGameThreadRelease {
    /// Исходный operator notice `exit` достигнут после паузы `2005 ms`.
    pub(crate) exit_notice: bool,
    /// Старый `exit(0)` возвращён владельцу будущего общего process lifecycle.
    pub(crate) legacy_exit_status: i32,
}

/// Итог единственного owned аналога исходного `GameThreadFunc`.
pub(crate) struct MiscGameThreadReport {
    pub(crate) initialization: MiscInitializationReport,
    pub(crate) completed_turns: u64,
    pub(crate) runtime_error: Option<MiscGameThreadRuntimeError>,
    pub(crate) release: MiscGameThreadRelease,
}

/// Достигнутая setup/network-часть исходного `CGame`.
pub(crate) struct CGame {
    setup: CSetup,
    auction_room: CAuctionRoom<CGoodsNode>,
    net_client: Option<CMyNetClient>,
    client_close: Option<bool>,
    current_message: u32,
    message_record: BTreeMap<i32, i32>,
    add_new_count: u32,
    deleted_new_count: u32,
    done_sync_message: bool,
    sync_start_time: u32,
    done_sync_count: Option<u32>,
    last_log_refresh_time: u32,
}

impl CGame {
    /// Создаёт partial owner с явно переданным единственным `CSetup`.
    pub(crate) fn with_setup(setup: CSetup) -> Self {
        Self {
            setup,
            auction_room: CAuctionRoom::new(),
            net_client: None,
            client_close: None,
            current_message: 0,
            message_record: BTreeMap::new(),
            add_new_count: 0,
            deleted_new_count: 0,
            done_sync_message: false,
            sync_start_time: legacy_tick_ms(),
            done_sync_count: None,
            last_log_refresh_time: 0,
        }
    }

    /// Создаёт единственный game-thread owner с исходно пустым setup singleton.
    pub(crate) fn new() -> Self {
        Self::with_setup(CSetup::new())
    }

    /// Возвращает setup для исходной позиции `CGame::Init -> LoadSetup`.
    pub(crate) const fn setup(&self) -> &CSetup {
        &self.setup
    }

    /// Даёт будущему `Init` единственный изменяемый setup-owner.
    pub(crate) fn setup_mut(&mut self) -> &mut CSetup {
        &mut self.setup
    }

    /// Выполняет исходный file/setup/connect init до успеха либо shutdown.
    ///
    /// `shutdown` остаётся пригодным будущему caller-у после успешного init.
    pub(crate) async fn initialize<Shutdown>(
        &mut self,
        runtime_directory: &Path,
        mut shutdown: Pin<&mut Shutdown>,
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
                    attempts: Vec::new(),
                    end: MiscInitializationEnd::DebugFileUnavailable {
                        path: debug_path,
                        source,
                    },
                };
            }
        };
        drop(debug_file);

        self.client_close = Some(true);
        let setup = self.setup.load_setup(runtime_directory.join("setup.ini"));
        let mut attempts = Vec::new();

        loop {
            let attempt = tokio::select! {
                biased;
                () = shutdown.as_mut() => {
                    return MiscInitializationReport {
                        legacy_result: 0,
                        setup: Some(setup),
                        attempts,
                        end: MiscInitializationEnd::Cancelled,
                    };
                }
                attempt = self.init_net_client() => attempt,
            };
            let connected = attempt.legacy_result() != 0;
            attempts.push(attempt);
            if connected {
                return MiscInitializationReport {
                    legacy_result: 0,
                    setup: Some(setup),
                    attempts,
                    end: MiscInitializationEnd::Connected,
                };
            }

            tokio::select! {
                biased;
                () = shutdown.as_mut() => {
                    return MiscInitializationReport {
                        legacy_result: 0,
                        setup: Some(setup),
                        attempts,
                        end: MiscInitializationEnd::Cancelled,
                    };
                }
                () = tokio::time::sleep(Duration::from_millis(8_000)) => {}
            }
        }
    }

    /// Сообщает достигнутое значение старого `m_bClientClose`.
    pub(crate) const fn client_close(&self) -> Option<bool> {
        self.client_close
    }

    /// Даёт достигнутым auction-handler единственный mutable owner комнаты.
    pub(crate) fn auction_room_mut(&mut self) -> &mut CAuctionRoom<CGoodsNode> {
        &mut self.auction_room
    }

    /// Даёт sync-handler неизменяемый owner комнаты для ordered merge.
    pub(crate) const fn auction_room(&self) -> &CAuctionRoom<CGoodsNode> {
        &self.auction_room
    }

    /// Увеличивает `m_dwAddNewCount` в исходной позиции auction-handler-а.
    pub(crate) fn count_new_auction_item(&mut self) {
        self.add_new_count = self.add_new_count.wrapping_add(1);
    }

    /// Передаёт owned-узел комнате вместе с её исходным delete-счётчиком.
    pub(crate) fn add_auction_item(
        &mut self,
        item: Box<CGoodsNode>,
    ) -> Result<bool, AddAuctionItemMissingGoodsType> {
        self.auction_room
            .add_item_to_auction_room(item, &mut self.deleted_new_count)
    }

    /// Возвращает wrapping-число созданных handler-ом auction-узлов.
    pub(crate) const fn add_new_count(&self) -> u32 {
        self.add_new_count
    }

    /// Возвращает wrapping-число удалённых комнатой новых auction-узлов.
    pub(crate) const fn deleted_new_count(&self) -> u32 {
        self.deleted_new_count
    }

    /// Воспроизводит запись `m_dwDoneSysnCount = 0` в начале game-thread turn.
    pub(crate) fn begin_auction_sync_turn(&mut self) {
        self.done_sync_count = Some(0);
    }

    /// Возвращает turn-local sync-count либо неинициализированную границу.
    pub(crate) const fn auction_sync_count(&self) -> Option<u32> {
        self.done_sync_count
    }

    /// Возвращает разрешение `m_bDoneSyscMsg` на фактическую синхронизацию.
    pub(crate) const fn auction_sync_enabled(&self) -> bool {
        self.done_sync_message
    }

    /// Возвращает wrapping boot tick начала ожидания sync.
    pub(crate) const fn auction_sync_start_time(&self) -> u32 {
        self.sync_start_time
    }

    /// Ставит исходный sync-флаг после строгого 120-секундного условия.
    pub(crate) fn enable_auction_sync(&mut self) {
        self.done_sync_message = true;
    }

    /// Отмечает единственный выполненный sync текущего game-thread turn.
    pub(crate) fn finish_auction_sync_turn(&mut self) {
        self.done_sync_count = Some(1);
    }

    /// Ставит исходный `m_bClientClose` перед блокирующим `ReConnect`.
    pub(crate) fn mark_client_closed(&mut self) {
        self.client_close = Some(true);
    }

    /// Возвращает текущий nullable WorldServer client.
    pub(crate) const fn net_client(&self) -> Option<&CMyNetClient> {
        self.net_client.as_ref()
    }

    /// Выполняет initial connect и оба исходных registration send.
    pub(crate) async fn init_net_client(&mut self) -> MiscClientConnectOutcome {
        self.connect_world(None, true).await
    }

    /// Пересоздаёт WorldServer client и использует исторический port `0x092F`.
    pub(crate) async fn reconnect(&mut self) -> MiscClientConnectOutcome {
        self.connect_world(Some(RECONNECT_LISTEN_PORT), false).await
    }

    /// Выполняет один awaitable read/send шаг текущего client-owner.
    pub(crate) async fn run_client_io_once(
        &mut self,
    ) -> Result<MiscClientIoStep, MiscClientIoError> {
        let client = self
            .net_client
            .as_mut()
            .ok_or(MiscClientIoError::NotConnected)?;
        client.run_io_once(legacy_tick_ms).await
    }

    /// Делает один cancellation-safe readiness poll бывших socket threads.
    pub(crate) async fn poll_client_io_once(
        &mut self,
    ) -> Option<Result<MiscClientIoStep, MiscClientIoError>> {
        match self.net_client.as_mut() {
            Some(client) if client.is_connected() => {
                poll_once(client.run_io_once(legacy_tick_ms)).await
            }
            Some(_) | None => None,
        }
    }

    /// Возвращает send-очередь текущего client доменным producers.
    pub(crate) fn client_send_queue(&self) -> Option<&ClientSendQueue> {
        self.net_client.as_ref().map(CMyNetClient::send_queue)
    }

    /// Выполняет один атомарный snapshot исходного `CGame::ProcessMessage`.
    ///
    /// Сообщения, опубликованные callback-ами после снятия FIFO, остаются
    /// следующему проходу. Reconnect полностью завершается до следующего
    /// элемента; отчёт сохраняет исходный безусловный return `0`.
    pub(crate) async fn process_message(&mut self) -> MiscProcessMessageOutcome {
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

    /// Возвращает полный opcode сообщения, которое сейчас исполняет handler.
    pub(crate) const fn current_message(&self) -> u32 {
        self.current_message
    }

    /// Возвращает ordered signed-счётчики обработанных opcode.
    pub(crate) const fn message_record(&self) -> &BTreeMap<i32, i32> {
        &self.message_record
    }

    /// Снимает process memory и при строгом превышении 40 MiB сбрасывает sync.
    ///
    /// Неиспользуемый `legacy_argument` сохраняет форму исходного `ulong`.
    pub(crate) fn put_mem_condition(&mut self, _legacy_argument: u32) -> MiscMemoryConditionReport {
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

    /// Возвращает один исходно упорядоченный diagnostic-report и очищает map.
    pub(crate) fn reflush_log(&mut self) -> MiscLogRefreshReport {
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

    /// Выполняет доказанный порядок одного `GameThreadFunc` turn до `Sleep(50)`.
    pub(crate) async fn run_game_thread_turn(
        &mut self,
    ) -> Result<MiscGameThreadTurn, MiscGameThreadRuntimeError> {
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
        let auction = {
            let sender = self
                .net_client
                .as_ref()
                .map(|client| client as &dyn MessageSender);
            self.auction_room.ai(sender)
        };
        let messages = self.process_message().await;
        let reconnect = match self.client_close {
            Some(false) => None,
            Some(true) => {
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
            }
            None => {
                // BLOCKED_MISSING_FACT: при ошибке `fopen(debug.txt, "wb")`
                // Init RVA 0x00001690 не записывает `m_bClientClose`, а
                // GameThreadFunc RVA 0x00002870 читает байт `CGame+0x7C` после
                // уже выполненных refresh/AI/ProcessMessage. Наблюдаемая ветвь
                // старого неинициализированного значения неизвестна.
                return Err(MiscGameThreadRuntimeError::ClientCloseUninitializedReactionUnknown);
            }
        };

        Ok(MiscGameThreadTurn {
            network,
            refresh,
            auction,
            messages,
            reconnect,
        })
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
        registration.base_mut().add_word(registration_port);
        add_legacy_c_string(registration.base_mut(), local_ip);
        let registration = registration.send(Some(&client), false);
        let initial_sync =
            send_initial_sync.then(|| CMessage::new(INITIAL_SYNC_TYPE).send(Some(&client), false));

        self.client_close = Some(false);
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
    setup: &crate::miscserver::miscserver::setup::setup::IpPortSetup,
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

fn add_legacy_c_string(message: &mut crate::nets::basemessage::CBaseMessage, value: &[u8]) {
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

pub(crate) fn legacy_tick_ms() -> u32 {
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

/// Выполняет полный lifecycle `GameThreadFunc` RVA `0x00002870`.
///
/// `shutdown` заменяет только внешний `g_bMainThreadExit`. Локальный owner
/// заменяет `g_pGame`; после любой достигнутой границы аукционная комната
/// очищается в исходной позиции общего shutdown-tail.
pub(crate) async fn game_thread_func<Shutdown>(
    runtime_directory: &Path,
    shutdown: Shutdown,
) -> MiscGameThreadReport
where
    Shutdown: Future<Output = ()>,
{
    let mut game = CGame::new();
    tokio::pin!(shutdown);
    let initialization = game.initialize(runtime_directory, shutdown.as_mut()).await;
    let init_cancelled = matches!(&initialization.end, MiscInitializationEnd::Cancelled);
    let mut completed_turns = 0_u64;
    let mut runtime_error = None;

    if !init_cancelled {
        'main_loop: loop {
            let turn = tokio::select! {
                biased;
                () = shutdown.as_mut() => break 'main_loop,
                result = game.run_game_thread_turn() => result,
            };
            match turn {
                Ok(_outcome) => {
                    completed_turns = completed_turns.wrapping_add(1);
                }
                Err(error) => {
                    runtime_error = Some(error);
                    break;
                }
            }
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
        runtime_error,
        release,
    }
}

// COMPONENT_VARIANT_BEGIN: MiscServer
// Точная пара: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SHA-256 EXE: F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65
// SHA-256 PDB: ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7
// Исходный владелец PDB: h:\fengyun\fy_russia\src\server\miscserver\miscserver\game.cpp

// ============================================================================
// FUNCTION: AddLogText
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\miscserver\miscserver\game.cpp:83
// RVA: 0x00001070
// ADDRESS: 00401070
// PROTOTYPE: void __cdecl AddLogText(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: MiscServer
