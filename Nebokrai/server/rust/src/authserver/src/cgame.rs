//! Runtime-владелец `CGame` AuthServer из `authserver/src/cgame.cpp/.h`.
//!
//! Статус владельца смешанный: `IMPLEMENTED` для сохранённого `mConfiger`,
//! DB-очередей, `PushDBQuest`, owned DB workers, `ProcessMessage`,
//! `ProcessDBResult`, `UpdateServerInfo`, `MainLoop`, `GameThreadFunc`, узкой
//! Linux-замены accept/net/worker lifecycle и внешнего порядка `Init/Release`;
//! остальные доменные функции ниже сохраняют `UNKNOWN` (исследовательский декомпилят хранится локально) до проходов.
//!
//! Точная пара: `AuthServer/authserver.exe + AuthServer/authserver.pdb`;
//! SHA-256 EXE
//! `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15`,
//! SHA-256 PDB
//! `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`.
//! Исходные пути PDB:
//! `h:\fengyun\fy_russia\src\server\authserver\src\cgame.cpp` и
//! `h:\fengyun\fy_russia\src\server\authserver\src\cgame.h`.
//!
//! Существенные RVA: `ProcessMessage` `0x00002090`, `InitNetServer_Auth`
//! `0x00002AF0`, `Release` `0x000034C0`, `gmaSendMessage` `0x00003590`, три
//! `Send*Result` `0x00004370/0x00004460/0x000045C0`, `ProcessDBResult`
//! `0x00004E70`, `PushDBQuest` `0x00005EC0`, `UpdateServerInfo` `0x00006310`,
//! `MainLoop` `0x00006890`, `GameThreadFunc` `0x000068C0`.
//! Состояние `mLSList`,
//! `CheckConnection` и helpers LoginServer реализованы конкретным handler-state
//! в `appauth/message/message_func.rs`; их provenance перенесён туда, а старые
//! STL/MFC тела здесь удалены.
//!
//! Оригинал держал accept-, net-, IOCP-worker- и game-thread раздельно. Их
//! относительный порядок зависел от Windows scheduler, но каждый net snapshot
//! и каждый message snapshot имел собственную доказанную границу. Rust
//! сериализует короткие state-owner проходы в одном `run_network_turn`, а
//! блокирующий accept держит отдельным `JoinHandle`, read/send — в
//! `tokio::task::JoinSet`. Это сохраняет очереди и не оставляет detached-задач.
//! `release_auth_network` отдельно отменяет accept, ставит `QUITALL`, ждёт
//! удаления всех clients и лишь затем останавливает оставшиеся I/O-задачи.
//! DB workers разделяют только config/DB queues/server-info/IP-filter через
//! `AuthDbContext`; network и handler-state никогда не уходят в их потоки.
//! Config защищён `RwLock`, поэтому reload и одна DB-команда видят целостные,
//! а не частично изменённые credential/procedure bytes.
//! `rustix::time::clock_gettime(CLOCK_BOOTTIME)` заменяет `timeGetTime`: safe
//! API возвращает suspend-aware время с запуска Linux, а явное сужение
//! сохраняет 32-битное wrapping миллисекунд.
//!
//! Accept сохраняет исходные ожидания: `100 ms` после каждого результата и
//! `1 s` после обнаруженного client limit. Управляемый `CGame` turn завершается
//! исходной паузой `1 ms`. `ProcessMessage` и `ProcessDBResult` по одному разу
//! читают размер своих FIFO и выполняют ровно столько извлечений; добавленные
//! во время обработки элементы остаются следующему turn. Rust-владение
//! заменяет virtual delete после `Run` и ручное удаление DB payload.
//!
//! Три DB-ответа сохраняют wire `0xCF601`, `0xCF602`, `0x10F202`, порядок
//! result/account/client IP/client socket и однобайтовый MSVC `bool`. Для
//! расширенного result `3` оригинал отправлял пять `SYSTEMTIME`-слов, пропуская
//! day-of-week и seconds; result `7` отправлял все 80 opaque bytes. Account
//! остаётся байтовой C-строкой с одним завершающим нулём.
//!
//! `UpdateServerInfo` сохраняет process-static время первого вызова, два
//! независимых интервала и 32-битное wrapping сравнение `last + interval < now`.
//! Due-запрос `0xCF702` идёт зарегистрированным LoginServer в порядке area ID;
//! due-запись ставит `WriteServerInfo`, а timer обновляется даже при отказе
//! переполненной DB-очереди. Ответ `0xCF802` принадлежит конкретному Auth-
//! handler и публикует player/GS/WS/LS tuple в coalescing `ServerInfoQueue`.
//! `parking_lot::Mutex` даёт безопасную interior mutability двум function-
//! static timer; при единственном исходном game-owner он не добавляет нового
//! межсервисного порядка и не заменяет доменную проверку интервалов.
//!
//! `InitNetServer_Auth` восстановлен целиком: прежний owner освобождается,
//! `allowed_ls.ini` загружается до `Host`, после успешного listen независимо
//! разрешаются DWORD и строковая формы первого IPv4 локального hostname, затем
//! применяются setup-лимиты. Rustix `uname().nodename()` заменяет Linux
//! `gethostname`, а `ToSocketAddrs` — legacy `gethostbyname`; неуспешное
//! разрешение сохраняет исходные `0`/пустую строку. Поздняя запись backlog
//! `10` по-прежнему не меняет уже созданный listener, а `SendInterTime` задаёт
//! таймаут первого пакета.
//!
//! Внешний `Init` сохраняет порядок: `setup.ini` с reset при любой ошибке,
//! одноэкземплярная проверка, message/socket owners, listener, DB workers и
//! необязательный `client_forbid_ip.ini`. Исходный `CheckOneInstance` не владел
//! mutex: он искал GUI-окно с заголовком `AuthServer [Port %d]`. На целевом
//! Linux без GUI тот же процессный инвариант обеспечивает эксклюзивный bind
//! listener того же порта, поэтому отдельный lock-файл не создаётся. Статическая
//! Auth-таблица обработчиков заменяет `InitMsgFuncPool`, локальные message-
//! буферы — `CBaseMessage::Initial/Release`, Tokio socket ownership —
//! `CMySocket::MySocketInit/MySocketClearUp`; пустых lifecycle-вызовов нет.
//! `Release` сначала останавливает и присоединяет DB workers, затем выполняет
//! полный network shutdown. Ошибка после частичного `Init` не откатывает уже
//! созданных владельцев: вызывающий, как исходный `GameThreadFunc`, обязан
//! всегда вызвать `release_auth_runtime`.
//!
//! `GameThreadFunc` теперь владеет единственным локальным `CGame`: Rust stack-
//! ownership заменяет `g_Game`, `CreateGame/GetGame/DeleteGame` и ручные
//! `new/delete`, не создавая глобальный accessor. После `Init` он выполняет
//! `MainLoop` до переданного shutdown future, затем всегда выполняет `Release`;
//! при ошибке runtime порядок освобождения не меняется. Windows thread-message
//! `0x464`, `_beginthreadex/_endthreadex` и финальный `PostMessageA` заменены
//! прямым async-вызовом из Linux entry point и `SIGINT/SIGTERM`: они управляли
//! только техническим lifecycle и не получают пустых Rust-аналогов.

