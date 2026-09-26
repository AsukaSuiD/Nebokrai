//! Данные хода `CGame::MainLoop` WorldServer без самой игры, перенесённые из
//! `src/worldserver/worldserver/game.rs` волной C5-A (hub-data уровень):
//! stage-отчёты (timer/AI/session-factory/faction-war/db-misc/net-session/
//! minute/bai-tan-jjc/save/tail/ping), конфигурация хода и связка
//! `StateOwners`/`Owners`/`Callbacks`/`Block`/`Report`.
//! Источник контракта — та же точная пара, что у [`crate::app::world_runtime`]
//! (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`,
//! RSDS совпадает).
//!
//! Link-типы DB-владельцев отделены от конкретных Tiberius-реализаций: поле
//! `rs_player` входит generic-параметром `RsPlayer` (шов
//! [`RsPlayerOwner`]<`CPlayer`), поле `largess` — generic-параметром `Largess`
//! (базовый шов [`LargessOwner`] и worker-lifecycle остаются у старого
//! владельца). Доставка `DoneOutList` и хранитель `CGame` внутри `refresh_union
//! _owned_city` входят generic-параметром `Game`; старый пакет специализирует
//! связку своими `CGame`/`TiberiusRsPlayer`/`TiberiusLargess` на границе
//! process-цепочки, точки которой (`process_message`/`process_world_message`/
//! `route_loaded_player`) держат прежнюю конкретную декларацию до следующей
//! порции. Глубокие timer-контексты (`WorldTimerHandler` и effect-адаптеры
//! countdown/phase) остаются у владельца стадий; worker-адаптеры
//! `WorldJjcWorkerContext`/`WorldLeiTingWorkerContext` с trait-связкой
//! (`WorldJjcRuntimeContext`/`WorldLeiTingRuntimeContext`) уехали в
//! [`crate::app::world_main_loop_contexts`] волной C5-B вместе с impl-ами
//! process-контекстов (единая волна сохранила orphan-rule).

use std::convert::Infallible;
use std::sync::Arc;

use parking_lot::Mutex;

use nebokrai_shared::resources::{CGodsBattleConf, LeiTingLocalTime};
use nebokrai_shared::runtime::{
    CNetSessionManager, CTimer, NetSessionRunReport, TimerCallbackSource, TimerId, TimerRunReport,
};
use nebokrai_shared::values::TagTime;

