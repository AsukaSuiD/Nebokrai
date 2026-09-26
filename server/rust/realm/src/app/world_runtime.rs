//! Драйвер потока игры WorldServer (`GameThreadFunc`) вместе с data-bundle
//! инициализации и release. Источник контракта — точная пара
//! `.exe/Nworldserver.exe` + `.exe/WorldServer.pdb` (SHA-256 `F3AC454D…`,
//! RSDS совпадает; S_PUB32 `?GameThreadFunc@@YGIPAX@Z` `1:00019310`,
//! `?CreateGame@@YAHXZ` `1:00014660`, `?DeleteGame@@YAHXZ` `1:00000780`,
//! `?Init@CGame@@QAEHXZ` `1:00017ee0`, `?MainLoop@CGame@@QAEHXZ` `1:00018a00`,
//! `?Release@CGame@@QAEHXZ` `1:0000d7f0`).
//!
//! Драйвер выполняет исходный порядок `CreateGame -> Init -> MainLoop ->
//! Release -> DeleteGame`: инициализация потока до принятия хода, turn-цикл
//! питается до `legacy_result != 0`, сохранная барьера ждётся до Release,
//! три owner-а (`CGoodsWarMember`, `CIncrementLog`, `CSkillFactory`) изымаются
//! до `CGame::release` и возвращаются после него на обеих ветках, и только
//! штатный конец публикует exit-event, затем window-close request и код `0`.
//! Fatal `_exit(1)` и typed safe-blocks возвращают live game-owner, поэтому
//! Rust не приписывает им исходно отсутствовавший Release/DeleteGame.
//!
//! Тип игры входит в generic-трейт [`WorldGameThreadRuntime`] ассоциированным
//! `type Game`, а его создание отсечено фабричным параметром `fn() -> Box<Game>`
//! у edge-оболочки `process/worldserver.rs`. Сам `CGame` живёт в
//! [`crate::app::world_game`], process owners с реализациями этого трейта
//! (`type Game = CGame`) — в `crate::app::world_process*`.
//! `WorldGameInitContext` объявлен в [`crate::app::world_init_context`]
//! (шов `&mut dyn RegionParameterLoadTarget` вместо CGame-typed
//! `load_region_parameters`); его impl — в [`crate::app::world_process_init`].

use std::error::Error;
use std::fmt;
use std::future::Future;
use std::io;
use std::net::SocketAddrV4;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use nebokrai_shared::network::{ClientConnectError, ServerHostError};
use nebokrai_shared::resources::QuestSystemLoadReport;