use std::error::Error;
use std::fmt;
use std::future::Future;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use rustix::system::uname;
use rustix::time::{ClockId, clock_gettime};
use tokio::net::TcpStream;
use tokio::task::{JoinError, JoinHandle, JoinSet};
use tokio::time;

use crate::authserver::appauth::message::message_func::AuthMessageHandlers;
use crate::authserver::src::configreader::{ConfigLoadError, ConfigReader};
use crate::authserver::src::dbqueue::{
    AuthExResultData, AuthResultData, DbQuest, DbResult, LockResultData, ServerInfo,
    ServerInfoQueue,
};
use crate::authserver::src::kl_ipfilter::{IpFilter, IpFilterLoadError, load_ip_patterns};
use crate::authserver::src::kl_multi_list::MultiList;
use crate::dbaccess::authdb::authproc::{
    AuthDatabaseNotice, AuthDatabaseWorkerJoinError, AuthDatabaseWorkerStartError,
    AuthDatabaseWorkers,
};
use crate::nets::mysocket::{DEFAULT_SOCKET_TYPE, legacy_ipv4_word};
use crate::nets::netauth::message::{
    AuthMessageHandler, CMessage, DispatchError, SendMessageError,
};
use crate::nets::netauth::mynetserver_auth::CMyNetServerAuth;
use crate::nets::netauth::mynetserverclient_auth::AuthReceiveError;
use crate::nets::servers::{
    ACCEPT_AT_CAPACITY_DELAY, ACCEPT_THREAD_DELAY, AcceptStart, AdmissionOutcome, NET_THREAD_DELAY,
    ServerHostError, ServerIoAction, ServerIoCompletion, ServerSnapshotError,
};

/// Сетевые Auth-параметры исходного `mConfiger`.
pub(crate) struct AuthNetworkConfig {
    host_port: u32,
    max_login_servers: i32,
    max_in_flight_sends: i32,
    permitted_send_bytes: i32,
    new_accept_timeout_ms: i32,
}

impl AuthNetworkConfig {
    /// Создаёт уже разобранную конфигурацию без новой грамматики `setup.ini`.
    pub(crate) const fn new(
        host_port: u32,
        max_login_servers: i32,
        max_in_flight_sends: i32,
        permitted_send_bytes: i32,
        new_accept_timeout_ms: i32,
    ) -> Self {
        Self {
            host_port,
            max_login_servers,
            max_in_flight_sends,
            permitted_send_bytes,
            new_accept_timeout_ms,
        }
    }
}

/// Наблюдаемые результаты одного управляемого Auth runtime-turn.
#[derive(Default)]
pub(crate) struct AuthRuntimeStep {
    /// Число команд атомарного общего net snapshot.
    pub(crate) processed_network_commands: i32,
    /// Число сообщений исходного snapshot `CGame::ProcessMessage`.
    pub(crate) processed_messages: i32,
    /// Число ответов исходного snapshot `CGame::ProcessDBResult`.
    pub(crate) processed_database_results: i32,
    /// Истёк ли интервал broadcast-запроса актуального server-info.
    pub(crate) server_info_requested: bool,
    /// Истёк ли интервал постановки coalesced server-info в DB-очередь.
    pub(crate) server_info_write_due: bool,
    /// Приняла ли DB-очередь due-команду записи server-info.
    pub(crate) server_info_write_queued: bool,
    /// Результаты admission завершившихся accept-операций.
    pub(crate) admissions: Vec<AdmissionOutcome>,
    /// Завершившиеся transport-задачи, включая штатный EOF.
    pub(crate) io_completions: Vec<ServerIoCompletion>,
    /// Локальные component/size-неизвестности net snapshot.
    pub(crate) network_errors: Vec<ServerSnapshotError<AuthReceiveError>>,
    /// Системные ошибки listener accept; следующий accept остаётся разрешён.
    pub(crate) accept_errors: Vec<io::Error>,
}

/// Ошибка, которая не позволяет продолжить управляемый Auth runtime.
#[derive(Debug)]
pub(crate) enum AuthRuntimeError {
    /// Runtime-turn вызван до успешного создания listener.
    NetworkNotInitialized,
    /// Listener не удалось создать из подтверждённых параметров `Host`.
    Host(ServerHostError),
    /// Управляемая Tokio-задача была отменена извне либо завершилась panic.
    Task(JoinError),
    /// Message dispatch достиг локально не восстановленного обработчика.
    Dispatch(DispatchError),
    /// Исходящее Auth-сообщение не представимо в legacy wire-диапазоне.
    MessageBuild(SendMessageError),
}

impl fmt::Display for AuthRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkNotInitialized => {
                formatter.write_str("сетевой runtime AuthServer ещё не инициализирован")
            }
            Self::Host(error) => write!(formatter, "AuthServer не смог открыть listener: {error}"),
            Self::Task(error) => {
                write!(formatter, "задача AuthServer завершилась аварийно: {error}")
            }
            Self::Dispatch(error) => write!(
                formatter,
                "обработка Auth-сообщения остановлена на неизвестном пути: {error}"
            ),
            Self::MessageBuild(error) => {
                write!(formatter, "не удалось построить Auth-сообщение: {error}")
            }
        }
    }
}

