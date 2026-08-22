//! Владелец `CGame` исторического BillingServer из `billingserver/game.cpp`.
//!
//! Восстановлены достигнутые PlayerFill-ветви, полные `Init/Release`, owned
//! network runtime и `GameThreadFunc`. Отдельная Windows-проверка единственного
//! экземпляра через `FindWindow` не переносится: тот же процессный инвариант
//! обеспечивает последующий exclusive listener bind в `InitServer`.
//!
//! Исходный путь PDB:
//!
//! `ProcessMessage` один раз читает размер FIFO `CServerForGS` и исполняет
//! ровно этот snapshot. Сообщения, добавленные после чтения размера, остаются
//! следующему turn. Каждое сообщение немедленно проходит уже восстановленный
//! `CMessage::Run`, который сам выбирает Billing либо server-handler; numeric
//! routing здесь не дублируется. Owned `CMessage` и `Drop` заменяют virtual
//! `Run` и последующий deleting destructor.
//!
//! Первый `MainLoop` дважды читает `timeGetTime`: первый tick инициализирует
//! три process-static cadence-слота, второй становится текущим. Следующие
//! проходы читают tick один раз. Refresh использует строгое
//! `interval < now - last`, выполняется до проверки `m_bExit` и при наличии
//! server-owner публикуется typed-событием вместо MFC `SetWindowText`.
//! Message snapshot идёт только без exit. Затем отдельно инициализируется ping
//! cadence; строгое `60000 < delta` один раз ставит `bInPing = true` без
//! придуманного сетевого ping, которого это тело не отправляло. Успешный turn
//! заканчивается точным `Sleep(1)`.
//!
//! `rustix::CLOCK_BOOTTIME` сохраняет suspend-aware 32-битный wrapping
//! `timeGetTime`; `VecDeque` хранит typed outcomes вместо Windows GUI-журнала.
//! Неиспользуемый этим телом `dwServerInfoLogLastTime` всё равно хранится и
//! инициализируется в исходной позиции. Для `ProcessMessage/MainLoop` повторный
//! reverse не требовался; отдельные setup/DB-факты ниже проверены адресно.
//!
//! `LoadSetup` читает пятнадцать positional-пар, игнорируя labels и сохраняя
//! partial mutation при преждевременном EOF. Конструктор `tagSetup` не попал в
//! owner-export, поэтому его точное тело проверено адресно: Billing EXE
//! задавая numeric/bool defaults. Rust хранит такие поля как `Option`; safe
//! init останавливается только на конкретной недоказанной границе вместо
//! придуманного значения. Найденный setup содержит все пятнадцать пар.
//!
//! `InitServer` удаляет прежний owner, сохраняет новый до `Host`, поэтому
//! ошибка bind оставляет неслушающий server как оригинал. Ошибка allow-list
//! после успешного Host не отменяет init; `rustix::uname + ToSocketAddrs`
//! заменяют `gethostname/gethostbyname` и сохраняют первый IPv4 одновременно
//! строкой и x86 DWORD. `InitBPManager` сначала выполняет нефатальный `Start`,
//! затем создаёт workers по одному; ошибка оставляет уже созданные элементы.
//! `ReleaseBPManager` останавливает/ждёт DB-workers по порядку и лишь затем
//! условно ждёт cash-log worker по текущему setup-флагу.
//! Optional PlayerFill получает те же основную DB-конфигурацию, sender и общий
//! `g_bGameThreadExit`; Start остаётся фатальным только при ошибке создания
//! thread, а Release при включённом setup-флаге только присоединяет worker.
//!
//! `Init` сохраняет порядок `LoadSetup -> CBaseMessage::Initial -> WinSock ->
//! InitServer -> InitBPManager -> optional PlayerFill`. Static message pool и
//! WinSock startup не имеют пустых Rust-вызовов: локальные message buffers и
//! Tokio sockets уже владеют их техническим эффектом. Network runtime держит
//! ровно одну accept-задачу и все read/send actions в `JoinSet`; каждый turn
//! применяет один атомарный `DoNetThreadFunc` snapshot перед исходным
//! `MainLoop`. `Release` сначала присоединяет Billing/PlayerFill workers, затем
//! при первом `m_bExit=false` выполняет `QUITALL`, snapshots до пустого map и
//! уничтожает server-owner. Повторный Release сохраняет исходный skip этой
//! последней стадии.
//!
//! Внешний `GameThreadFunc` владеет единственным локальным `CGame` вместо
//! `g_pGame/CreateGame/GetGame/DeleteGame`, всегда выполняет Release после
//! partial Init и принимает внешний shutdown future вместо Windows global,
//! event и `WM_CLOSE`. Перед cleanup он публикует тот же
//! `g_bGameThreadExit`, чтобы optional workers увидели уже наступившую границу.
//! Отдельный Linux entrypoint `src/bin/billingserver.rs` передаёт сюда
//! process shutdown и снимает наблюдаемые результаты каждого turn.

use std::collections::VecDeque;
use std::fs;
use std::future::Future;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

use rustix::system::uname;
use rustix::time::{ClockId, clock_gettime};
use tokio::net::TcpStream;
use tokio::task::{JoinError, JoinHandle, JoinSet};
use tokio::time::{self, Instant};