use crate::activities::attackcitysys::{
    AttackCityEnemyRelationReport, AttackCityLoadError, AttackCityLoadReport,
};
use crate::activities::countrywarsys::{CountryWarLoadError, CountryWarLoadReport};
use crate::activities::factionwarsys::{FactionWarInitializationBlock, FactionWarInitializationReport};
use crate::activities::fournationwarsys::{FourNationWarLoadError, FourNationWarLoadReport};
use crate::activities::jjcsystem::JjcConfigurationLoadReport;
use crate::activities::misc::{CopyNumberScheduleBlock, CopyNumberScheduleReport};
use crate::activities::villagewarsys::{VillageWarLoadError, VillageWarLoadReport};
use crate::app::loginreconnectworker::LoginEndpointError;
use crate::app::world_client::CMyNetClient;
use crate::app::world_message::SendMessageError;
use crate::app::world_server::CMyNetServer;
use crate::app::worldserver::{
    AddLogTextDisposition, WorldRegionOwnerSerializationBlock, WorldReloadBlock,
    WorldSaveThreadHandleState,
};
use crate::auction::auctionlog::AuctionLogLoadOutcome;
use crate::billing::incrementlog::{CIncrementLog, IncrementLogLoadOutcome};
use crate::characters::honordb::HonorRanksLoadOutcome;
use crate::characters::playerloadworker::WorldGameInitWorkerHandleState;
use crate::characters::playerranks::{
    PlayerRanksInitializationReport, PlayerRanksReleaseReport, PlayerRanksScheduleBlock,
};
use crate::content::countryparam::{CountryParamLoadError, CountryParamLoadReport};
use crate::content::skillfactory::CSkillFactory;
use crate::content::{TimeToReturnLoadError, TimeToReturnLoadReport};
use crate::content::variablelist::VariableListLoadReport;
use crate::organizations::countryhandler::CountryHandlerInitializeReport;
use crate::organizations::factionenemyblock::FactionEnemyMutationBlock;
use crate::organizations::goodswarmember::{CGoodsWarMember, GoodsWarDatabaseLoadReport};
use crate::organizations::organizingctrl::{OrganizingInitializeBlock, OrganizingInitializeReport};
use crate::organizations::organizingparam::{
    OrganizingParamLoadError, OrganizingParamLoadReport, OrganizingParamReleaseReport,
};
use crate::persistence::largess::CostDatabaseSettings;
use crate::persistence::rsgenvar::GenVarLoadOutcome;
use crate::persistence::rsplayer::{PlayerRanksStatBlock, PlayerRanksStatOutcome};
use crate::persistence::rssetup::{LoadedSetupIds, WorldDatabaseSettings};
use crate::regions::region::RegionSerializationBlock;
use crate::regions::worldcityregion::CWorldCityRegion;
use crate::regions::worldcountrywarregion::WorldCountryWarRegion;
use crate::regions::worldregion::{
    CWorldRegion, WorldRegionOwnerRelationBlock, WorldRegionOwnerRelationReport,
};
use crate::regions::worldvillageregion::CWorldVillageRegion;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldSetupSource {
    Plain,
    Encoded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldSetupLoadReport {
    pub source: WorldSetupSource,
    pub parsed_pairs: usize,
    pub stopped_at_pair: Option<usize>,
    pub instance_title: Vec<u8>,
    pub instance_claimed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldServerSetupLoadReport {
    pub declared_records: i32,
    pub applied_records: usize,
    pub unique_game_servers: usize,
    pub stream_complete: bool,
    pub blocked_at_record: Option<usize>,
    pub read_end_notice: bool,
}

#[derive(Debug)]
pub enum WorldNetworkInitializationError {
    MissingSetupField(&'static str),
    Host(ServerHostError),
}

impl fmt::Display for WorldNetworkInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSetupField(field) => {
                write!(formatter, "World setup не назначил поле {field}")
            }
            Self::Host(error) => write!(formatter, "World listener не запущен: {error}"),
        }
    }
}

impl Error for WorldNetworkInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingSetupField(_) => None,
            Self::Host(error) => Some(error),
        }
    }
}

#[derive(Debug)]
pub struct WorldClientInitialization {
    pub endpoint: SocketAddrV4,
    pub registration: Result<i32, SendMessageError>,
}

#[derive(Debug)]
pub enum WorldClientInitializationError {
    MissingSetupField(&'static str),
    LoginAddressEncodingUnsupported,
    LoginAddressResolution,
    Bind(io::Error),
    Connect(ClientConnectError),
}

impl fmt::Display for WorldClientInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSetupField(field) => {
                write!(formatter, "World setup не назначил поле {field}")
            }
            Self::LoginAddressEncodingUnsupported => formatter.write_str(
                "кодировка LoginServer-адреса не поддерживается безопасным Linux resolver",
            ),
            Self::LoginAddressResolution => {
                formatter.write_str("LoginServer-адрес не разрешён в IPv4")
            }
            Self::Bind(error) => write!(formatter, "не создан World-to-Login socket: {error}"),
            Self::Connect(error) => {
                write!(formatter, "WorldServer не подключён к LoginServer: {error}")
            }
        }
    }
}

impl Error for WorldClientInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Bind(error) => Some(error),
            Self::Connect(error) => Some(error),
            Self::MissingSetupField(_) | Self::LoginAddressEncodingUnsupported
            | Self::LoginAddressResolution => None,
        }
    }
}

impl From<LoginEndpointError> for WorldClientInitializationError {
    fn from(error: LoginEndpointError) -> Self {
        match error {
            LoginEndpointError::EncodingUnsupported => Self::LoginAddressEncodingUnsupported,
            LoginEndpointError::Resolution => Self::LoginAddressResolution,
        }
    }
}