impl Error for AuthRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Host(error) => Some(error),
            Self::Task(error) => Some(error),
            Self::MessageBuild(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::NetworkNotInitialized => None,
        }
    }
}

impl From<SendMessageError> for AuthRuntimeError {
    fn from(error: SendMessageError) -> Self {
        Self::MessageBuild(error)
    }
}

struct ServerInfoTimers {
    last_update_ms: Option<u32>,
    last_write_ms: Option<u32>,
}

static SERVER_INFO_TIMERS: Mutex<ServerInfoTimers> = Mutex::new(ServerInfoTimers {
    last_update_ms: None,
    last_write_ms: None,
});

struct ServerInfoUpdate {
    requested: bool,
    write_due: bool,
    write_queued: bool,
}

/// Нефатальное наблюдаемое событие исходного `CGame::Init`.
#[derive(Debug)]
pub(crate) enum AuthInitializationNotice {
    /// `setup.ini` не прочитан; перед продолжением применены исходные defaults.
    SetupDefaultsApplied(ConfigLoadError),
    /// `allowed_ls.ini` не прочитан, но listener по исходному порядку запущен.
    AllowedClientsUnavailable(io::Error),
    /// Необязательный deny-list клиентов не прочитан; `Init` всё равно успешен.
    ClientIpFilterUnavailable(IpFilterLoadError),
}

impl fmt::Display for AuthInitializationNotice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetupDefaultsApplied(error) => write!(
                formatter,
                "Auth setup.ini отклонён, применены исходные defaults: {error}"
            ),
            Self::AllowedClientsUnavailable(error) => write!(
                formatter,
                "Auth allowed_ls.ini не загружен, сохранён исходный запуск listener: {error}"
            ),
            Self::ClientIpFilterUnavailable(error) => write!(
                formatter,
                "Auth client_forbid_ip.ini не загружен, сохранён исходный запуск: {error}"
            ),
        }
    }
}

/// Итог успешной внешней инициализации Auth runtime.
#[derive(Debug)]
pub(crate) struct AuthInitializationReport {
    /// Нефатальные ошибки файлов, которые оригинал только писал в журнал.
    pub(crate) notices: Vec<AuthInitializationNotice>,
    /// Фактическое значение исходного операторского сообщения о server-info.
    pub(crate) update_server_info_enabled: bool,
}

/// Фатальная граница внешнего `CGame::Init`.
#[derive(Debug)]
pub(crate) enum AuthInitializationError {
    /// Listener или прежний network-owner не удалось подготовить.
    Network(AuthRuntimeError),
    /// Настроенное число owned DB workers не удалось запустить полностью.
    DatabaseWorkers(AuthDatabaseWorkerStartError),
}

impl fmt::Display for AuthInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(error) => write!(
                formatter,
                "не удалось инициализировать Auth network: {error}"
            ),
            Self::DatabaseWorkers(error) => {
                write!(
                    formatter,
                    "не удалось инициализировать Auth DB workers: {error}"
                )
            }
        }
    }
}

impl Error for AuthInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Network(error) => Some(error),
            Self::DatabaseWorkers(error) => Some(error),
        }
    }
}

/// Итог полного обратного прохода `CGame::Release`.
#[derive(Debug)]
pub(crate) struct AuthReleaseReport {
    /// DB workers, завершившиеся panic вместо штатного выхода.
    pub(crate) database_worker_errors: Vec<AuthDatabaseWorkerJoinError>,
    /// Ошибка присоединения network-задачи после уже выполненного DB shutdown.
    pub(crate) network_error: Option<AuthRuntimeError>,
}

/// Полный итог одного прохода исходного `GameThreadFunc`.
#[derive(Debug)]
pub(crate) struct AuthGameThreadReport {
    /// Результат `CGame::Init`, сохранённый до обязательного release.
    pub(crate) initialization: Result<AuthInitializationReport, AuthInitializationError>,
    /// Первая фатальная ошибка `MainLoop`; нормальный сигнал даёт `None`.
    pub(crate) runtime_error: Option<AuthRuntimeError>,
    /// Итог обязательного `CGame::Release` после init и main-loop.
    pub(crate) release: AuthReleaseReport,
}

/// Три относительных runtime-файла исторического каталога AuthServer.
#[derive(Debug)]
pub(crate) struct AuthRuntimePaths {
    setup: PathBuf,
    allowed_clients: PathBuf,
    client_forbid: PathBuf,
}

impl AuthRuntimePaths {
    /// Разрешает исходные имена относительно рабочего каталога AuthServer.
    pub(crate) fn from_runtime_directory(directory: impl AsRef<Path>) -> Self {
        let directory = directory.as_ref();
        Self {
            setup: directory.join("setup.ini"),
            allowed_clients: directory.join("allowed_ls.ini"),
            client_forbid: directory.join("client_forbid_ip.ini"),
        }
    }
}

/// Нефатальный итог подготовки Auth listener до внешнего `Init`.
#[derive(Debug)]
pub(crate) struct AuthNetworkInitialization {
    /// Ошибка `allowed_ls.ini`, которую исходный Host не считал фатальной.
    pub(crate) allowed_clients_error: Option<io::Error>,
}

struct LegacyTickClock;

impl LegacyTickClock {
    const fn new() -> Self {
        Self
    }

    fn now_ms(&self) -> u32 {
        let now = clock_gettime(ClockId::Boottime);
        let seconds_ms = (now.tv_sec as u64).wrapping_mul(1000);
        let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
        seconds_ms.wrapping_add(nanoseconds_ms) as u32
    }
}

/// Разделяемое между `CGame` и Auth DB workers состояние исходных очередей.
#[derive(Clone)]
pub(crate) struct AuthDbContext {
    config: Arc<RwLock<ConfigReader>>,
    quests: Arc<MultiList<DbQuest>>,
    results: Arc<MultiList<DbResult>>,
    server_info: Arc<ServerInfoQueue>,
    client_ip_forbider: Arc<RwLock<IpFilter<false>>>,
}