use crate::billingserver::appbilling::billingmessage::{
    BillingMessageHandler, BillingMessageOutcome,
};
use crate::billingserver::appbilling::billingplayermanager::{
    BillingPlayerManagerNotice, BillingPlayerManagerRuntime, CBillingPlayerManager,
    CreateBillingPlayerWorkerError,
};
use crate::billingserver::appbilling::playerfillmgr::{
    CPlayerFillMgr, PlayerFillNotice, PlayerFillRuntime, StartPlayerFillError,
};
use crate::billingserver::appbilling::servermessage::{ServerMessageHandler, ServerMessageOutcome};
use crate::dbaccess::dbbilling::rsplayeraccount::{
    BillingDatabaseSettings, BillingDatabaseSettingsParts,
};
use crate::dbaccess::dbbilling::rsplayerfillmgr::{
    PlayerFillDatabaseSettings, PlayerFillDatabaseSettingsParts,
};
use crate::nets::netbilling::clientforgs::BillingReceiveError;
use crate::nets::netbilling::message::{BillingMessageHandlers, CMessage};
use crate::nets::netbilling::serverforgs::CServerForGS;
use crate::nets::servers::{
    ACCEPT_AT_CAPACITY_DELAY, ACCEPT_THREAD_DELAY, AcceptStart, AdmissionOutcome, ServerHostError,
    ServerIoAction, ServerIoCompletion, ServerSnapshotError,
};

const PING_INTERVAL_MS: u32 = 60_000;
const MAIN_LOOP_SLEEP: Duration = Duration::from_millis(1);

#[derive(Default)]
struct MainLoopState {
    current_time_ms: Option<u32>,
    last_refresh_info_ms: Option<u32>,
    last_server_info_log_ms: Option<u32>,
    last_ping_ms: Option<u32>,
    in_ping: bool,
}

#[derive(Default)]
struct BillingSetup {
    gs_listen_port: Option<u32>,
    sql_server_ip: Vec<u8>,
    sql_user_name: Vec<u8>,
    sql_password: Vec<u8>,
    database_name: Vec<u8>,
    log_server_enabled: Option<bool>,
    log_server_ip: Vec<u8>,
    log_server_user_name: Vec<u8>,
    log_server_password: Vec<u8>,
    log_database_name: Vec<u8>,
    refresh_info_time_ms: Option<u32>,
    save_info_time_ms: Option<u32>,
    save_log_server_time_ms: Option<u32>,
    database_io_thread_count: Option<u32>,
    player_fill_check_enabled: Option<bool>,
}

/// Итог positional-чтения Billing `Setup.ini` без раскрытия значений.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BillingSetupLoadReport {
    pub(crate) parsed_pairs: usize,
    pub(crate) stopped_at_pair: Option<usize>,
}

/// Ошибка открытия исходного Billing setup.
#[derive(Debug)]
pub(crate) struct BillingSetupOpenError {
    pub(crate) path: PathBuf,
    pub(crate) source: io::Error,
}

/// Ошибка `InitServer` после уже сохранённого нового server-owner.
#[derive(Debug)]
pub(crate) enum BillingServerInitializationError {
    MissingSetupField(&'static str),
    Host(ServerHostError),
}

/// Нефатальный результат загрузки `GSInfoSetup.ini` после успешного `Host`.
#[derive(Debug)]
pub(crate) struct BillingServerInitialization {
    pub(crate) allowed_clients_error: Option<io::Error>,
}

/// Ошибка точного `InitBPManager` после возможного запуска ранних workers.
#[derive(Debug)]
pub(crate) enum BillingPlayerManagerInitializationError {
    MissingSetupField(&'static str),
    MissingGameServer,
    CreateWorker(CreateBillingPlayerWorkerError),
}

/// Ошибка optional PlayerFill-ветви `CGame::Init/Release`.
#[derive(Debug)]
pub(crate) enum BillingPlayerFillInitializationError {
    MissingSetupField(&'static str),
    MissingGameServer,
    Start(StartPlayerFillError),
}

/// Результат одного конкретного Billing/server-handler.
#[derive(Debug)]
pub(crate) enum BillingGameMessageOutcome {
    Billing(BillingMessageOutcome),
    Server(ServerMessageOutcome),
}

/// Наблюдаемая позиция MFC/ping-состояния одного `MainLoop`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BillingGameEvent {
    RefreshInfo,
    PingStarted,
}

/// Ошибка безопасной границы при непрочитанном numeric-поле setup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BillingMainLoopError {
    MissingRefreshInfoTime,
}

/// Пути двух файлов, буквально читаемых исходным `CGame::Init`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BillingRuntimePaths {
    pub(crate) setup: PathBuf,
    pub(crate) allowed_game_servers: PathBuf,
}

impl BillingRuntimePaths {
    /// Восстанавливает исходные имена относительно runtime-каталога.
    pub(crate) fn from_runtime_directory(directory: impl AsRef<Path>) -> Self {
        let directory = directory.as_ref();
        Self {
            setup: directory.join("Setup.ini"),
            allowed_game_servers: directory.join("GSInfoSetup.ini"),
        }
    }
}