/// DB snapshot исходного семиаргументного `CMyAdoBase::Initialize`.
///
/// Тип намеренно не реализует `Debug`, чтобы credentials не попали в logs.
pub struct WorldGameDatabaseInitialization {
    pub settings: WorldDatabaseSettings,
    pub log_settings: WorldDatabaseSettings,
    pub cost_settings: CostDatabaseSettings,
    pub incoming_cost_settings: CostDatabaseSettings,
    pub load_largess_time_ms: u32,
    pub use_old_save_largess_way: bool,
    pub connection_type: Vec<u8>,
    pub legacy_zero: &'static [u8],
    pub integrated_security: &'static [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameDatabaseOwner {
    RsPlayer,
    RsGenVar,
    RsFaction,
    RsUnion,
    RsEnemyFactions,
    RsVillageWar,
    RsCityWar,
    RsRegion,
    DbCountry,
    GoodsWarMember,
    DbMisc,
    RsGodsBattle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameInitVoidOwner {
    InitializeOrganizingController,
    InitializeFactionWar,
    InitializeQuestSystem,
    CreateGeneralVariableList,
    LoadGeneralVariableList,
    LoadGeneralVariableData,
    InitializeBaseMessage,
    InitializeSocket,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameInitBooleanOwner {
    InitializeTimeToReturn,
    InitializeAttackCity,
    InitializeFourNationWar,
    InitializeVillageWar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameInitWorkerKind {
    WriteLog,
    LoadPlayerData { worker_index: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldGameInitOperatorNotice {
    pub title: Vec<u8>,
    pub message: Vec<u8>,
}

#[derive(Debug)]
pub enum WorldGameInitEvent {
    CrashReporterInstalled,
    RandomInitialized {
        seed: u32,
        discarded_roll: i32,
    },
    RustLocksReady,
    DebugStartPublished,
    ServerResourcesLoaded,
    SetupLoaded(WorldSetupLoadReport),
    ServerSetupLoaded(WorldServerSetupLoadReport),
    PlayerLoadThreadCountValidated(u32),
    StringTablesCleared,
    StringTableLoaded {
        package: Vec<u8>,
    },
    StringTablesCoded,
    DupliRegionSetupLoaded,
    DatabaseLayerInitialized,
    DatabaseOwnerCreated(WorldGameDatabaseOwner),
    GoodsWarMemberLoaded(GoodsWarDatabaseLoadReport),
    RsSetupOwnerCreated(LoadedSetupIds),
    VoidOwner(WorldGameInitVoidOwner),
    GeneralVariableListLoaded(VariableListLoadReport),
    GeneralVariableDataLoaded(GenVarLoadOutcome),
    TimeToReturnInitialized(TimeToReturnLoadReport),
    AttackCityInitialized(AttackCityLoadReport),
    AttackCityEnemyRelationsInitialized(AttackCityEnemyRelationReport),
    FourNationWarInitialized(FourNationWarLoadReport),
    VillageWarInitialized(VillageWarLoadReport),
    FactionWarInitialized(FactionWarInitializationReport),
    QuestSystemInitialized(QuestSystemLoadReport),
    JjcConfigurationLoaded(JjcConfigurationLoadReport),
    Reload {
        profile: &'static [u8],
        legacy_result: i32,
    },
    RegionParametersLoaded {
        succeeded: bool,
    },
    WordsFilterInitialized,
    BooleanOwner {
        owner: WorldGameInitBooleanOwner,
        succeeded: bool,
    },
    OrganizingParametersLoaded(OrganizingParamLoadReport),
    OrganizingControllerInitialized(OrganizingInitializeReport),
    GodsBattleFactionXydLoaded {
        succeeded: bool,
    },
    CountryParametersLoaded(CountryParamLoadReport),
    CountryHandlerInitialized(CountryHandlerInitializeReport),
    CountryWarInitialized(CountryWarLoadReport),
    RegionOwnerRelationInitialized {
        region_id: i32,
        report: WorldRegionOwnerRelationReport,
    },
    PlayerRanksInitialized(PlayerRanksInitializationReport),
    PlayerRanksLoaded(PlayerRanksStatRunReport),
    HonorRanksLoaded {
        started_at_ms: u32,
        finished_at_ms: u32,
        elapsed_ms: u32,
        outcome: HonorRanksLoadOutcome,
    },
    IncrementLogLoaded {
        outcome: IncrementLogLoadOutcome,
    },
    AuctionLogLoaded {
        outcome: AuctionLogLoadOutcome,
    },
    Log {
        payload: Vec<u8>,
        disposition: AddLogTextDisposition,
    },
    NetworkClientInitialized(WorldClientInitialization),
    NetworkServerInitialized,
    PlayerDataQueueCleared,
    CopyNumberResetScheduled(CopyNumberScheduleReport),
    WorkerStarted {
        kind: WorldGameInitWorkerKind,
        handle: WorldGameInitWorkerHandleState,
    },
    OperatorNotice(WorldGameInitOperatorNotice),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldStringTableEncodingBlock {
    pub entry_count: usize,
}

#[derive(Debug)]
pub enum WorldGameInitBlockReason<ContextBlock> {
    SetupOpen(WorldSetupOpenError),
    ExistingInstance { title: Vec<u8> },
    ServerSetup(io::Error),
    MissingPlayerLoadThreadCount,
    InvalidPlayerLoadThreadCount { count: u32, legacy_exit_code: i32 },
    DefaultLanguageTable,
    ConfiguredLanguageTable,
    StringTableEncoding(WorldStringTableEncodingBlock),
    DupliRegionSetup,
    Context(ContextBlock),
    Reload(WorldReloadBlock),
    JjcConfiguration(JjcConfigurationLoadReport),
    BooleanOwner(WorldGameInitBooleanOwner),
    TimeToReturnLoad(TimeToReturnLoadError),
    AttackCityLoad(AttackCityLoadError),
    AttackCityEnemyRelation(WorldGameInitAttackCityRelationBlock),
    FourNationWarLoad(FourNationWarLoadError),
    VillageWarLoad(VillageWarLoadError),
    OrganizingParameters(OrganizingParamLoadError),
    OrganizingController(OrganizingInitializeBlock),
    CountryParameters(CountryParamLoadError),
    CountryHandler,
    CountryWarLoad(CountryWarLoadError),
    CountryWar,
    RegionOwnerRelation {
        region_id: i32,
        source: WorldRegionOwnerRelationBlock,
    },
    FactionWar(FactionWarInitializationBlock),
    PlayerRanksSchedule(PlayerRanksScheduleBlock),
    PlayerRanksStat(PlayerRanksStatRunBlock),
    NetworkClient(WorldClientInitializationError),
    NetworkServer(WorldNetworkInitializationError),
    CopyNumberSchedule(CopyNumberScheduleBlock),
}

#[derive(Debug)]
pub struct WorldGameInitBlock<ContextBlock> {
    pub events: Vec<WorldGameInitEvent>,
    pub reason: WorldGameInitBlockReason<ContextBlock>,
}

#[derive(Debug)]
pub struct WorldGameInitReport {
    pub events: Vec<WorldGameInitEvent>,
    pub legacy_result: i32,
}

pub type WorldGameInitResult<ContextBlock> =
    Result<WorldGameInitReport, Box<WorldGameInitBlock<ContextBlock>>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameInitAttackCityRelationBlock {
    NullUnion { map_key: i32 },
    MissingOrganizing { organizing_id: i32 },
    EnemyMutation {
        organizing_id: i32,
        enemy_organizing_id: i32,
        source: FactionEnemyMutationBlock,
    },
}

#[derive(Debug)]
pub struct PlayerRanksStatRunReport {
    pub started_at_ms: u32,
    pub finished_at_ms: u32,
    pub elapsed_ms: u32,
    pub outcome: PlayerRanksStatOutcome,
    pub start_log: AddLogTextDisposition,
    pub complete_log: AddLogTextDisposition,
}

#[derive(Debug)]
pub struct PlayerRanksStatRunBlock {
    pub started_at_ms: u32,
    pub start_log: AddLogTextDisposition,
    pub source: PlayerRanksStatBlock,
}

#[derive(Debug)]
pub struct WorldSetupOpenError {
    plain_path: PathBuf,
    plain: io::Error,
    encoded_path: PathBuf,
    encoded: io::Error,
}

impl WorldSetupOpenError {
    pub fn new(
        plain_path: PathBuf,
        plain: io::Error,
        encoded_path: PathBuf,
        encoded: io::Error,
    ) -> Self {
        Self {
            plain_path,
            plain,
            encoded_path,
            encoded,
        }
    }
}

impl fmt::Display for WorldSetupOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "не открыты {} ({}) и {} ({})",
            self.plain_path.display(),
            self.plain,
            self.encoded_path.display(),
            self.encoded
        )
    }
}

impl Error for WorldSetupOpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.encoded)
    }
}