impl AuthDbContext {
    fn new(config: ConfigReader) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            quests: Arc::new(MultiList::new()),
            results: Arc::new(MultiList::new()),
            server_info: Arc::new(ServerInfoQueue::new()),
            client_ip_forbider: Arc::new(RwLock::new(IpFilter::new())),
        }
    }

    /// Читает один согласованный config snapshot без копирования секретов.
    pub(crate) fn config(&self) -> RwLockReadGuard<'_, ConfigReader> {
        self.config.read()
    }

    /// Публикует DB-команду либо исходный немедленный ответ о переполнении.
    pub(crate) fn push_quest(&self, quest: DbQuest) -> bool {
        if self.quests.size() < self.config.read().max_auth_queue_size() as u32 {
            self.quests.push_back(quest);
            return true;
        }

        let rejection = match quest {
            DbQuest::Authenticate {
                return_socket_id,
                request,
            } => Some(DbResult::Authenticate {
                return_socket_id,
                result: AuthResultData::new(
                    6,
                    request.account,
                    request.client_ip,
                    request.client_socket_id,
                ),
            }),
            DbQuest::AuthenticateExtended {
                return_socket_id,
                request,
            } => Some(DbResult::AuthenticateExtended {
                return_socket_id,
                result: AuthExResultData::new(
                    6,
                    request.account,
                    request.client_ip,
                    request.client_socket_id,
                ),
            }),
            DbQuest::Lock {
                return_socket_id,
                request,
            } => Some(DbResult::Lock {
                return_socket_id,
                result: LockResultData {
                    account: request.account,
                    succeeded: false,
                },
            }),
            DbQuest::WriteServerInfo => None,
        };

        if let Some(rejection) = rejection {
            self.results.push_back(rejection);
        }
        false
    }

    /// Возвращает отдельный snapshot числа ожидающих DB-команд.
    pub(crate) fn quest_count(&self) -> u32 {
        self.quests.size()
    }

    /// Ждёт старейшую команду либо прерывает пустое ожидание при shutdown.
    pub(crate) fn pop_quest_until_stopped(&self, stopped: &AtomicBool) -> Option<DbQuest> {
        self.quests.pop_front_wait_until_stopped(stopped)
    }

    /// Будит DB workers после публикации stop-флага.
    pub(crate) fn wake_quest_waiters(&self) {
        self.quests.wake_all();
    }

    /// Передаёт готовый результат главному game-loop.
    pub(crate) fn push_result(&self, result: DbResult) {
        self.results.push_back(result);
    }

    /// Применяет coalescing-обновление одного server-info tuple.
    pub(crate) fn push_server_info(&self, info: ServerInfo) {
        self.server_info.push_back(info);
    }

    /// Атомарно передаёт DB writer весь coalesced server-info snapshot.
    pub(crate) fn pop_all_server_info(&self) -> std::collections::VecDeque<ServerInfo> {
        self.server_info.pop_all()
    }

    /// Применяет текущую config-политику к уже разобранному DWORD IPv4.
    pub(crate) fn is_client_ip_allowed(&self, address: u32) -> bool {
        !self.config.read().client_ip_filter_enabled()
            || self
                .client_ip_forbider
                .read()
                .is_allowed(address.to_le_bytes())
    }
}

/// Частично восстановленный `CGame`, владеющий config, Auth network и handlers.
pub(crate) struct CGame<Handler> {
    db: AuthDbContext,
    db_workers: AuthDatabaseWorkers,
    net_server_auth: Option<CMyNetServerAuth>,
    message_handler: Handler,
    accept_task: Option<JoinHandle<io::Result<(TcpStream, SocketAddrV4)>>>,
    io_tasks: JoinSet<ServerIoCompletion>,
    next_accept_at: Instant,
    clock: LegacyTickClock,
}