/// Нефатальное событие полного Billing `Init`.
#[derive(Debug)]
pub(crate) enum BillingInitializationNotice {
    AllowedGameServersUnavailable(io::Error),
}

/// Фатальная стадия буквального Billing `CGame::Init`.
#[derive(Debug)]
pub(crate) enum BillingInitializationError {
    Setup(BillingSetupOpenError),
    Server(BillingServerInitializationError),
    BillingPlayerManager(BillingPlayerManagerInitializationError),
    PlayerFill(BillingPlayerFillInitializationError),
}

/// Полный успешный итог Billing `CGame::Init`.
#[derive(Debug)]
pub(crate) struct BillingInitializationReport {
    pub(crate) setup: BillingSetupLoadReport,
    pub(crate) notices: Vec<BillingInitializationNotice>,
}

/// Ошибка owned accept/read/send runtime.
#[derive(Debug)]
pub(crate) enum BillingNetworkRuntimeError {
    MissingGameServer,
    Task(JoinError),
}

/// Один concurrent network snapshot перед доменным `MainLoop`.
#[derive(Debug, Default)]
pub(crate) struct BillingNetworkTurn {
    pub(crate) admissions: Vec<AdmissionOutcome>,
    pub(crate) accept_errors: Vec<io::Error>,
    pub(crate) io_completions: Vec<ServerIoCompletion>,
    pub(crate) processed_commands: i32,
    pub(crate) snapshot_errors: Vec<ServerSnapshotError<BillingReceiveError>>,
}

/// Полный наблюдаемый результат одного Billing runtime-turn.
#[derive(Debug, Default)]
pub(crate) struct BillingRuntimeStep {
    pub(crate) network: BillingNetworkTurn,
    pub(crate) keep_running: bool,
    pub(crate) message_outcomes: Vec<BillingGameMessageOutcome>,
    pub(crate) events: Vec<BillingGameEvent>,
    pub(crate) billing_player_notices: Vec<BillingPlayerManagerNotice>,
    pub(crate) player_fill_notices: Vec<PlayerFillNotice>,
}

/// Фатальная граница одного полного runtime-turn.
#[derive(Debug)]
pub(crate) enum BillingRuntimeError {
    Network(BillingNetworkRuntimeError),
    MainLoop(BillingMainLoopError),
}

/// Отчёт буквального `CGame::Release`, который в EXE всегда возвращал true.
#[derive(Debug)]
pub(crate) struct BillingReleaseReport {
    pub(crate) legacy_result: bool,
    pub(crate) network_cleanup_performed: bool,
    pub(crate) billing_player_manager_error: Option<BillingPlayerManagerInitializationError>,
    pub(crate) player_fill_error: Option<BillingPlayerFillInitializationError>,
    pub(crate) network_task_errors: Vec<JoinError>,
}

/// Итог единственного owned аналога исходного `GameThreadFunc`.
#[derive(Debug)]
pub(crate) struct BillingGameThreadReport {
    pub(crate) initialization: Result<BillingInitializationReport, BillingInitializationError>,
    pub(crate) completed_turns: u64,
    pub(crate) runtime_error: Option<BillingRuntimeError>,
    pub(crate) release: BillingReleaseReport,
}

/// Достигнутая часть исходного `CGame`; остальные поля добавляются их owners.
pub(crate) struct CGame {
    gs_server: Option<CServerForGS>,
    billing_players: Arc<CBillingPlayerManager>,
    billing_player_runtime: Option<BillingPlayerManagerRuntime>,
    player_fill: Arc<CPlayerFillMgr>,
    player_fill_runtime: Option<PlayerFillRuntime>,
    accept_task: Option<JoinHandle<io::Result<(TcpStream, std::net::SocketAddrV4)>>>,
    io_tasks: JoinSet<ServerIoCompletion>,
    next_accept_at: Instant,
    setup: BillingSetup,
    log_server_enabled: Arc<AtomicBool>,
    save_log_server_time_ms: Arc<AtomicU32>,
    game_thread_exit: Arc<AtomicBool>,
    exit_requested: bool,
    main_loop_state: MainLoopState,
    message_outcomes: VecDeque<BillingGameMessageOutcome>,
    events: VecDeque<BillingGameEvent>,
}

impl CGame {
    /// Создаёт исходное null-server/non-exit состояние без запуска runtime.
    pub(crate) fn new() -> Self {
        Self {
            gs_server: None,
            billing_players: Arc::new(CBillingPlayerManager::new()),
            billing_player_runtime: None,
            player_fill: Arc::new(CPlayerFillMgr::default()),
            player_fill_runtime: None,
            accept_task: None,
            io_tasks: JoinSet::new(),
            next_accept_at: Instant::now(),
            setup: BillingSetup::default(),
            log_server_enabled: Arc::new(AtomicBool::new(false)),
            save_log_server_time_ms: Arc::new(AtomicU32::new(0)),
            game_thread_exit: Arc::new(AtomicBool::new(false)),
            exit_requested: false,
            main_loop_state: MainLoopState::default(),
            message_outcomes: VecDeque::new(),
            events: VecDeque::new(),
        }
    }