pub enum WorldRegionOwner {
    Base(Box<CWorldRegion>),
    Village(Box<CWorldVillageRegion>),
    City(Box<CWorldCityRegion>),
    Country(Box<WorldCountryWarRegion>),
}

impl WorldRegionOwner {
    pub fn base(&self) -> &CWorldRegion {
        match self {
            Self::Base(region) => region,
            Self::Village(region) => region.war().base(),
            Self::City(region) => region.war().base(),
            Self::Country(region) => region.base(),
        }
    }

    pub fn base_mut(&mut self) -> &mut CWorldRegion {
        match self {
            Self::Base(region) => region,
            Self::Village(region) => region.war_mut().base_mut(),
            Self::City(region) => region.war_mut().base_mut(),
            Self::Country(region) => region.base_mut(),
        }
    }

    pub fn save_to_resource_directory(
        &mut self,
        runtime_directory: &Path,
    ) -> Result<i32, RegionSerializationBlock> {
        self.base_mut()
            .region_base_mut()
            .save_to_resource_directory(runtime_directory)
    }

 /// Выполняет virtual AI всех поставочных World region owner-ов.
 /// Их slot `+0x40` указывает на общий однокомандный `ret`.
    pub const fn ai(&mut self) {}

    pub fn add_full_initial_snapshot(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), WorldRegionOwnerSerializationBlock> {
        match self {
            Self::Base(region) => {
                let _ = region
                    .add_to_byte_array(destination, true)
                    .map_err(WorldRegionOwnerSerializationBlock::Base)?;
            }
            Self::Village(region) => {
                let _ = region
                    .add_to_byte_array(destination, true)
                    .map_err(WorldRegionOwnerSerializationBlock::Village)?;
            }
            Self::City(region) => {
                let _ = region
                    .add_to_byte_array(destination, true)
                    .map_err(WorldRegionOwnerSerializationBlock::City)?;
            }
            Self::Country(region) => {
                let _ = region
                    .add_to_byte_array(destination, true)
                    .map_err(WorldRegionOwnerSerializationBlock::Country)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldSaveCityRegionBlock {
    UninitializedRegionType { region_id: i32 },
    NullCityRegion { region_id: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameReleaseLiveList {
    Creation,
    Restore,
    Deletion,
    Online,
    Offline,
    Login,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameReleaseVoidOwner {
    ReleaseGoodsLinks,
    UninitializeTimeToReturn,
    UninitializeIncrementLog,
    ReleaseCountryHandler,
    ReleaseWordsFilter,
    ReleaseOrganizingController,
    ReleaseAttackCity,
    ReleaseVillageWar,
    ReleaseQuestSystem,
    ReleaseFactionWar,
    ReleaseTimer,
    ClearSkillCache,
    ClearSkillUsageCache,
    ReleaseGoodsFactory,
    CleanupSocket,
    ReleaseBaseMessage,
    ReleaseNetSessionManager,
    RequestWriteLogWorkerExit,
    UninitializeLargess,
    UninitializeDatabaseLayer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameReleaseOptionalOwner {
    FunctionListFileData,
    VariableListFileData,
    ScriptFileData,
    GeneralVariableList,
    DefaultClientResource,
    DupliRegionSetup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameReleaseDatabaseOwner {
    RsPlayer,
    RsSetup,
    RsGenVar,
    RsFaction,
    RsUnion,
    RsEnemyFactions,
    RsVillageWar,
    RsCityWar,
    GoodsWarMember,
    RsRegion,
    DbCountry,
    RsGodsBattle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldGameReleaseEvent {
    DebugPublished(&'static [u8]),
    PlayerDataQueueCleared,
    CityRegionSaved {
        region_id: i32,
    },
    NetworkServerWorkerExited,
    NetworkClientWorkerExited,
    LiveListCleared {
        owner: WorldGameReleaseLiveList,
        entries: usize,
    },
    PlayerMapCleared {
        entries: usize,
    },
    DbDataCleared,
    PlayerRanksReleased(PlayerRanksReleaseReport),
    OrganizingParametersReleased(OrganizingParamReleaseReport),
    SaveWorkerJoined {
        previous_handle: WorldSaveThreadHandleState,
    },
    VoidOwner(WorldGameReleaseVoidOwner),
    RegionOwnerReleased {
        region_id: i32,
    },
    OptionalOwner {
        owner: WorldGameReleaseOptionalOwner,
        released: bool,
    },
    NetworkClientReleased,
    NetworkServerReleased,
    DatabaseOwner {
        owner: WorldGameReleaseDatabaseOwner,
        released: bool,
    },
    DatabaseMiscRetained,
    RustLocksRetired,
    WriteLogWorkerJoined {
        previous_handle: WorldGameInitWorkerHandleState,
    },
    PlayerLoadWorkersStopped {
        workers: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldGameReleaseReport {
    pub events: Vec<WorldGameReleaseEvent>,
    pub legacy_result: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldGameReleaseBlock {
    pub events: Vec<WorldGameReleaseEvent>,
    pub block: WorldSaveCityRegionBlock,
}

pub type WorldGameReleaseResult = Result<WorldGameReleaseReport, Box<WorldGameReleaseBlock>>;

pub trait WorldGameReleaseContext {
    fn put_debug_string(&mut self, payload: &'static [u8]);
    fn save_city_region(&mut self, region_id: i32, region: &mut WorldRegionOwner);
    fn exit_network_server_worker(&mut self, server: &mut CMyNetServer);
    fn exit_network_client_worker(&mut self, client: &mut CMyNetClient);
    fn release_void_owner(&mut self, owner: WorldGameReleaseVoidOwner);
    fn release_optional_owner(&mut self, owner: WorldGameReleaseOptionalOwner) -> bool;
    fn release_database_owner(&mut self, owner: WorldGameReleaseDatabaseOwner) -> bool;
    fn release_player_ranks(&mut self) -> PlayerRanksReleaseReport;
 /// Снимает events, которые поставил `COrganizingParam`, затем уничтожает
 /// его раньше общего timer-owner-а.
    fn release_organizing_parameters(&mut self) -> OrganizingParamReleaseReport;

    fn join_save_worker(&mut self) -> WorldSaveThreadHandleState;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldCreateGameBlock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldCreateGameReport {
    pub legacy_result: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldDeleteGameReport {
    pub owner_was_present: bool,
    pub legacy_result: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameThreadStop {
    InitializationFailed,
    ExitRequested,
    MainLoopReturned { legacy_result: i32 },
}

pub enum WorldGameThreadInitialization<ContextBlock> {
    Complete(WorldGameInitReport),
    Failed(Box<WorldGameInitBlock<ContextBlock>>),
}

pub enum WorldGameThreadReport<Game, InitBlock, MainLoopBlock> {
    Complete {
        creation: WorldCreateGameReport,
        initialization: WorldGameThreadInitialization<InitBlock>,
        main_loop_calls: u64,
        stop: WorldGameThreadStop,
        release: WorldGameReleaseReport,
        deletion: WorldDeleteGameReport,
        legacy_exit_code: u32,
    },
    FatalInitialization {
        game: Box<Game>,
        creation: WorldCreateGameReport,
        block: Box<WorldGameInitBlock<InitBlock>>,
    },
    BlockedInitialization {
        game: Box<Game>,
        creation: WorldCreateGameReport,
        block: Box<WorldGameInitBlock<InitBlock>>,
    },
    BlockedMainLoop {
        game: Box<Game>,
        creation: WorldCreateGameReport,
        initialization: WorldGameInitReport,
        main_loop_calls: u64,
        block: MainLoopBlock,
    },
    BlockedRelease {
        game: Box<Game>,
        creation: WorldCreateGameReport,
        initialization: WorldGameThreadInitialization<InitBlock>,
        main_loop_calls: u64,
        stop: WorldGameThreadStop,
        block: Box<WorldGameReleaseBlock>,
    },
}

/// Игровой владелец, управляемый [`game_thread_func`]: драйвер вручает ему
/// только полный `Release` с заранее изъятыми owner-ами в исходном порядке.
pub trait WorldGameThreadGame {
    fn release<Context: WorldGameReleaseContext>(
        &mut self,
        context: &mut Context,
        goods_war: &mut CGoodsWarMember,
        increment_log: &mut CIncrementLog,
        skills: &mut CSkillFactory,
    ) -> WorldGameReleaseResult;
}

pub trait WorldGameThreadRuntime: WorldGameReleaseContext {
    type Game;
    type InitBlock;
    type MainLoopBlock;

    fn initialize_game<'game>(
        &'game mut self,
        game: &'game mut Self::Game,
    ) -> Pin<Box<dyn Future<Output = WorldGameInitResult<Self::InitBlock>> + 'game>>;
    fn game_thread_exit_requested(&self) -> bool;
    fn run_main_loop<'game>(
        &'game mut self,
        game: &'game mut Self::Game,
    ) -> Pin<Box<dyn Future<Output = Result<i32, Self::MainLoopBlock>> + 'game>>;
    fn wait_for_save_barrier(&mut self);
    fn take_goods_war_member(&mut self) -> CGoodsWarMember;
    fn restore_goods_war_member(&mut self, owner: CGoodsWarMember);
    fn take_increment_log(&mut self) -> CIncrementLog;
    fn restore_increment_log(&mut self, owner: CIncrementLog);
    fn take_skill_factory(&mut self) -> CSkillFactory;
    fn restore_skill_factory(&mut self, owner: CSkillFactory);
    fn signal_game_thread_exit(&mut self);
    fn request_window_close(&mut self);
}

pub fn create_game<Game>(
    game: &mut Option<Box<Game>>,
    new_game: fn() -> Box<Game>,
) -> Result<WorldCreateGameReport, WorldCreateGameBlock> {
    if game.is_some() {
 // Повторный CreateGame перезаписывал бы global pointer и терял прежний
 // owner. Такой caller не определён, поэтому safe API не создаёт утечку.
        return Err(WorldCreateGameBlock);
    }
    *game = Some(new_game());
    Ok(WorldCreateGameReport { legacy_result: 1 })
}

pub fn get_game<Game>(game: &mut Option<Box<Game>>) -> Option<&mut Game> {
    game.as_deref_mut()
}

pub fn delete_game<Game>(game: &mut Option<Box<Game>>) -> WorldDeleteGameReport {
    let owner_was_present = game.is_some();
    drop(game.take());
    WorldDeleteGameReport {
        owner_was_present,
        legacy_result: 1,
    }
}

#[derive(Clone, Copy)]
enum WorldGameInitCallerDisposition {
    ReturnedFalse,
    FatalNonreturn,
    Blocked,
}

fn classify_game_init_for_caller<ContextBlock>(
    block: &WorldGameInitBlock<ContextBlock>,
) -> WorldGameInitCallerDisposition {
    match &block.reason {
        WorldGameInitBlockReason::InvalidPlayerLoadThreadCount { .. } => {
            WorldGameInitCallerDisposition::FatalNonreturn
        }
        WorldGameInitBlockReason::MissingPlayerLoadThreadCount
        | WorldGameInitBlockReason::Context(_)
        | WorldGameInitBlockReason::Reload(_)
        | WorldGameInitBlockReason::OrganizingParameters(_)
        | WorldGameInitBlockReason::PlayerRanksSchedule(_)
        | WorldGameInitBlockReason::PlayerRanksStat(_)
        | WorldGameInitBlockReason::NetworkClient(
            WorldClientInitializationError::MissingSetupField(_)
            | WorldClientInitializationError::LoginAddressEncodingUnsupported,
        )
        | WorldGameInitBlockReason::NetworkServer(
            WorldNetworkInitializationError::MissingSetupField(_),
        ) => WorldGameInitCallerDisposition::Blocked,
        _ => WorldGameInitCallerDisposition::ReturnedFalse,
    }
}

/// Выполняет `CreateGame -> Init -> MainLoop -> Release -> DeleteGame`.
///
/// Fatal `_exit(1)` и typed safe-blocks возвращают live `Box<Game>`, поэтому
/// Rust не приписывает им исходно отсутствовавший Release/DeleteGame. Только
/// штатный конец публикует exit-event, затем window-close request и код `0`.
pub async fn game_thread_func<Runtime: WorldGameThreadRuntime>(
    runtime: &mut Runtime,
    new_game: fn() -> Box<Runtime::Game>,
) -> WorldGameThreadReport<Runtime::Game, Runtime::InitBlock, Runtime::MainLoopBlock>
where
    Runtime::Game: WorldGameThreadGame,
{
    let mut game_slot = None;
    let creation =
        create_game(&mut game_slot, new_game).expect("GameThreadFunc начинает с пустого g_pGame");

    let initialization_result = runtime
        .initialize_game(
            game_slot
                .as_deref_mut()
                .expect("CreateGame только что опубликовал owner"),
        )
        .await;

    let mut main_loop_calls = 0_u64;
    let (initialization, stop) = match initialization_result {
        Ok(initialization) => {
            let stop = loop {
                if runtime.game_thread_exit_requested() {
                    break WorldGameThreadStop::ExitRequested;
                }
                let result = runtime.run_main_loop(
                    game_slot
                        .as_deref_mut()
                        .expect("game owner жив до Release/DeleteGame"),
                )
                .await;
                main_loop_calls = main_loop_calls.wrapping_add(1);
                match result {
                    Ok(legacy_result) if legacy_result != 0 => {}
                    Ok(legacy_result) => {
                        break WorldGameThreadStop::MainLoopReturned { legacy_result };
                    }
                    Err(block) => {
                        return WorldGameThreadReport::BlockedMainLoop {
                            game: game_slot
                                .take()
                                .expect("blocked MainLoop сохраняет live game-owner"),
                            creation,
                            initialization,
                            main_loop_calls,
                            block,
                        };
                    }
                }
            };
            runtime.wait_for_save_barrier();
            (
                WorldGameThreadInitialization::Complete(initialization),
                stop,
            )
        }
        Err(block) => match classify_game_init_for_caller(&block) {
            WorldGameInitCallerDisposition::FatalNonreturn => {
                return WorldGameThreadReport::FatalInitialization {
                    game: game_slot
                        .take()
                        .expect("fatal Init сохраняет опубликованный owner"),
                    creation,
                    block,
                };
            }
            WorldGameInitCallerDisposition::Blocked => {
                return WorldGameThreadReport::BlockedInitialization {
                    game: game_slot
                        .take()
                        .expect("blocked Init сохраняет опубликованный owner"),
                    creation,
                    block,
                };
            }
            WorldGameInitCallerDisposition::ReturnedFalse => (
                WorldGameThreadInitialization::Failed(block),
                WorldGameThreadStop::InitializationFailed,
            ),
        },
    };

    let mut goods_war = runtime.take_goods_war_member();
    let mut increment_log = runtime.take_increment_log();
    let mut skills = runtime.take_skill_factory();
    let release = match game_slot
        .as_deref_mut()
        .expect("Release вызывается до DeleteGame")
        .release(runtime, &mut goods_war, &mut increment_log, &mut skills)
    {
        Ok(release) => release,
        Err(block) => {
            runtime.restore_goods_war_member(goods_war);
            runtime.restore_increment_log(increment_log);
            runtime.restore_skill_factory(skills);
            return WorldGameThreadReport::BlockedRelease {
                game: game_slot
                    .take()
                    .expect("blocked Release сохраняет live game-owner"),
                creation,
                initialization,
                main_loop_calls,
                stop,
                block,
            };
        }
    };
    runtime.restore_goods_war_member(goods_war);
    runtime.restore_increment_log(increment_log);
    runtime.restore_skill_factory(skills);
    let deletion = delete_game(&mut game_slot);
    runtime.signal_game_thread_exit();
    runtime.request_window_close();
    WorldGameThreadReport::Complete {
        creation,
        initialization,
        main_loop_calls,
        stop,
        release,
        deletion,
        legacy_exit_code: 0,
    }
}