impl<Handler> CGame<Handler>
where
    Handler: AuthMessageHandler,
{
    /// Создаёт `CGame` без сети, как исходный constructor с null server pointer.
    pub(crate) fn new(config: ConfigReader, message_handler: Handler) -> Self {
        Self::from_context(AuthDbContext::new(config), message_handler)
    }

    fn from_context(db: AuthDbContext, message_handler: Handler) -> Self {
        let clock = LegacyTickClock::new();
        Self {
            db,
            db_workers: AuthDatabaseWorkers::new(),
            net_server_auth: None,
            message_handler,
            accept_task: None,
            io_tasks: JoinSet::new(),
            next_accept_at: Instant::now(),
            clock,
        }
    }

    /// Возвращает конкретное доменное состояние message-handler’ов.
    pub(crate) const fn message_handler(&self) -> &Handler {
        &self.message_handler
    }

    /// Возвращает mutable-доступ для config reload и извлечения событий.
    pub(crate) fn message_handler_mut(&mut self) -> &mut Handler {
        &mut self.message_handler
    }

    /// Возвращает сохранённый снимок исходного `mConfiger`.
    pub(crate) fn config(&self) -> RwLockReadGuard<'_, ConfigReader> {
        self.db.config()
    }

    /// Передаёт DB-очереди команду либо создаёт исходный немедленный отказ
    /// при достижении `_max_auth_quest`.
    ///
    /// Signed предел намеренно сравнивается после приведения к `uint`, как в
    /// RVA `0x00005EC0`: отрицательная malformed-конфигурация превращалась в
    /// большой unsigned предел. Для server-info команды переполнение не
    /// создаёт результата.
    pub(crate) fn push_db_quest(&self, quest: DbQuest) -> bool {
        self.db.push_quest(quest)
    }

    /// Возвращает отдельный snapshot числа DB-команд для исходного worker-loop.
    pub(crate) fn db_quest_count(&self) -> u32 {
        self.db.quest_count()
    }

    /// Ждёт и передаёт DB worker’у старейшую команду.
    pub(crate) fn pop_db_quest_wait(&self) -> DbQuest {
        self.db.quests.pop_front_wait()
    }

    /// Передаёт главному game-loop результат DB worker’а.
    pub(crate) fn push_db_result(&self, result: DbResult) {
        self.db.push_result(result);
    }

    /// Возвращает отдельный snapshot числа готовых DB-результатов.
    pub(crate) fn db_result_count(&self) -> u32 {
        self.db.results.size()
    }

    /// Ждёт и передаёт game-loop старейший DB-результат.
    pub(crate) fn pop_db_result_wait(&self) -> DbResult {
        self.db.results.pop_front_wait()
    }

    /// Применяет исходное coalescing-обновление одного server-info tuple.
    pub(crate) fn push_server_info(&self, info: ServerInfo) {
        self.db.push_server_info(info);
    }

    /// Передаёт DB writer’у весь текущий coalesced server-info снимок.
    pub(crate) fn pop_all_server_info(&self) -> std::collections::VecDeque<ServerInfo> {
        self.db.pop_all_server_info()
    }

    /// Заменяет уже разобранный deny-list клиентских IPv4 перед запуском DB.
    pub(crate) fn replace_client_forbid_patterns(&mut self, patterns: Vec<[u8; 4]>) {
        self.db
            .client_ip_forbider
            .write()
            .replace_patterns(patterns);
    }

    /// Повторяет обе перегрузки исходного `CheckClientIP` для DWORD IPv4.
    pub(crate) fn is_client_ip_allowed(&self, address: u32) -> bool {
        self.db.is_client_ip_allowed(address)
    }

    /// Запускает настроенное число owned Auth DB workers.
    pub(crate) fn start_auth_database_workers(
        &mut self,
    ) -> Result<(), AuthDatabaseWorkerStartError> {
        let worker_count = self.db.config().database_thread_count();
        self.db_workers.start(self.db.clone(), worker_count)
    }

    /// Публикует stop, будит ожидающие очереди и присоединяет все DB workers.
    pub(crate) fn release_auth_database_workers(&mut self) -> Vec<AuthDatabaseWorkerJoinError> {
        self.db_workers.stop_and_join()
    }

    /// Забирает следующее структурированное событие Auth DB workers.
    pub(crate) fn pop_auth_database_notice(&self) -> Option<AuthDatabaseNotice> {
        self.db_workers.pop_notice()
    }

    /// Пересоздаёт Auth listener и применяет доказанные runtime-лимиты.
    ///
    /// Старые задачи сначала отменяются и полностью присоединяются. Ошибка
    /// `Host` сохраняет созданный, но не слушающий server-owner, как старый
    /// `InitNetServer_Auth` сохранял выделенный pointer до `Release`.
    pub(crate) async fn init_auth_network(
        &mut self,
        config: AuthNetworkConfig,
        allowed_clients_path: impl AsRef<Path>,
    ) -> Result<AuthNetworkInitialization, AuthRuntimeError> {
        self.release_auth_network().await?;

        let mut server = CMyNetServerAuth::new(self.clock.now_ms());
        let allowed_clients_error = server.load_allowed_clients(allowed_clients_path).err();
        let host_result = server.host(config.host_port, None, DEFAULT_SOCKET_TYPE, true);
        if host_result.is_ok() {
            let local_ipv4_word = get_local_ipv4_word();
            let local_ip = get_local_ip();
            server.set_local_identity(&local_ip, local_ipv4_word);
            server.configure_limits(
                config.max_login_servers,
                config.max_in_flight_sends,
                config.permitted_send_bytes,
                config.new_accept_timeout_ms,
            );
        }
        self.net_server_auth = Some(server);
        self.next_accept_at = Instant::now();
        host_result
            .map(|()| AuthNetworkInitialization {
                allowed_clients_error,
            })
            .map_err(AuthRuntimeError::Host)
    }

    /// Возвращает текущий dotted IPv4 созданного network-owner.
    ///
    /// После ошибки `Host` это constructor default; успешный init заменяет его
    /// результатом отдельного строкового hostname resolution.
    pub(crate) fn auth_local_ip(&self) -> Option<&[u8]> {
        self.net_server_auth
            .as_ref()
            .map(CMyNetServerAuth::local_ip)
    }

    /// Возвращает текущее DWORD-представление IPv4 созданного network-owner.
    pub(crate) fn auth_local_ipv4_word(&self) -> Option<u32> {
        self.net_server_auth
            .as_ref()
            .map(CMyNetServerAuth::local_ipv4_word)
    }

    /// Выполняет один message snapshot исходного `CGame::ProcessMessage`.
    pub(crate) fn process_message(&mut self) -> Result<i32, DispatchError> {
        let (network, handler) = (&self.net_server_auth, &mut self.message_handler);
        let Some(network) = network else {
            return Ok(0);
        };
        let sender = network.command_handle();

        let mut remaining = network.pending_messages();
        let mut processed = 0_i32;
        while remaining > 0 {
            if let Some(mut message) = network.pop_received_message() {
                message.run(handler, &sender)?;
                processed = processed.wrapping_add(1);
            }
            remaining = remaining.wrapping_sub(1);
        }
        Ok(processed)
    }

    /// Выполняет один snapshot исходного `CGame::ProcessDBResult`.
    ///
    /// Размер читается один раз: ответы, опубликованные DB workers во время
    /// прохода, остаются следующему turn. Типизированный enum устраняет только
    /// невозможное в Rust неизвестное integer-значение старого `_type`.
    pub(crate) fn process_database_results(&mut self) -> Result<i32, AuthRuntimeError> {
        let sender = self
            .net_server_auth
            .as_ref()
            .ok_or(AuthRuntimeError::NetworkNotInitialized)?
            .command_handle();

        let mut remaining = self.db.results.size();
        let mut processed = 0_i32;
        while remaining > 0 {
            let result = self.db.results.pop_front_wait();
            match result {
                DbResult::Authenticate {
                    return_socket_id,
                    result,
                } => send_auth_result(&sender, return_socket_id, result)?,
                DbResult::AuthenticateExtended {
                    return_socket_id,
                    result,
                } => send_auth_extended_result(&sender, return_socket_id, result)?,
                DbResult::Lock {
                    return_socket_id,
                    result,
                } => send_lock_result(&sender, return_socket_id, result)?,
            }
            processed = processed.wrapping_add(1);
            remaining = remaining.wrapping_sub(1);
        }
        Ok(processed)
    }

    /// Выполняет один общий net/message turn до доменной части main-loop.
    ///
    /// Нормальные accept/I/O ошибки возвращаются в [`AuthRuntimeStep`]; panic
    /// управляемой задачи и неизвестный message handler останавливают turn.
    pub(crate) async fn run_network_turn(&mut self) -> Result<AuthRuntimeStep, AuthRuntimeError> {
        if self.net_server_auth.is_none() {
            return Err(AuthRuntimeError::NetworkNotInitialized);
        }

        let now_ms = self.clock.now_ms();
        let mut step = AuthRuntimeStep::default();
        self.drain_completed_tasks(now_ms, &mut step).await?;
        self.start_accept_if_due()?;

        let (snapshot, commands) = {
            let network = self
                .net_server_auth
                .as_mut()
                .ok_or(AuthRuntimeError::NetworkNotInitialized)?;
            let snapshot = network.process_network_snapshot(now_ms);
            (snapshot, network.command_handle())
        };
        step.processed_network_commands = snapshot.processed_commands();
        let (io_actions, network_errors) = snapshot.into_parts();
        step.network_errors = network_errors;
        self.spawn_io_actions(io_actions, commands);

        step.processed_messages = self.process_message().map_err(AuthRuntimeError::Dispatch)?;
        Ok(step)
    }

    /// Повторяет порядок `ExitWorkerThread` до уничтожения Auth network.
    pub(crate) async fn release_auth_network(&mut self) -> Result<(), AuthRuntimeError> {
        let mut task_error = None;
        if let Some(accept_task) = self.accept_task.take() {
            accept_task.abort();
            match accept_task.await {
                Ok(result) => drop(result),
                Err(error) if error.is_cancelled() => {}
                Err(error) => task_error = Some(error),
            }
        }

        if let Some(network) = &self.net_server_auth {
            network.command_handle().quit_all();
        }
        while self
            .net_server_auth
            .as_ref()
            .is_some_and(CMyNetServerAuth::has_clients)
        {
            let now_ms = self.clock.now_ms();
            let (snapshot, commands) = {
                let network = self
                    .net_server_auth
                    .as_mut()
                    .ok_or(AuthRuntimeError::NetworkNotInitialized)?;
                let snapshot = network.process_network_snapshot(now_ms);
                (snapshot, network.command_handle())
            };
            let (actions, _network_errors) = snapshot.into_parts();
            self.spawn_io_actions(actions, commands);
            while let Some(completion) = self.io_tasks.try_join_next() {
                if let Err(error) = completion {
                    task_error.get_or_insert(error);
                }
            }
            time::sleep(ACCEPT_THREAD_DELAY).await;
        }

        self.io_tasks.shutdown().await;
        self.net_server_auth.take();
        task_error.map(AuthRuntimeError::Task).map_or(Ok(()), Err)
    }

    /// Выполняет полный обратный порядок исходного `CGame::Release`.
    ///
    /// Метод всегда пытается закончить обе стадии. Panic отдельного DB worker’а
    /// и ошибка managed network-задачи возвращаются вместе, а не прерывают
    /// освобождение следующего владельца.
    pub(crate) async fn release_auth_runtime(&mut self) -> AuthReleaseReport {
        let database_worker_errors = self.release_auth_database_workers();
        let network_error = self.release_auth_network().await.err();
        AuthReleaseReport {
            database_worker_errors,
            network_error,
        }
    }

    /// Возвращает число accept/read/send задач под управлением этого `CGame`.
    pub(crate) fn active_network_tasks(&self) -> usize {
        usize::from(self.accept_task.is_some()) + self.io_tasks.len()
    }

    async fn drain_completed_tasks(
        &mut self,
        now_ms: u32,
        step: &mut AuthRuntimeStep,
    ) -> Result<(), AuthRuntimeError> {
        if self
            .accept_task
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            let result = self
                .accept_task
                .take()
                .expect("завершившаяся accept-задача уже проверена")
                .await
                .map_err(AuthRuntimeError::Task)?;
            self.next_accept_at = Instant::now() + ACCEPT_THREAD_DELAY;
            match result {
                Ok((stream, peer)) => {
                    let network = self
                        .net_server_auth
                        .as_mut()
                        .ok_or(AuthRuntimeError::NetworkNotInitialized)?;
                    step.admissions
                        .push(network.queue_accepted(stream, peer, now_ms));
                }
                Err(error) => step.accept_errors.push(error),
            }
        }
        self.drain_io_completions(Some(&mut step.io_completions))
    }

    fn start_accept_if_due(&mut self) -> Result<(), AuthRuntimeError> {
        if self.accept_task.is_some() || Instant::now() < self.next_accept_at {
            return Ok(());
        }

        let network = self
            .net_server_auth
            .as_ref()
            .ok_or(AuthRuntimeError::NetworkNotInitialized)?;
        match network.begin_accept() {
            AcceptStart::NotListening => Err(AuthRuntimeError::NetworkNotInitialized),
            AcceptStart::AtCapacity => {
                self.next_accept_at = Instant::now() + ACCEPT_AT_CAPACITY_DELAY;
                Ok(())
            }
            AcceptStart::Pending(accept) => {
                self.accept_task = Some(tokio::spawn(async move { accept.accept().await }));
                Ok(())
            }
        }
    }

    fn spawn_io_actions(
        &mut self,
        actions: Vec<ServerIoAction>,
        commands: crate::nets::servers::ServerCommandHandle,
    ) {
        for action in actions {
            let commands = commands.clone();
            self.io_tasks
                .spawn(async move { action.run(commands).await });
        }
    }

    fn drain_io_completions(
        &mut self,
        mut output: Option<&mut Vec<ServerIoCompletion>>,
    ) -> Result<(), AuthRuntimeError> {
        while let Some(completion) = self.io_tasks.try_join_next() {
            let completion = completion.map_err(AuthRuntimeError::Task)?;
            if let Some(output) = &mut output {
                output.push(completion);
            }
        }
        Ok(())
    }
}