    /// Подключает уже созданный component-server в позиции `InitServer`.
    pub(crate) fn set_gs_server(&mut self, server: CServerForGS) {
        self.gs_server = Some(server);
    }

    /// Задаёт доказанное поле setup для cadence `MainLoop`.
    pub(crate) fn set_refresh_info_time(&mut self, interval_ms: u32) {
        self.setup.refresh_info_time_ms = Some(interval_ms);
    }

    /// Возвращает общие process-static Billing FIFO.
    pub(crate) fn billing_players(&self) -> &Arc<CBillingPlayerManager> {
        &self.billing_players
    }

    /// Ставит внутренний аналог `m_bExit`, который проверяется после refresh.
    pub(crate) fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    /// Публикует внешний `g_bGameThreadExit` для optional background workers.
    pub(crate) fn request_game_thread_exit(&self) {
        self.game_thread_exit.store(true, Ordering::Release);
    }

    /// Позиционно читает пятнадцать пар исходного `Setup.ini`.
    pub(crate) fn load_setup(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<BillingSetupLoadReport, BillingSetupOpenError> {
        let requested = path.as_ref();
        let path = resolve_legacy_ascii_case(requested).unwrap_or_else(|| requested.to_path_buf());
        let bytes = fs::read(&path).map_err(|source| BillingSetupOpenError {
            path: path.clone(),
            source,
        })?;
        let report = self.setup.parse_positional(&bytes);
        if let Some(enabled) = self.setup.log_server_enabled {
            self.log_server_enabled.store(enabled, Ordering::Release);
        }
        if let Some(interval) = self.setup.save_log_server_time_ms {
            self.save_log_server_time_ms
                .store(interval, Ordering::Release);
        }
        Ok(report)
    }

    /// Пересоздаёт Billing listener, затем нефатально читает `GSInfoSetup.ini`.
    pub(crate) fn init_server(
        &mut self,
        allowed_clients_path: impl AsRef<Path>,
    ) -> Result<BillingServerInitialization, BillingServerInitializationError> {
        self.gs_server = None;
        let mut server = CServerForGS::new(legacy_tick_ms());
        let port = self.setup.gs_listen_port.ok_or(
            BillingServerInitializationError::MissingSetupField("dwGSListenPort"),
        )?;
        let host_result = server.host(port, None, 1, true);
        if let Err(error) = host_result {
            self.gs_server = Some(server);
            return Err(BillingServerInitializationError::Host(error));
        }

        let requested = allowed_clients_path.as_ref();
        let allowed_path =
            resolve_legacy_ascii_case(requested).unwrap_or_else(|| requested.to_path_buf());
        let allowed_clients_error = server.load_allowed_clients(allowed_path).err();
        if let Some(address) = resolve_first_local_ipv4() {
            server.set_local_identity(
                address.to_string().as_bytes(),
                u32::from_le_bytes(address.octets()),
            );
        }
        self.gs_server = Some(server);
        Ok(BillingServerInitialization {
            allowed_clients_error,
        })
    }

    /// Выполняет `Start` и создаёт точное число элементов старого `vecBPM`.
    pub(crate) fn init_billing_player_manager(
        &mut self,
    ) -> Result<(), BillingPlayerManagerInitializationError> {
        let server = self
            .gs_server
            .as_ref()
            .ok_or(BillingPlayerManagerInitializationError::MissingGameServer)?;
        let log_enabled = required_setup(self.setup.log_server_enabled, "bLogSvrSwitch")?;
        let worker_count = required_setup(self.setup.database_io_thread_count, "dwDBIOThreadNum")?;
        if log_enabled {
            required_setup(self.setup.save_log_server_time_ms, "dwSaveLogSvrTime")?;
        }

        let settings = self.billing_database_settings();
        let mut runtime = BillingPlayerManagerRuntime::new(
            Arc::clone(&self.billing_players),
            server.command_handle(),
            settings,
            Arc::clone(&self.log_server_enabled),
            Arc::clone(&self.save_log_server_time_ms),
            Arc::clone(&self.game_thread_exit),
        );
        runtime.start(log_enabled);
        self.billing_player_runtime = Some(runtime);

        for _ in 0..worker_count {
            self.billing_player_runtime
                .as_mut()
                .expect("runtime сохранён до первого CreateThread")
                .create_thread()
                .map_err(BillingPlayerManagerInitializationError::CreateWorker)?;
        }
        Ok(())
    }

    /// Последовательно освобождает DB-workers и затем optional log-worker.
    pub(crate) fn release_billing_player_manager(
        &mut self,
    ) -> Result<bool, BillingPlayerManagerInitializationError> {
        let log_enabled = required_setup(self.setup.log_server_enabled, "bLogSvrSwitch")?;
        let Some(runtime) = &mut self.billing_player_runtime else {
            return Ok(true);
        };
        runtime.release();
        let result = runtime.end(log_enabled);
        self.billing_player_runtime = None;
        Ok(result)
    }

    /// Условно создаёт единственный PlayerFill worker после BP manager.
    pub(crate) fn init_player_fill_manager(
        &mut self,
    ) -> Result<bool, BillingPlayerFillInitializationError> {
        let enabled = self.setup.player_fill_check_enabled.ok_or(
            BillingPlayerFillInitializationError::MissingSetupField("bPlayerFillCheckSrvSwitch"),
        )?;
        if !enabled {
            return Ok(true);
        }
        let server = self
            .gs_server
            .as_ref()
            .ok_or(BillingPlayerFillInitializationError::MissingGameServer)?;
        let mut runtime = PlayerFillRuntime::new(
            Arc::clone(&self.player_fill),
            self.player_fill_database_settings(),
            self.billing_database_settings(),
            server.command_handle(),
            Arc::clone(&self.game_thread_exit),
        );
        runtime
            .start()
            .map_err(BillingPlayerFillInitializationError::Start)?;
        self.player_fill_runtime = Some(runtime);
        Ok(true)
    }

    /// Условно ждёт PlayerFill worker в исходной позиции `Release`.
    pub(crate) fn release_player_fill_manager(
        &mut self,
    ) -> Result<bool, BillingPlayerFillInitializationError> {
        let enabled = self.setup.player_fill_check_enabled.ok_or(
            BillingPlayerFillInitializationError::MissingSetupField("bPlayerFillCheckSrvSwitch"),
        )?;
        if enabled && let Some(mut runtime) = self.player_fill_runtime.take() {
            runtime.end();
        }
        Ok(true)
    }

    /// Выполняет полный порядок исходного `CGame::Init` над готовым owner.
    ///
    /// Ошибка allow-list остаётся нефатальной. После любой фатальной стадии
    /// вызывающий обязан всё равно выполнить [`Self::release`], потому что
    /// исходный `GameThreadFunc` освобождал и частично созданное состояние.
    pub(crate) fn initialize(
        &mut self,
        paths: &BillingRuntimePaths,
    ) -> Result<BillingInitializationReport, BillingInitializationError> {
        let setup = self
            .load_setup(&paths.setup)
            .map_err(BillingInitializationError::Setup)?;

        // `CBaseMessage::Initial` и `CMySocket::MySocketInit` создавали static
        // buffers/WinSock state. Их эффект уже принадлежит owned messages и
        // Tokio sockets, поэтому пустые lifecycle-вызовы здесь не вводятся.
        let server = self
            .init_server(&paths.allowed_game_servers)
            .map_err(BillingInitializationError::Server)?;
        let mut notices = Vec::new();
        if let Some(error) = server.allowed_clients_error {
            notices.push(BillingInitializationNotice::AllowedGameServersUnavailable(
                error,
            ));
        }
        self.init_billing_player_manager()
            .map_err(BillingInitializationError::BillingPlayerManager)?;
        self.init_player_fill_manager()
            .map_err(BillingInitializationError::PlayerFill)?;
        Ok(BillingInitializationReport { setup, notices })
    }

    /// Выполняет один concurrent accept/read/send snapshot Billing transport.
    pub(crate) async fn run_network_turn(
        &mut self,
    ) -> Result<BillingNetworkTurn, BillingNetworkRuntimeError> {
        if self.gs_server.is_none() {
            return Err(BillingNetworkRuntimeError::MissingGameServer);
        }

        let now_ms = legacy_tick_ms();
        let mut turn = BillingNetworkTurn::default();
        self.drain_completed_tasks(now_ms, &mut turn).await?;
        self.start_accept_if_due()?;

        let (snapshot, commands) = {
            let network = self
                .gs_server
                .as_mut()
                .ok_or(BillingNetworkRuntimeError::MissingGameServer)?;
            let snapshot = network.process_network_snapshot(now_ms);
            (snapshot, network.command_handle())
        };
        turn.processed_commands = snapshot.processed_commands();
        let (actions, errors) = snapshot.into_parts();
        turn.snapshot_errors = errors;
        self.spawn_io_actions(actions, commands);
        Ok(turn)
    }

    /// Выполняет технический network snapshot, затем буквальный `MainLoop`.
    pub(crate) async fn run_runtime_turn(
        &mut self,
    ) -> Result<BillingRuntimeStep, BillingRuntimeError> {
        let network = self
            .run_network_turn()
            .await
            .map_err(BillingRuntimeError::Network)?;
        let keep_running = self.main_loop().map_err(BillingRuntimeError::MainLoop)?;
        let mut step = BillingRuntimeStep {
            network,
            keep_running,
            ..BillingRuntimeStep::default()
        };
        self.drain_runtime_observations(&mut step);
        Ok(step)
    }

    fn drain_runtime_observations(&mut self, step: &mut BillingRuntimeStep) {
        while let Some(outcome) = self.pop_message_outcome() {
            step.message_outcomes.push(outcome);
        }
        while let Some(event) = self.pop_event() {
            step.events.push(event);
        }
        while let Some(notice) = self.billing_players.pop_notice() {
            step.billing_player_notices.push(notice);
        }
        while let Some(notice) = self.pop_player_fill_notice() {
            step.player_fill_notices.push(notice);
        }
    }

    /// Выполняет полный порядок исходного `CGame::Release`.
    ///
    /// Billing/PlayerFill workers присоединяются до проверки `m_bExit`.
    /// Повторный вызов, как EXE, пропускает network/socket/message cleanup.
    pub(crate) async fn release(&mut self) -> BillingReleaseReport {
        let billing_player_manager_error = self.release_billing_player_manager().err();
        let player_fill_error = self.release_player_fill_manager().err();
        let mut network_task_errors = Vec::new();
        let network_cleanup_performed = !self.exit_requested;

        if network_cleanup_performed {
            self.exit_requested = true;
            network_task_errors.extend(self.stop_accept_task().await);
            if let Some(network) = &self.gs_server {
                network.command_handle().quit_all();
            }
            while self
                .gs_server
                .as_ref()
                .is_some_and(CServerForGS::has_clients)
            {
                let (snapshot, commands) = {
                    let network = self
                        .gs_server
                        .as_mut()
                        .expect("Billing shutdown проверил наличие server-owner");
                    let snapshot = network.process_network_snapshot(legacy_tick_ms());
                    (snapshot, network.command_handle())
                };
                let (actions, _errors) = snapshot.into_parts();
                self.spawn_io_actions(actions, commands);
                self.collect_io_task_errors(&mut network_task_errors);
                time::sleep(ACCEPT_THREAD_DELAY).await;
            }
            self.io_tasks.shutdown().await;
            self.gs_server.take();
            // WinSock cleanup и CBaseMessage::Release уже выражены Drop
            // transport/message owners и не имеют отдельного Rust-состояния.
        }

        BillingReleaseReport {
            legacy_result: true,
            network_cleanup_performed,
            billing_player_manager_error,
            player_fill_error,
            network_task_errors,
        }
    }

    async fn drain_completed_tasks(
        &mut self,
        now_ms: u32,
        turn: &mut BillingNetworkTurn,
    ) -> Result<(), BillingNetworkRuntimeError> {
        if self
            .accept_task
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            let result = self
                .accept_task
                .take()
                .expect("завершившаяся Billing accept-задача уже проверена")
                .await
                .map_err(BillingNetworkRuntimeError::Task)?;
            self.next_accept_at = Instant::now() + ACCEPT_THREAD_DELAY;
            match result {
                Ok((stream, peer)) => {
                    let network = self
                        .gs_server
                        .as_mut()
                        .ok_or(BillingNetworkRuntimeError::MissingGameServer)?;
                    turn.admissions
                        .push(network.queue_accepted(stream, peer, now_ms));
                }
                Err(error) => turn.accept_errors.push(error),
            }
        }

        while let Some(completion) = self.io_tasks.try_join_next() {
            turn.io_completions
                .push(completion.map_err(BillingNetworkRuntimeError::Task)?);
        }
        Ok(())
    }