use crate::activities::attackcitysys::{
    AttackCityCallbackKind, AttackCityCallbacks, AttackCityCountdownBlock,
    AttackCityCountdownReport, AttackCityPhaseReport, CAttackCitySys,
};
use crate::activities::countrywarsys::{
    CountryWarCallbackKind, CountryWarCallbacks, CountryWarFinishBlock, CountryWarFinishReport,
    CountryWarPhaseBlock, CountryWarPhaseReport, CountryWarStartBlock, CountryWarStartReport,
    CountryWarSys, CountryWarTopInfoBlock, CountryWarTopInfoReport,
};
use crate::activities::factionwarsys::{CFactionWarSys, FactionWarRunReport, FactionWarStopBlock};
use crate::activities::fournationwarsys::{
    CFourNationWarSys, FourNationWarCalendarBlock, FourNationWarCallbackKind,
    FourNationWarCallbacks,
};
use crate::activities::jjcmaintenanceworker::WorldJjcWeekClearWorker;
use crate::activities::jjcsystem::{CJJcSystem, JjcRunBlock, JjcRunReport};
use crate::activities::leiting::{CLeiTing, LeiTingBlock, LeiTingRunReport};
use crate::activities::leitingresetworker::WorldLeiTingResetWorker;
use crate::activities::misc::{CopyNumberResetReport, CopyNumberScheduleBlock, CopyNumberTimerState};
use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::activities::villagewarsys::{
    CVillageWarSys, VillageWarCallbackKind, VillageWarCallbacks, VillageWarCountdownBlock,
    VillageWarCountdownReport, VillageWarPhaseReport,
};
use crate::app::baitan::WorldDoneBaiTanListReport;
use crate::app::organsysmessage::WorldOrganizingSessionRuntimeOwner;
use crate::app::playerdataqueue::WorldMainLoopPlayerDataQueueStageReport;
use crate::app::world_game_view::{WorldGameView, WorldLoginTimeoutTeamExit};
use crate::app::world_hub_data::{
    WorldMainLoopAiStageReport, WorldMainLoopClockState, WorldMainLoopFactionWarBlock,
    WorldMainLoopInitializationState, WorldMainLoopLargessState, WorldMainLoopLoginReleaseState,
    WorldMainLoopProfileState, WorldMainLoopSessionFactoryStageReport,
    WorldMainLoopTailClockInitialization, WorldMainLoopTailClockState,
    WorldProcessMessageStageState, WorldUnionApplicationRuntimeReport,
};
use crate::app::world_message::{CMessage, SendMessageError};
use crate::app::world_reload_profiles::{
    WorldMainLoopLargessGateReport, WorldMainLoopMaintenanceBlock, WorldMainLoopMaintenanceReport,
    WorldMainLoopProfileInitialization, WorldMainLoopRefreshInitialization,
    WorldMainLoopRefreshStageReport, WorldMainLoopResourceContext, WorldMainLoopSaveInitialization,
    WorldPlayerRanksRequestState, WorldProcessMessageStageReport, WorldReloadProfileFlags,
    WorldReloadProfilesReport,
};
use crate::app::world_save_reports::{
    WorldCollectPlayerDataBroadcast, WorldCollectPlayerDataRequestState,
    WorldManualSaveRequestReport, WorldRunSaveTriggerDisposition, WorldRunSaveTriggerState,
};
use crate::app::worldserver::{
    AddLogTextDisposition, WorldGenerateDbDataBlock, WorldLogTextOwner,
    WorldOnlinePlayerRemoveOutcome, WorldRefreshInfoHighWater, WorldSaveThreadHandleState,
};
use crate::auction::auctionlog::CAuctionLog;
use crate::billing::incrementlog::CIncrementLog;
use crate::characters::honorranks::CHonorRanks;
use crate::characters::player::{CPlayer, PlayerCodecError};
use crate::characters::playerranks::{
    CPlayerRanks, PlayerRanksGameServerUpdate, PlayerRanksScheduleBlock,
    PlayerRanksSerializationBlock,
};
use crate::content::countryparam::CCountryParam;
use crate::content::skillfactory::CSkillFactory;
use crate::content::{TimeToReturn, TimeToReturnCallbacks, TimeToReturnFireReport};
use crate::content::variablelist::CVariableList;
use crate::organizations::countryhandler::{CCountryHandler, CountryRunBlock, CountryRunReport};
use crate::organizations::goodswarmember::CGoodsWarMember;
use crate::organizations::organizingctrl::{
    COrganizingCtrl, OrganizingRunBlock, OrganizingRunReport, OrganizingSaveDataBlock,
};
use crate::organizations::organizingparam::{
    COrganizingParam, OrganizingTaxScheduleBlock, OrganizingTodayTaxRefreshReport,
};
use crate::persistence::dbmisc::{
    CDbMisc, DbMiscDoneInReport, DbMiscDoneOutBlock, DbMiscDoneOutReport, DbMiscGameServer,
    DbMiscLoadAuctionReport,
};
use crate::persistence::rssetup::WorldTdsClient;
use crate::persistence::savedb::SaveDataLifecycleState;
use crate::persistence::saveworker::WorldSaveRuntimeContext;
use crate::persistence::writelogqueue::WorldWriteLogQueue;
use crate::app::world_runtime::{PlayerRanksStatRunBlock, PlayerRanksStatRunReport};
use crate::sessions::csessionfactory::CSessionFactory;

#[derive(Debug)]
pub struct PlayerRanksTimerRefreshReport {
    pub stat: PlayerRanksStatRunReport,
    pub publication: PlayerRanksGameServerUpdate,
    pub next_time: TagTime,
    pub next_event_id: Option<TimerId>,
}

#[derive(Debug)]
pub enum PlayerRanksTimerRefreshBlock {
    Stat(PlayerRanksStatRunBlock),
    Serialization(PlayerRanksSerializationBlock),
    Schedule(PlayerRanksScheduleBlock),
}

#[derive(Debug)]
pub enum CountryWarTimerReport {
    Phase {
        callback: CountryWarCallbackKind,
        war_id: i32,
        report: CountryWarPhaseReport,
    },
    Start {
        war_id: i32,
        report: CountryWarStartReport,
    },
    End {
        war_id: i32,
        report: CountryWarFinishReport,
    },
    TopInfo {
        callback: CountryWarCallbackKind,
        report: CountryWarTopInfoReport,
    },
}