impl CGame<AuthMessageHandlers> {
    /// Связывает один config snapshot с конкретным Auth handler-state.
    pub(crate) fn configured(config: ConfigReader, allowed_patterns: Vec<[u8; 4]>) -> Self {
        let context = AuthDbContext::new(config);
        let handlers = {
            let config = context.config();
            config.auth_message_handlers(allowed_patterns, context.clone())
        };
        Self::from_context(context, handlers)
    }

    /// Выполняет внешний порядок исходного `CGame::Init` над готовыми owners.
    ///
    /// Ошибка `setup.ini` применяет полный `reset` и остаётся нефатальным
    /// событием. Ошибки `allowed_ls.ini` и `client_forbid_ip.ini` также не
    /// отменяют запуск. После фатальной ошибки вызывающий всё равно обязан
    /// выполнить [`Self::release_auth_runtime`]: оригинальный `GameThreadFunc`
    /// вызывал `Release` и после частично неуспешного `Init`.
    pub(crate) async fn initialize_auth_runtime(
        &mut self,
        setup_path: impl AsRef<Path>,
        allowed_clients_path: impl AsRef<Path>,
        client_forbid_path: impl AsRef<Path>,
    ) -> Result<AuthInitializationReport, AuthInitializationError> {
        let mut notices = Vec::new();
        let setup_error = {
            let mut config = self.db.config.write();
            match config.load(setup_path) {
                Ok(()) => None,
                Err(error) => {
                    config.reset();
                    Some(error)
                }
            }
        };
        if let Some(error) = setup_error {
            notices.push(AuthInitializationNotice::SetupDefaultsApplied(error));
        }

        // CheckOneInstance был GUI-поиском окна по тому же port. На Linux его
        // процессный запрет обеспечивает следующий эксклюзивный listener bind.
        // Message pool, scratch-buffers и WinSock startup уже выражены static
        // dispatch, локальным владением и Tokio и не требуют пустых вызовов.
        let network_config = self.db.config().auth_network_config();
        let network = self
            .init_auth_network(network_config, allowed_clients_path)
            .await
            .map_err(AuthInitializationError::Network)?;
        if let Some(error) = network.allowed_clients_error {
            notices.push(AuthInitializationNotice::AllowedClientsUnavailable(error));
        }

        self.start_auth_database_workers()
            .map_err(AuthInitializationError::DatabaseWorkers)?;

        // `ConfigReader::load` этой точной сборки всегда отключает общий
        // ipfilter.ini; пустой список сохраняет фактически достижимый Init.
        self.db
            .config
            .read()
            .apply_login_server_filter(&mut self.message_handler, Vec::new());

        self.replace_client_forbid_patterns(Vec::new());
        if self.db.config().client_ip_filter_enabled() {
            match load_ip_patterns(client_forbid_path) {
                Ok(patterns) => self.replace_client_forbid_patterns(patterns),
                Err(error) => {
                    notices.push(AuthInitializationNotice::ClientIpFilterUnavailable(error))
                }
            }
        }

        let update_server_info_enabled = self.db.config().update_server_info_enabled();
        Ok(AuthInitializationReport {
            notices,
            update_server_info_enabled,
        })
    }