    fn start_accept_if_due(&mut self) -> Result<(), BillingNetworkRuntimeError> {
        if self.accept_task.is_some() || Instant::now() < self.next_accept_at {
            return Ok(());
        }
        let network = self
            .gs_server
            .as_ref()
            .ok_or(BillingNetworkRuntimeError::MissingGameServer)?;
        match network.begin_accept() {
            AcceptStart::NotListening => Err(BillingNetworkRuntimeError::MissingGameServer),
            AcceptStart::AtCapacity => {
                self.next_accept_at = Instant::now() + ACCEPT_AT_CAPACITY_DELAY;
                Ok(())
            }
            AcceptStart::Pending(accept) => {
                self.accept_task = Some(tokio::spawn(accept.accept()));
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

    async fn stop_accept_task(&mut self) -> Vec<JoinError> {
        let Some(task) = self.accept_task.take() else {
            return Vec::new();
        };
        task.abort();
        match task.await {
            Ok(result) => {
                drop(result);
                Vec::new()
            }
            Err(error) if error.is_cancelled() => Vec::new(),
            Err(error) => vec![error],
        }
    }

    fn collect_io_task_errors(&mut self, errors: &mut Vec<JoinError>) {
        while let Some(completion) = self.io_tasks.try_join_next() {
            if let Err(error) = completion {
                errors.push(error);
            }
        }
    }

    /// Забирает следующее typed-событие optional PlayerFill worker.
    pub(crate) fn pop_player_fill_notice(&self) -> Option<PlayerFillNotice> {
        self.player_fill.pop_notice()
    }

    /// Исполняет ровно начальный размер FIFO GameServer-сообщений.
    pub(crate) fn process_message(&mut self) -> bool {
        let mut remaining = self
            .gs_server
            .as_ref()
            .map_or(0, CServerForGS::pending_messages);
        while remaining > 0 {
            let outcome = {
                let server = self
                    .gs_server
                    .as_ref()
                    .expect("размер snapshot ненулевой только при server-owner");
                server.pop_received_message().and_then(|mut message| {
                    let mut handlers =
                        GameMessageHandlers::new(self.billing_players.as_ref(), Some(server));
                    message.run(&mut handlers);
                    handlers.outcome
                })
            };
            if let Some(outcome) = outcome {
                self.message_outcomes.push_back(outcome);
            }
            remaining = remaining.wrapping_sub(1);
        }
        true
    }

    /// Выполняет один точный доменный turn `MainLoop` без network polling.
    pub(crate) fn main_loop(&mut self) -> Result<bool, BillingMainLoopError> {
        let initial_now = match self.main_loop_state.current_time_ms {
            Some(now) => now,
            None => {
                let now = legacy_tick_ms();
                self.main_loop_state.current_time_ms = Some(now);
                now
            }
        };
        self.main_loop_state
            .last_refresh_info_ms
            .get_or_insert(initial_now);
        self.main_loop_state
            .last_server_info_log_ms
            .get_or_insert(initial_now);

        let now = legacy_tick_ms();
        self.main_loop_state.current_time_ms = Some(now);
        let refresh_interval = self
            .setup
            .refresh_info_time_ms
            .ok_or(BillingMainLoopError::MissingRefreshInfoTime)?;
        let last_refresh = self
            .main_loop_state
            .last_refresh_info_ms
            .expect("refresh cadence только что инициализирован");
        if refresh_interval < now.wrapping_sub(last_refresh) {
            self.main_loop_state.last_refresh_info_ms = Some(now);
            if self.gs_server.is_some() {
                self.events.push_back(BillingGameEvent::RefreshInfo);
            }
        }

        if self.exit_requested {
            return Ok(false);
        }

        self.process_message();
        let last_ping = self.main_loop_state.last_ping_ms.get_or_insert(now);
        if !self.main_loop_state.in_ping && PING_INTERVAL_MS < now.wrapping_sub(*last_ping) {
            self.main_loop_state.last_ping_ms = Some(now);
            self.main_loop_state.in_ping = true;
            self.events.push_back(BillingGameEvent::PingStarted);
        }
        thread::sleep(MAIN_LOOP_SLEEP);
        Ok(true)
    }

    /// Забирает следующий handler-outcome в порядке message snapshot.
    pub(crate) fn pop_message_outcome(&mut self) -> Option<BillingGameMessageOutcome> {
        self.message_outcomes.pop_front()
    }

    /// Забирает следующее typed-событие refresh/ping.
    pub(crate) fn pop_event(&mut self) -> Option<BillingGameEvent> {
        self.events.pop_front()
    }

    fn billing_database_settings(&self) -> BillingDatabaseSettings {
        BillingDatabaseSettings::from_parts(BillingDatabaseSettingsParts {
            account_host: self.setup.sql_server_ip.clone(),
            account_database: self.setup.database_name.clone(),
            account_user: self.setup.sql_user_name.clone(),
            account_password: self.setup.sql_password.clone(),
            cash_log_host: self.setup.log_server_ip.clone(),
            cash_log_database: self.setup.log_database_name.clone(),
            cash_log_user: self.setup.log_server_user_name.clone(),
            cash_log_password: self.setup.log_server_password.clone(),
        })
    }

    fn player_fill_database_settings(&self) -> PlayerFillDatabaseSettings {
        PlayerFillDatabaseSettings::from_parts(PlayerFillDatabaseSettingsParts {
            host: self.setup.sql_server_ip.clone(),
            database: self.setup.database_name.clone(),
            user: self.setup.sql_user_name.clone(),
            password: self.setup.sql_password.clone(),
        })
    }
}

impl Default for CGame {
    fn default() -> Self {
        Self::new()
    }
}

impl BillingSetup {
    fn parse_positional(&mut self, bytes: &[u8]) -> BillingSetupLoadReport {
        let mut tokens = SetupTokens::new(bytes);

        macro_rules! read_value {
            ($field:ident, $parser:expr) => {{
                let Some(raw) = tokens.next_value() else {
                    return tokens.report();
                };
                let Some(value) = $parser(raw) else {
                    // std::string. Malformed numeric token безопасно оставляет
                    // Option пустым и останавливает positional parsing; Init
                    // затем возвращает конкретный MissingSetupField.
                    return tokens.report();
                };
                self.$field = value;
                tokens.parsed();
            }};
        }

        macro_rules! read_number {
            ($field:ident, $type:ty) => {
                read_value!($field, |raw| parse_ascii::<$type>(raw).map(Some));
            };
        }

        macro_rules! read_bool {
            ($field:ident) => {
                read_value!($field, |raw| parse_legacy_bool(raw).map(Some));
            };
        }

        macro_rules! read_bytes {
            ($field:ident) => {
                read_value!($field, |raw: &[u8]| Some(raw.to_vec()));
            };
        }

        read_number!(gs_listen_port, u32);
        read_bytes!(sql_server_ip);
        read_bytes!(sql_user_name);
        read_bytes!(sql_password);
        read_bytes!(database_name);
        read_bool!(log_server_enabled);
        read_bytes!(log_server_ip);
        read_bytes!(log_server_user_name);
        read_bytes!(log_server_password);
        read_bytes!(log_database_name);
        read_number!(refresh_info_time_ms, u32);
        read_number!(save_info_time_ms, u32);
        read_number!(save_log_server_time_ms, u32);
        read_number!(database_io_thread_count, u32);
        read_bool!(player_fill_check_enabled);
        tokens.report()
    }
}

struct SetupTokens<'a> {
    tokens: Vec<&'a [u8]>,
    next: usize,
    attempted_pairs: usize,
    parsed_pairs: usize,
}

impl<'a> SetupTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            tokens: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
            attempted_pairs: 0,
            parsed_pairs: 0,
        }
    }

    fn next_value(&mut self) -> Option<&'a [u8]> {
        self.attempted_pairs += 1;
        let _label = self.tokens.get(self.next)?;
        let value = self.tokens.get(self.next + 1).copied()?;
        self.next += 2;
        Some(value)
    }

    fn parsed(&mut self) {
        self.parsed_pairs += 1;
    }

    fn report(&self) -> BillingSetupLoadReport {
        BillingSetupLoadReport {
            parsed_pairs: self.parsed_pairs,
            stopped_at_pair: (self.parsed_pairs < self.attempted_pairs)
                .then_some(self.attempted_pairs),
        }
    }
}