#[derive(Debug)]
pub enum CountryWarTimerBlock {
    Phase(CountryWarPhaseBlock<Infallible>),
    Start(CountryWarStartBlock<Infallible>),
    End(CountryWarFinishBlock<Infallible>),
    TopInfo(CountryWarTopInfoBlock<Infallible>),
}

#[derive(Debug)]
pub enum AttackCityTimerOutcome {
    Phase(AttackCityPhaseReport),
    Countdown(AttackCityCountdownReport),
}

#[derive(Debug)]
pub struct AttackCityTimerReport {
    pub callback: AttackCityCallbackKind,
    pub war_number: i32,
    pub outcome: AttackCityTimerOutcome,
}

#[derive(Debug)]
pub enum VillageWarTimerOutcome {
    Phase(VillageWarPhaseReport),
    Countdown(VillageWarCountdownReport),
}

#[derive(Debug)]
pub struct VillageWarTimerReport {
    pub callback: VillageWarCallbackKind,
    pub war_number: i32,
    pub outcome: VillageWarTimerOutcome,
}

#[derive(Debug)]
pub enum WorldTimerCallbackBlock {
    CopyNumber(CopyNumberScheduleBlock),
    PlayerRanks(PlayerRanksTimerRefreshBlock),
    OrganizingTax(OrganizingTaxScheduleBlock),
    AttackCity(AttackCityCountdownBlock<Infallible>),
    VillageWar(VillageWarCountdownBlock<Infallible>),
    CountryWar(CountryWarTimerBlock),
    FourNationWar(FourNationWarCalendarBlock),
    UnexpectedCallback {
        source: TimerCallbackSource,
        parameter: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FourNationWarTimerReport {
    pub callback: FourNationWarCallbackKind,
    pub index: i32,
}

#[derive(Debug)]
pub struct WorldMainLoopTimerStageBlock {
    pub timer: TimerRunReport,
    pub source: WorldTimerCallbackBlock,
}

#[derive(Debug)]
pub struct WorldMainLoopTimerStageReport {
    pub timer: TimerRunReport,
    pub copy_number_resets: Vec<CopyNumberResetReport>,
    pub player_ranks: Vec<PlayerRanksTimerRefreshReport>,
    pub organizing_taxes: Vec<OrganizingTodayTaxRefreshReport>,
    pub time_to_returns: Vec<TimeToReturnFireReport>,
    pub attack_city_wars: Vec<AttackCityTimerReport>,
    pub village_wars: Vec<VillageWarTimerReport>,
    pub country_wars: Vec<CountryWarTimerReport>,
    pub four_nation_wars: Vec<FourNationWarTimerReport>,
    pub finished_at_ms: u32,
    pub elapsed_ms: u32,
    pub accumulated_time_ms: u32,
    pub next_stage_started_at_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMainLoopFactionWarStageReport {
    pub faction_war: FactionWarRunReport,
    pub finished_at_ms: u32,
    pub elapsed_ms: u32,
    pub accumulated_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMainLoopDbMiscStageReport {
    pub output: DbMiscDoneOutReport,
    pub input: DbMiscDoneInReport,
    pub auction: DbMiscLoadAuctionReport,
    pub next_stage_started_at_ms: u32,
}

/// Краткоживущий доступ `DoneOutList` к актуальным World registries.
///
/// TDS-контекст не хранит ссылку на `CGame`; этот адаптер создаётся только на
/// время output batch после завершения всех предшествующих MainLoop-мутаций.
/// Игра входит объявленным швом [`WorldGameView`] вместо конкретного `CGame`.
pub struct WorldDbMiscDeliveryContext<'a, Game: ?Sized> {
    pub game: &'a Game,
    pub gold_coin_index: u32,
    pub offline_drops: usize,
}

impl<Game: WorldGameView> crate::persistence::dbmisc::DbMiscDeliveryContext
    for WorldDbMiscDeliveryContext<'_, Game>
{
    fn player_game_server(&mut self, player_id: u32) -> Option<DbMiscGameServer> {
        self.game
            .player_game_server(player_id as i32)
            .map(|server| DbMiscGameServer {
                connected: server.connected,
                index: server.index,
            })
    }

    fn online_player_id(&mut self, player_id: u32) -> Option<i32> {
        self.game.online_player_by_id(player_id).map(CPlayer::get_id)
    }

    fn send_to_map_id(&mut self, message: &CMessage, map_id: u32) {
        let _ = self.game.send_msg_to_game_server(map_id as i32, message);
    }

    fn log_player_not_online_drop_goods(&mut self) {
        self.offline_drops = self.offline_drops.saturating_add(1);
    }

    fn gold_coin_index(&mut self) -> u32 {
        self.gold_coin_index
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldMainLoopNetSessionStageReport {
    pub sessions: NetSessionRunReport,
    pub union_applications: WorldUnionApplicationRuntimeReport,
    pub finished_at_ms: u32,
    pub elapsed_ms: u32,
    pub accumulated_time_ms: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldMainLoopMinuteStageReport {
    pub initialization: WorldMainLoopTailClockInitialization,
    pub current_tick_ms: u32,
    pub minute_delta: i32,
    pub organizing: OrganizingRunReport,
    pub country: CountryRunReport,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldMainLoopMinuteStageBlock {
    Organizing(OrganizingRunBlock),
    Country(CountryRunBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldMainLoopBaiTanJjcStageReport {
    pub bai_tan: WorldDoneBaiTanListReport,
    pub jjc: JjcRunReport,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldLoginTimeoutFriendOutcome {
    Offline {
        friend_index: usize,
    },
    Notified {
        friend_index: usize,
        friend_player_id: u32,
        target_game_server_index: Option<u32>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldLoginTimeoutEntryOutcome {
    Waiting {
        player_id: u32,
        elapsed_ms: u32,
    },
    ExpiredMissingPlayer {
        player_id: u32,
        elapsed_ms: u32,
    },
    Released {
        player_id: u32,
        elapsed_ms: u32,
        login_delivery: Result<i32, SendMessageError>,
        team_id: i32,
        team_session_id: i32,
        team_exit: WorldLoginTimeoutTeamExit,
        online_removal: WorldOnlinePlayerRemoveOutcome,
        offline_inserted: bool,
        friend_outcomes: Vec<WorldLoginTimeoutFriendOutcome>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldLoginTimeoutReport {
    pub snapshot_tick_ms: u32,
    pub entries: Vec<WorldLoginTimeoutEntryOutcome>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMainLoopPacingReport {
    pub sampled_tick_ms: u32,
    pub wait_duration_ms: Option<u32>,
    pub next_deadline_ms: u32,
    pub signed_lag_ms: i32,
    pub warning_resync_tick_ms: Option<u32>,
    pub release_gate_tick_ms: u32,
    pub release_gate_elapsed_ms: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldMainLoopTailStageReport {
    BlockedMissingReleaseInterval {
        pacing: WorldMainLoopPacingReport,
    },
    Complete {
        pacing: WorldMainLoopPacingReport,
        release_interval_ms: u32,
        login_timeout: Option<WorldLoginTimeoutReport>,
    },
}

#[derive(Debug)]
pub struct WorldMainLoopSaveStageReport {
    pub manual_request: Option<WorldManualSaveRequestReport>,
    pub profile_started_at_ms: u32,
    pub elapsed_ms: u32,
    pub save_point_time_ms: u32,
    pub disposition: WorldMainLoopSaveStageDisposition,
}

#[derive(Debug)]
pub enum WorldMainLoopSaveStageDisposition {
    IntervalNotElapsed,
    SaveLockBusy {
        adjusted_last_save_point_time_ms: u32,
    },
    Triggered(WorldRunSaveTriggerDisposition),
}

#[derive(Clone, Copy, Debug)]
pub struct WorldMainLoopConfiguration {
    pub refresh_external_counts: WorldRefreshExternalCounts,
}

pub struct WorldMainLoopStateOwners<'a> {
    pub initialization: &'a mut WorldMainLoopInitializationState,
    pub clocks: &'a mut WorldMainLoopClockState,
    pub tail_clocks: &'a mut WorldMainLoopTailClockState,
    pub login_release: &'a mut WorldMainLoopLoginReleaseState,
    pub largess: &'a mut WorldMainLoopLargessState,
    pub profile: &'a mut WorldMainLoopProfileState,
    pub copy_number_timer: &'a mut CopyNumberTimerState,
    pub process_message: &'a mut WorldProcessMessageStageState,
    pub refresh_high_water: &'a mut WorldRefreshInfoHighWater,
    pub collect_player_data: &'a mut WorldCollectPlayerDataRequestState,
    pub save_trigger: &'a mut WorldRunSaveTriggerState,
    pub save_lifecycle: &'a Arc<Mutex<SaveDataLifecycleState>>,
    pub reload_flags: &'a WorldReloadProfileFlags,
    pub player_ranks_request: &'a WorldPlayerRanksRequestState,
    pub save_thread_handle: &'a mut WorldSaveThreadHandleState,
}

pub struct WorldMainLoopOwners<
    'a,
    TimerCallback,
    LeiTingContextOwner,
    DbMiscContextOwner,
    JjcContext,
    RsPlayer,
> {
    pub resources: &'a mut dyn WorldMainLoopResourceContext,
    pub load_player_largess: &'a mut dyn FnMut(&mut CPlayer),
    pub organizing: &'a mut COrganizingCtrl,
    pub country: &'a mut CCountryHandler,
    pub country_parameters: &'a mut CCountryParam,
    pub time_to_return: &'a mut TimeToReturn,
    pub time_to_return_callbacks: TimeToReturnCallbacks<TimerCallback>,
    pub country_war: &'a mut CountryWarSys,
    pub country_war_callbacks: CountryWarCallbacks<TimerCallback>,
    pub four_nation_war: &'a mut CFourNationWarSys,
    pub four_nation_war_callbacks: FourNationWarCallbacks<TimerCallback>,
    pub honor_ranks: &'a mut CHonorRanks,
    pub organizing_parameters: &'a mut COrganizingParam,
    pub organizing_tax_callback: TimerCallback,
    pub player_ranks: &'a mut CPlayerRanks,
    pub rs_player: &'a mut RsPlayer,
    pub player_database: Option<&'a mut WorldTdsClient>,
    pub general_variables: Option<&'a mut CVariableList>,
    pub gods_battle: &'a mut CGodsBattleConf,
    pub skills: &'a mut CSkillFactory,
    pub rs_gods_battle: Option<&'a mut TiberiusRsGodsBattle>,
    pub gods_battle_database: Option<&'a mut WorldTdsClient>,
    pub auction_log: &'a mut CAuctionLog,
    pub auction_log_database: Option<&'a mut WorldTdsClient>,
    pub session_factory: &'a mut CSessionFactory,
    pub increment_log: &'a mut CIncrementLog,
    pub timer: &'a mut CTimer<TimerCallback>,
    pub faction_war: &'a mut CFactionWarSys,
    pub attack_city: &'a mut CAttackCitySys,
    pub attack_city_callbacks: AttackCityCallbacks<TimerCallback>,
    pub village_war: &'a mut CVillageWarSys,
    pub goods_war: &'a mut CGoodsWarMember,
    pub village_war_callbacks: VillageWarCallbacks<TimerCallback>,
    pub lei_ting: &'a mut CLeiTing,
    pub lei_ting_reset_worker: &'a WorldLeiTingResetWorker,
    pub tokio_runtime: tokio::runtime::Handle,
    pub db_misc: &'a mut CDbMisc,
    pub net_sessions: &'a CNetSessionManager,
    pub union_application_runtime: &'a WorldOrganizingSessionRuntimeOwner,
    pub jjc: &'a mut CJJcSystem,
    pub jjc_week_clear_worker: &'a WorldJjcWeekClearWorker,
    pub lei_ting_context: &'a mut LeiTingContextOwner,
    pub db_misc_context: &'a mut DbMiscContextOwner,
    pub jjc_context: &'a mut JjcContext,
    pub log: &'a mut WorldLogTextOwner,
}

pub struct WorldMainLoopCallbacks<'a, Game, Largess> {
    pub get_tick: &'a mut dyn FnMut() -> u32,
    pub get_save_point_time: &'a mut dyn FnMut() -> u32,
    pub save_runtime: &'a mut dyn WorldSaveRuntimeContext,
    pub get_log_local_time:
        &'a mut dyn FnMut() -> crate::app::worldserver::WorldLogLocalTime,
    pub put_log_info: &'a mut dyn FnMut(&[u8]),
    pub get_auction_month_day: &'a mut dyn FnMut() -> i32,
    pub largess: &'a Largess,
 /// Общий producer исходного `CWriteLogQueue`; faction-owner-ы ставят
 /// typed записи в тот же FIFO непосредственно в своих точках вызова.
    pub write_log_queue: WorldWriteLogQueue,
    pub random: &'a mut dyn FnMut(i32) -> i32,
    pub get_timer_local_time: &'a mut dyn FnMut() -> TagTime,
    pub refresh_union_owned_city:
        &'a mut dyn FnMut(&Game, i32, i32, i32, Option<u8>),
    pub update_union_player: &'a mut dyn FnMut(i32),
    pub get_lei_ting_local_time: &'a mut dyn FnMut() -> LeiTingLocalTime,
    pub wait: &'a mut dyn FnMut(u32),
    pub output_debug: &'a mut dyn FnMut(&'static str),
}

pub enum WorldMainLoopBlock<LeiTingContextBlock> {
    Largess(WorldMainLoopLargessGateReport),
    Refresh(WorldMainLoopRefreshStageReport),
    Reload(WorldReloadProfilesReport),
    Maintenance(WorldMainLoopMaintenanceBlock),
    MissingCountryLimit {
        parameter: &'static str,
    },
    SaveAllOrganizations {
        block: OrganizingSaveDataBlock,
    },
    ImmediateSave {
        log: AddLogTextDisposition,
        block: WorldGenerateDbDataBlock,
    },
    ProcessMessage(WorldProcessMessageStageReport),
    PlayerDataQueue(WorldMainLoopPlayerDataQueueStageReport),
    Timer(WorldMainLoopTimerStageBlock),
    FactionWar(FactionWarStopBlock<WorldMainLoopFactionWarBlock>),
    LeiTing(LeiTingBlock<LeiTingContextBlock, PlayerCodecError>),
    DbMisc(DbMiscDoneOutBlock),
    Ping(WorldMainLoopPingError),
    Minute(WorldMainLoopMinuteStageBlock),
    Jjc(JjcRunBlock),
    Tail(WorldMainLoopPacingReport),
}

pub type WorldMainLoopResult<LeiTingContextBlock> =
    Result<WorldMainLoopReport, Box<WorldMainLoopBlock<LeiTingContextBlock>>>;

#[derive(Debug)]
pub struct WorldMainLoopReport {
    pub profile_initialization: Option<WorldMainLoopProfileInitialization>,
    pub save_initialization: Option<WorldMainLoopSaveInitialization>,
    pub refresh_initialization: Option<WorldMainLoopRefreshInitialization>,
    pub current_tick_ms: u32,
    pub largess: WorldMainLoopLargessGateReport,
    pub refresh_profile_started_at_ms: u32,
    pub refresh: WorldMainLoopRefreshStageReport,
    pub reload: WorldReloadProfilesReport,
    pub maintenance: WorldMainLoopMaintenanceReport,
    pub collect_player_data: Option<WorldCollectPlayerDataBroadcast>,
    pub save: WorldMainLoopSaveStageReport,
    pub ai: WorldMainLoopAiStageReport,
    pub process_message: WorldProcessMessageStageReport,
    pub session_factory: WorldMainLoopSessionFactoryStageReport,
    pub player_data_queue: WorldMainLoopPlayerDataQueueStageReport,
    pub timer: WorldMainLoopTimerStageReport,
    pub faction_war: WorldMainLoopFactionWarStageReport,
    pub lei_ting: LeiTingRunReport,
    pub db_misc: WorldMainLoopDbMiscStageReport,
    pub net_sessions: WorldMainLoopNetSessionStageReport,
    pub ping: WorldMainLoopPingStageReport,
    pub minute: WorldMainLoopMinuteStageReport,
    pub bai_tan_jjc: WorldMainLoopBaiTanJjcStageReport,
    pub tail: WorldMainLoopTailStageReport,
    pub legacy_result: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldMainLoopPingStageReport {
    Idle,
    Waiting {
        connected_game_servers: i32,
        received_responses: u32,
        elapsed_ms: u32,
    },
    Published {
        connected_game_servers: i32,
        received_responses: u32,
        elapsed_ms: u32,
        all_connected_responded: bool,
        timed_out: bool,
        online_players: u32,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldMainLoopPingError {
    ResponseCountOutsideLegacyRange { count: usize },
    OnlinePlayerCountOutsideLegacyRange { count: usize },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldRefreshExternalCounts {
    pub team_sessions: i32,
    pub largess_entries: u32,
    pub reback_messages: i32,
}