    /// Выполняет один полный проход исходного `CGame::MainLoop`.
    ///
    /// После технического network snapshot порядок доменного владельца остаётся
    /// буквальным: message snapshot, DB-result snapshot, условный server-info и
    /// пауза `1 ms`. Новые элементы обеих очередей не втягиваются в уже начатый
    /// snapshot.
    pub(crate) async fn run_main_loop_turn(&mut self) -> Result<AuthRuntimeStep, AuthRuntimeError> {
        let mut step = self.run_network_turn().await?;
        step.processed_database_results = self.process_database_results()?;

        let update_enabled = self.db.config().update_server_info_enabled();
        if update_enabled {
            let server_info = self.update_server_info()?;
            step.server_info_requested = server_info.requested;
            step.server_info_write_due = server_info.write_due;
            step.server_info_write_queued = server_info.write_queued;
        }

        time::sleep(NET_THREAD_DELAY).await;
        Ok(step)
    }

    fn update_server_info(&self) -> Result<ServerInfoUpdate, AuthRuntimeError> {
        let (update_interval_ms, write_interval_ms) = {
            let config = self.db.config();
            (
                config.update_server_info_time_ms(),
                config.write_server_info_time_ms(),
            )
        };
        let mut timers = SERVER_INFO_TIMERS.lock();
        if timers.last_update_ms.is_none() {
            timers.last_update_ms = Some(self.clock.now_ms());
        }
        if timers.last_write_ms.is_none() {
            timers.last_write_ms = Some(self.clock.now_ms());
        }
        let now_ms = self.clock.now_ms();

        let requested = timers
            .last_update_ms
            .is_some_and(|last| last.wrapping_add(update_interval_ms) < now_ms);
        if requested {
            let message = CMessage::new(0x000C_F702);
            self.send_to_registered_login_servers(&message)?;
            timers.last_update_ms = Some(now_ms);
        }

        let write_due = timers
            .last_write_ms
            .is_some_and(|last| last.wrapping_add(write_interval_ms) < now_ms);
        let write_queued = write_due && self.push_db_quest(DbQuest::WriteServerInfo);
        if write_due {
            // Исходный timer обновлялся даже при отказе переполненной DB queue.
            timers.last_write_ms = Some(now_ms);
        }

        Ok(ServerInfoUpdate {
            requested,
            write_due,
            write_queued,
        })
    }

    fn send_to_registered_login_servers(&self, message: &CMessage) -> Result<(), AuthRuntimeError> {
        let sender = self
            .net_server_auth
            .as_ref()
            .ok_or(AuthRuntimeError::NetworkNotInitialized)?
            .command_handle();
        for socket_id in self.message_handler.login_server_socket_ids() {
            let _legacy_result = message.send_to_login(&sender, socket_id)?;
        }
        Ok(())
    }

    /// Открывает Auth network из фактически сохранённого `mConfiger`.
    pub(crate) async fn init_configured_auth_network(
        &mut self,
        allowed_clients_path: impl AsRef<Path>,
    ) -> Result<(), AuthRuntimeError> {
        let network_config = self.db.config().auth_network_config();
        self.init_auth_network(network_config, allowed_clients_path)
            .await
            .map(|_| ())
    }