struct GameMessageHandlers<'a> {
    billing: BillingMessageHandler<'a>,
    server: ServerMessageHandler<'a>,
    outcome: Option<BillingGameMessageOutcome>,
}

impl<'a> GameMessageHandlers<'a> {
    fn new(queues: &'a CBillingPlayerManager, server: Option<&'a CServerForGS>) -> Self {
        Self {
            billing: BillingMessageHandler::new(queues),
            server: ServerMessageHandler::new(server),
            outcome: None,
        }
    }
}

impl BillingMessageHandlers for GameMessageHandlers<'_> {
    fn on_billing(&mut self, message: &mut CMessage) {
        self.outcome = Some(BillingGameMessageOutcome::Billing(
            self.billing.on_billing_message(message),
        ));
    }

    fn on_server(&mut self, message: &mut CMessage) {
        self.outcome = Some(BillingGameMessageOutcome::Server(
            self.server.on_server_message(message),
        ));
    }
}

fn legacy_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u64).wrapping_mul(1000);
    let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

fn required_setup<T: Copy>(
    value: Option<T>,
    name: &'static str,
) -> Result<T, BillingPlayerManagerInitializationError> {
    value.ok_or(BillingPlayerManagerInitializationError::MissingSetupField(
        name,
    ))
}

fn parse_ascii<T: FromStr>(raw: &[u8]) -> Option<T> {
    std::str::from_utf8(raw).ok()?.parse().ok()
}