    /// Повторяет config reload и обновляет handler filter только после успеха.
    pub(crate) fn reload_config_and_login_filter(
        &mut self,
        setup_path: impl AsRef<Path>,
        allowed_clients_path: impl AsRef<Path>,
        allowed_patterns: Vec<[u8; 4]>,
    ) -> Result<(), ConfigLoadError> {
        self.db.config.write().load(setup_path)?;
        self.db
            .config
            .read()
            .apply_login_server_filter(&mut self.message_handler, allowed_patterns);
        if let Some(network) = &mut self.net_server_auth {
            // Исходный ReloadSetup только писал FAILED в операторский лог, но
            // после успешного setup всё равно возвращал true.
            let _allowed_clients_result = network.load_allowed_clients(allowed_clients_path);
        }
        Ok(())
    }
}

fn resolve_first_local_ipv4() -> Option<Ipv4Addr> {
    let hostname = uname();
    let hostname = hostname.nodename().to_str().ok()?;
    (hostname, 0)
        .to_socket_addrs()
        .ok()?
        .find_map(|address| match address {
            SocketAddr::V4(address) => Some(*address.ip()),
            SocketAddr::V6(_) => None,
        })
}

fn get_local_ipv4_word() -> u32 {
    resolve_first_local_ipv4()
        .map(legacy_ipv4_word)
        .unwrap_or(0)
}

fn get_local_ip() -> Vec<u8> {
    resolve_first_local_ipv4()
        .map(|address| address.to_string().into_bytes())
        .unwrap_or_default()
}

fn send_auth_result(
    sender: &crate::nets::servers::ServerCommandHandle,
    return_socket_id: i32,
    result: AuthResultData,
) -> Result<(), SendMessageError> {
    let mut message = CMessage::new(0x000C_F601);
    let base = message.base_mut();
    base.add_long(result.result);
    add_legacy_string(base, &result.account);
    base.add_ulong(result.client_ip);
    base.add_long(result.client_socket_id);
    let _legacy_result = message.send_to_login(sender, return_socket_id)?;
    Ok(())
}

fn send_auth_extended_result(
    sender: &crate::nets::servers::ServerCommandHandle,
    return_socket_id: i32,
    result: AuthExResultData,
) -> Result<(), SendMessageError> {
    let mut message = CMessage::new(0x000C_F602);
    let base = message.base_mut();
    base.add_long(result.result);
    add_legacy_string(base, &result.account);
    base.add_ulong(result.client_ip);
    base.add_long(result.client_socket_id);
    if result.result == 3 {
        // Auth RVA 0x00004460 отправлял пять SYSTEMTIME-полей и буквально
        // пропускал day-of-week (+4) и seconds (+12).
        for offset in [0, 2, 6, 8, 10] {
            base.add_word(u16::from_le_bytes([
                result.extra[offset],
                result.extra[offset + 1],
            ]));
        }
    } else if result.result == 7 {
        base.add(&result.extra);
    }
    let _legacy_result = message.send_to_login(sender, return_socket_id)?;
    Ok(())
}

fn send_lock_result(
    sender: &crate::nets::servers::ServerCommandHandle,
    return_socket_id: i32,
    result: LockResultData,
) -> Result<(), SendMessageError> {
    let mut message = CMessage::new(0x0010_F202);
    let base = message.base_mut();
    add_legacy_string(base, &result.account);
    base.add_byte(u8::from(result.succeeded));
    let _legacy_result = message.send_to_login(sender, return_socket_id)?;
    Ok(())
}

fn add_legacy_string(message: &mut crate::nets::basemessage::CBaseMessage, value: &[u8]) {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    message.add(&value[..end]);
    message.add_byte(0);
}

/// Выполняет полный lifecycle исходного `GameThreadFunc` над одним Auth owner.
///
/// `shutdown` заменяет только Windows thread-message `0x464`. Инициализация не
/// прерывается сигналом, а release выполняется при любом результате `Init` и
/// после первой фатальной ошибки main-loop.
pub(crate) async fn game_thread_func<Shutdown>(
    paths: &AuthRuntimePaths,
    shutdown: Shutdown,
) -> AuthGameThreadReport
where
    Shutdown: Future<Output = ()>,
{
    // Локальное владение заменяет g_Game/CreateGame/GetGame/DeleteGame:
    // отдельный глобальный доступ не был частью сетевого или DB-контракта.
    let mut game = CGame::configured(ConfigReader::new(), Vec::new());
    let initialization = game
        .initialize_auth_runtime(&paths.setup, &paths.allowed_clients, &paths.client_forbid)
        .await;

    let runtime_error = if initialization.is_ok() {
        tokio::pin!(shutdown);
        loop {
            tokio::select! {
                biased;
                () = &mut shutdown => break None,
                result = game.run_main_loop_turn() => {
                    match result {
                        Ok(_step) => {}
                        Err(error) => break Some(error),
                    }
                }
            }
        }
    } else {
        None
    };

    let release = game.release_auth_runtime().await;
    drop(game);
    AuthGameThreadReport {
        initialization,
        runtime_error,
        release,
    }
}

impl<Handler> Drop for CGame<Handler> {
    fn drop(&mut self) {
        if let Some(accept_task) = self.accept_task.take() {
            accept_task.abort();
        }
        // JoinSet abort-ит все owned I/O-задачи своим Drop. Нормальный путь
        // вызывает async release_auth_network и дополнительно ждёт их выход.
    }
}

// COMPONENT_VARIANT_BEGIN: AuthServer
// Точная пара: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SHA-256 EXE: AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15
// SHA-256 PDB: 26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5
// Исходный владелец PDB: h:\fengyun\fy_russia\src\server\authserver\src\cgame.cpp
// Исходный владелец PDB: h:\fengyun\fy_russia\src\server\authserver\src\cgame.h

// ============================================================================
// FUNCTION: AddLogText
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\cgame.cpp:613
// RVA: 0x00002250
// ADDRESS: 00402250
// PROTOTYPE: void __cdecl AddLogText(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::CheckClientIP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\cgame.cpp:437
// RVA: 0x00005920
// ADDRESS: 00405920
// PROTOTYPE: bool __thiscall CheckClientIP(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::CheckClientIP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\cgame.cpp:421
// RVA: 0x00005A50
// ADDRESS: 00405a50
// PROTOTYPE: bool __thiscall CheckClientIP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::ReloadSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\server\authserver\src\cgame.cpp:132
// RVA: 0x00006740
// ADDRESS: 00406740
// PROTOTYPE: bool __thiscall ReloadSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: AuthServer