fn parse_legacy_bool(raw: &[u8]) -> Option<bool> {
    match raw {
        b"0" => Some(false),
        b"1" => Some(true),
        _ => None,
    }
}

fn resolve_legacy_ascii_case(requested: &Path) -> Option<PathBuf> {
    if requested.is_file() {
        return Some(requested.to_path_buf());
    }
    let parent = requested.parent().unwrap_or_else(|| Path::new("."));
    let name = requested.file_name()?.to_str()?;
    fs::read_dir(parent).ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let entry_name = entry.file_name();
        let entry_name = entry_name.to_str()?;
        (entry_name.eq_ignore_ascii_case(name) && entry.file_type().ok()?.is_file())
            .then(|| entry.path())
    })
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

/// Выполняет полный lifecycle исходного `GameThreadFunc` над одним Billing owner.
///
/// `shutdown` заменяет только внешний `g_bGameThreadExit`. Локальное владение
/// заменяет process-global `g_pGame`; после partial Init и любой runtime-ошибки
/// всё равно исполняется буквальный Release.
pub(crate) async fn game_thread_func<Shutdown>(
    paths: &BillingRuntimePaths,
    shutdown: Shutdown,
    mut observe_step: impl FnMut(&BillingRuntimeStep),
) -> BillingGameThreadReport
where
    Shutdown: Future<Output = ()>,
{
    let mut game = CGame::new();
    let initialization = game.initialize(paths);
    let mut completed_turns = 0_u64;
    let runtime_error = if initialization.is_ok() {
        tokio::pin!(shutdown);
        loop {
            let outcome = tokio::select! {
                biased;
                () = &mut shutdown => break None,
                result = game.run_runtime_turn() => result,
            };
            match outcome {
                Ok(step) => {
                    completed_turns = completed_turns.wrapping_add(1);
                    let keep_running = step.keep_running;
                    observe_step(&step);
                    if !keep_running {
                        break None;
                    }
                }
                Err(error) => break Some(error),
            }
        }
    } else {
        None
    };

    // Нормальный внешний shutdown уже означает true в исходном global.
    // На partial Init/runtime error Rust также публикует его перед join:
    // иначе доказанные workers, читающие только этот flag, остались бы
    // detached навсегда и safe owned cleanup был бы невозможен.
    game.request_game_thread_exit();
    let release = game.release().await;
    let mut final_step = BillingRuntimeStep::default();
    game.drain_runtime_observations(&mut final_step);
    if !final_step.message_outcomes.is_empty()
        || !final_step.events.is_empty()
        || !final_step.billing_player_notices.is_empty()
        || !final_step.player_fill_notices.is_empty()
    {
        observe_step(&final_step);
    }
    drop(game);
    BillingGameThreadReport {
        initialization,
        completed_turns,
        runtime_error,
        release,
    }
}

impl Drop for CGame {
    fn drop(&mut self) {
        if let Some(task) = self.accept_task.take() {
            task.abort();
        }
        // Drop JoinSet отменяет оставшиеся owned I/O-задачи. Нормальный путь
        // GameThreadFunc до этого вызывает async Release и ждёт shutdown.
    }
}
