//! Process owners исторического WorldServer, перенесённые из
//! `src/worldserver/worldserver/runtime.rs` в Realm `app/` волной C5-D:
//! долгоживущие domain owners и их `initialize_game`, process-состояние
//! MainLoop, network-обвязка хода, post-init DB-контексты и сам
//! `WorldProcessRuntime` с `run_connected_main_loop`.
//!
//! Источник контракта — та же точная пара, что у [`crate::app::world_runtime`]
//! (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`,
//! RSDS совпадает). Это рантайм-проводка: основания Init/MainLoop/Release
//! сверены прежними волнами (см. `world_game_init`, `world_main_loop`,
//! `world_runtime`); сам перенос новой машинной сверки не требует, тела
//! перенесены буквально.
//!
//! Состав соседей: Init-контекст процесса — [`crate::app::world_process_init`],
//! save-worker — [`crate::app::world_process_save`], ресурсный держатель —
//! [`crate::app::world_process_resources`], release-контекст и impl драйвера —
//! [`crate::app::world_process_release`]. JJC/LeiTing process-контексты и
//! ошибка их сборки уже в [`crate::app::world_main_loop_contexts`] (волна
//! C5-B), realm-shaped состав Init — в [`crate::app::world_init_context`]
//! (волна C5-B), сам `CGame` — в [`crate::app::world_game`] (волна C5-C).
//!
//! StringTable и встроенные setup-владельцы остаются у единственного `CGame`;
//! здесь собраны исторические process-global owners, передаваемые ему
//! ссылками.

use std::convert::Infallible;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{Datelike, Timelike};
use parking_lot::Mutex;

use nebokrai_shared::resources::{CGodsBattleConf, GlobeSetupSnapshot};
use nebokrai_shared::runtime::{CNetSessionManager, CTimer, NetSessionManagerVariant};
use nebokrai_shared::values::TagTime;

use crate::activities::attackcitysys::{AttackCityCallbacks, CAttackCitySys};
use crate::activities::countrywarsys::{CountryWarCallbacks, CountryWarSys};
use crate::activities::factionwarsys::CFactionWarSys;
use crate::activities::fournationwarsys::{CFourNationWarSys, FourNationWarCallbacks};
use crate::activities::jjcmaintenanceworker::WorldJjcWeekClearWorker;
use crate::activities::jjcsystem::CJJcSystem;
use crate::activities::leiting::CLeiTing;
use crate::activities::leitingresetworker::WorldLeiTingResetWorker;
use crate::activities::misc::CopyNumberTimerState;
use crate::activities::villagewarsys::{CVillageWarSys, VillageWarCallbacks};
use crate::app::misc_game::legacy_tick_ms;
use crate::app::organsysmessage::WorldOrganizingSessionRuntimeOwner as WorldUnionApplicationRuntimeOwner;
use crate::app::world_client::CMyNetClient;
use crate::app::world_game::{CGame, WorldOwnedCityRefreshOutcome};
use crate::app::world_hub_data::{
    WorldGameInitCallbacks, WorldMainLoopClockState, WorldMainLoopInitializationState,
    WorldMainLoopLargessState, WorldMainLoopLoginReleaseState, WorldMainLoopProfileState,
    WorldMainLoopTailClockState, WorldProcessMessageStageState,
};
use crate::app::world_init_context::WorldDbMiscProcessConfiguration;
use crate::app::world_main_loop_contexts::{
    WorldJjcProcessContext, WorldLeiTingProcessContext, WorldMainLoopContextBuildError,
    WorldPlatformTimeError, current_lei_ting_local_time,
};
use crate::app::world_main_loop_data::{
    WorldMainLoopBlock, WorldMainLoopCallbacks, WorldMainLoopConfiguration,
    WorldMainLoopOwners, WorldMainLoopStateOwners, WorldRefreshExternalCounts,
};
use crate::app::world_network::{
    WorldProcessNetworkError, WorldProcessNetworkRuntime as RealmProcessNetworkRuntime,
    WorldProcessNetworkTurn, report_world_network_turn,
};
use crate::app::world_process_init::{WorldProcessInitContext, world_player_load_largess};
use crate::app::world_process_release::WorldProcessReleaseContext;
use crate::app::world_process_resources::WorldProcessResources;
use crate::app::world_process_save::WorldProcessSaveRuntime;
use crate::app::world_reload_profiles::{WorldPlayerRanksRequestState, WorldReloadProfileFlags};
use crate::app::world_runtime::WorldGameInitResult;
use crate::app::world_save_reports::{
    WorldCollectPlayerDataRequestState, WorldRunSaveTriggerState,
};
use crate::app::world_server::CMyNetServer;
use crate::app::worldserver::{
    WorldLogLocalTime, WorldLogTextOwner, WorldRefreshInfoHighWater,
    WorldSaveThreadHandleState,
};
use crate::auction::auctionlog::CAuctionLog;
use crate::billing::incrementlog::CIncrementLog;
use crate::characters::honorranks::CHonorRanks;
use crate::characters::player::CPlayer;
use crate::characters::playerranks::CPlayerRanks;
use crate::content::countryparam::CCountryParam;
use crate::content::skillfactory::CSkillFactory;
use crate::content::variablelist::CVariableList;
use crate::content::{TimeToReturn, TimeToReturnCallbacks};
use crate::organizations::countryhandler::CCountryHandler;
use crate::organizations::goodswarmember::CGoodsWarMember;
use crate::organizations::organizingctrl::COrganizingCtrl;
use crate::organizations::organizingparam::COrganizingParam;
use crate::persistence::dbmisc::{CDbMisc, TiberiusDbMiscContext};
use crate::persistence::savedb::{SaveDataLifecycleState, SaveDataLocalTime};
use crate::sessions::csessionfactory::CSessionFactory;

/// Стабильные typed-ключи всех callback-ов единственного World timer-owner-а.
///
/// В EXE это разные адреса функций. Rust хранит их как значения одного enum,
/// чтобы календарные записи всех подсистем находились в общем `CTimer`, но не
/// превращались в нетипизированные числовые адреса или отдельные registries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldTimerCallback {
    TimeToReturn,
    CopyNumberReset,
    OrganizingTax,
    PlayerRanks,
    AttackCityDeclare,
    AttackCityStartInfo,
    AttackCityStart,
    AttackCityEndInfo,
    AttackCityEnd,
    AttackCityMass,
    AttackCityClearOtherPlayer,
    AttackCityRefreshRegion,
    FourNationSignUpStart,
    FourNationSignUpEnd,
    FourNationWarStart,
    FourNationWarEnd,
    FourNationWarEndInfo,
    FourNationEnterStart,
    FourNationEnterEnd,
    FourNationRefreshRegion,
    FourNationClearWar,
    VillageWarDeclare,
    VillageWarStartInfo,
    VillageWarStart,
    VillageWarEndInfo,
    VillageWarEnd,
    VillageWarClearPlayer,
    CountryWarClear,
    CountryWarDeclareBegin,
    CountryWarDeclareEnd,
    CountryWarPrepareBegin,
    CountryWarPrepareEnd,
    CountryWarStart,
    CountryWarEnd,
    CountryWarStartInfo,
    CountryWarEndInfo,
}

#[derive(Clone, Copy, Debug)]
pub struct WorldTimerCallbacks {
    pub(crate) time_to_return: TimeToReturnCallbacks<WorldTimerCallback>,
    pub(crate) attack_city: AttackCityCallbacks<WorldTimerCallback>,
    pub(crate) four_nation_war: FourNationWarCallbacks<WorldTimerCallback>,
    pub(crate) village_war: VillageWarCallbacks<WorldTimerCallback>,
    pub(crate) country_war: CountryWarCallbacks<WorldTimerCallback>,
}

impl WorldTimerCallbacks {
    pub(crate) const fn new() -> Self {
        Self {
            time_to_return: TimeToReturnCallbacks {
                on_time: WorldTimerCallback::TimeToReturn,
            },
            attack_city: AttackCityCallbacks {
                declare: WorldTimerCallback::AttackCityDeclare,
                start_info: WorldTimerCallback::AttackCityStartInfo,
                start: WorldTimerCallback::AttackCityStart,
                end_info: WorldTimerCallback::AttackCityEndInfo,
                end: WorldTimerCallback::AttackCityEnd,
                mass: WorldTimerCallback::AttackCityMass,
                clear_other_player: WorldTimerCallback::AttackCityClearOtherPlayer,
                refresh_region: WorldTimerCallback::AttackCityRefreshRegion,
            },
            four_nation_war: FourNationWarCallbacks {
                sign_up_start: WorldTimerCallback::FourNationSignUpStart,
                sign_up_end: WorldTimerCallback::FourNationSignUpEnd,
                war_start: WorldTimerCallback::FourNationWarStart,
                war_end: WorldTimerCallback::FourNationWarEnd,
                war_end_info: WorldTimerCallback::FourNationWarEndInfo,
                enter_start: WorldTimerCallback::FourNationEnterStart,
                enter_end: WorldTimerCallback::FourNationEnterEnd,
                refresh_region: WorldTimerCallback::FourNationRefreshRegion,
                clear_war: WorldTimerCallback::FourNationClearWar,
            },
            village_war: VillageWarCallbacks {
                declare: WorldTimerCallback::VillageWarDeclare,
                start_info: WorldTimerCallback::VillageWarStartInfo,
                start: WorldTimerCallback::VillageWarStart,
                end_info: WorldTimerCallback::VillageWarEndInfo,
                end: WorldTimerCallback::VillageWarEnd,
                clear_player: WorldTimerCallback::VillageWarClearPlayer,
            },
            country_war: CountryWarCallbacks {
                clear: WorldTimerCallback::CountryWarClear,
                declare_begin: WorldTimerCallback::CountryWarDeclareBegin,
                declare_end: WorldTimerCallback::CountryWarDeclareEnd,
                prepare_begin: WorldTimerCallback::CountryWarPrepareBegin,
                prepare_end: WorldTimerCallback::CountryWarPrepareEnd,
                start: WorldTimerCallback::CountryWarStart,
                end: WorldTimerCallback::CountryWarEnd,
                start_info: WorldTimerCallback::CountryWarStartInfo,
                end_info: WorldTimerCallback::CountryWarEndInfo,
            },
        }
    }
}

/// Долгоживущие domain owners единственного World process.
///
/// Они создаются один раз до `CGame::Init`, затем те же экземпляры проходят
/// MainLoop и Release. Это заменяет process-global singleton pointers, не
/// создавая второй module graph или копии игровых типов.
pub struct WorldProcessDomainOwners {
    pub(crate) callbacks: WorldTimerCallbacks,
    pub(crate) jjc: CJJcSystem,
    pub(crate) gods_battle: CGodsBattleConf,
    pub(crate) skills: CSkillFactory,
    pub(crate) time_to_return: TimeToReturn,
    pub(crate) general_variables: Option<CVariableList>,
    pub(crate) organizing_parameters: COrganizingParam,
    pub(crate) attack_city: CAttackCitySys,
    pub(crate) four_nation_war: CFourNationWarSys,
    pub(crate) village_war: CVillageWarSys,
    pub(crate) faction_war: CFactionWarSys,
    pub(crate) player_ranks: CPlayerRanks,
    pub(crate) timer: CTimer<WorldTimerCallback>,
    pub(crate) copy_number_timer: CopyNumberTimerState,
    pub(crate) organizing: COrganizingCtrl,
    pub(crate) country_handler: CCountryHandler,
    pub(crate) country_parameters: CCountryParam,
    pub(crate) goods_war: CGoodsWarMember,
    pub(crate) country_war: CountryWarSys,
    pub(crate) honor_ranks: CHonorRanks,
    pub(crate) increment_log: CIncrementLog,
    pub(crate) auction_log: CAuctionLog,
    pub(crate) db_misc: CDbMisc,
    pub(crate) session_factory: CSessionFactory,
    pub(crate) net_sessions: CNetSessionManager,
    pub(crate) union_application_runtime: WorldUnionApplicationRuntimeOwner,
    pub(crate) log: WorldLogTextOwner,
}

impl WorldProcessDomainOwners {
    pub(crate) fn new(start_tick_ms: u32) -> Self {
        Self {
            callbacks: WorldTimerCallbacks::new(),
            jjc: CJJcSystem::new(),
            gods_battle: Default::default(),
            skills: Default::default(),
            time_to_return: TimeToReturn::new(),
            general_variables: None,
            organizing_parameters: Default::default(),
            attack_city: CAttackCitySys::new(),
            four_nation_war: Default::default(),
            village_war: CVillageWarSys::new(),
            faction_war: CFactionWarSys::new(start_tick_ms),
            player_ranks: Default::default(),
            timer: CTimer::new(),
            copy_number_timer: Default::default(),
            organizing: COrganizingCtrl::with_reached_callback_state(),
            country_handler: Default::default(),
            country_parameters: CCountryParam::new(),
            goods_war: CGoodsWarMember::with_reached_empty_state(),
            country_war: Default::default(),
            honor_ranks: Default::default(),
            increment_log: CIncrementLog::new(),
            auction_log: Default::default(),
            db_misc: CDbMisc::with_empty_queues(),
            session_factory: CSessionFactory::new(),
            net_sessions: CNetSessionManager::new(NetSessionManagerVariant::WorldServer),
            union_application_runtime: WorldUnionApplicationRuntimeOwner::default(),
            log: Default::default(),
        }
    }

    pub(crate) async fn initialize_game(
        &mut self,
        game: &mut CGame,
        runtime_directory: &Path,
        context: &mut WorldProcessInitContext,
        resources: &mut WorldProcessResources,
    ) -> WorldGameInitResult<Infallible> {
        let started_at = context.started_at();
        let mut get_tick = move || started_at.elapsed().as_millis() as u32;
        let mut get_log_local_time = || {
            let now = chrono::Local::now();
            WorldLogLocalTime {
                year: now.year() as u16,
                month: now.month() as u16,
                day: now.day() as u16,
                hour: now.hour() as u16,
                minute: now.minute() as u16,
                second: now.second() as u16,
            }
        };
        let mut get_timer_local_time = TagTime::local_now;
        let mut put_log_info = |payload: &[u8]| {
            eprintln!("WorldServer: {}", String::from_utf8_lossy(payload));
        };
        let mut callbacks = WorldGameInitCallbacks {
            get_tick: &mut get_tick,
            get_log_local_time: &mut get_log_local_time,
            get_timer_local_time: &mut get_timer_local_time,
            put_log_info: &mut put_log_info,
        };
        game.init(
            runtime_directory,
            context,
            resources,
            &mut self.jjc,
            &mut self.gods_battle,
            &mut self.skills,
            &mut self.time_to_return,
            self.callbacks.time_to_return,
            &mut self.general_variables,
            &mut self.organizing_parameters,
            &mut self.attack_city,
            self.callbacks.attack_city,
            &mut self.four_nation_war,
            self.callbacks.four_nation_war,
            &mut self.village_war,
            self.callbacks.village_war,
            &mut self.faction_war,
            &mut self.player_ranks,
            &mut self.timer,
            &mut self.copy_number_timer,
            WorldTimerCallback::CopyNumberReset,
            WorldTimerCallback::OrganizingTax,
            WorldTimerCallback::PlayerRanks,
            &mut self.organizing,
            &mut self.country_handler,
            &mut self.country_parameters,
            &mut self.goods_war,
            &mut self.country_war,
            self.callbacks.country_war,
            &mut self.honor_ranks,
            &mut self.increment_log,
            &mut self.auction_log,
            &mut self.log,
            &mut callbacks,
        )
        .await
    }
}

pub struct WorldProcessMainLoopState {
    initialization: WorldMainLoopInitializationState,
    clocks: WorldMainLoopClockState,
    tail_clocks: WorldMainLoopTailClockState,
    login_release: WorldMainLoopLoginReleaseState,
    largess: WorldMainLoopLargessState,
    profile: WorldMainLoopProfileState,
    process_message: WorldProcessMessageStageState,
    refresh_high_water: WorldRefreshInfoHighWater,
    collect_player_data: WorldCollectPlayerDataRequestState,
    save_trigger: WorldRunSaveTriggerState,
    save_lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
    reload_flags: WorldReloadProfileFlags,
    player_ranks_request: WorldPlayerRanksRequestState,
    save_thread_handle: WorldSaveThreadHandleState,
}

impl WorldProcessMainLoopState {
    pub(crate) fn new() -> Self {
        Self {
            initialization: Default::default(),
            clocks: Default::default(),
            tail_clocks: Default::default(),
            login_release: Default::default(),
            largess: Default::default(),
            profile: WorldMainLoopProfileState {
                last_published_at_ms: 0,
                ai_calls: 0,
                ai_time_ms: 0,
                refresh_text_time_ms: 0,
                net_session_time_ms: 0,
                faction_war_time_ms: 0,
                timer_time_ms: 0,
                process_player_data_queue_time_ms: 0,
                session_factory_time_ms: 0,
                save_point_time_ms: 0,
            },
            process_message: Default::default(),
            refresh_high_water: Default::default(),
            collect_player_data: Default::default(),
            save_trigger: WorldRunSaveTriggerState {
                send_save_message_now: false,
                save_all_organizations: false,
                save_now_data: false,
                last_save_point_time_ms: 0,
            },
            save_lifecycle: Arc::new(Mutex::new(SaveDataLifecycleState {
                this_save_start_tick_ms: 0,
                last_save_tick_ms: 0,
                last_save_time: SaveDataLocalTime {
                    year: 0,
                    month: 0,
                    day_of_week: 0,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    milliseconds: 0,
                },
                is_saving_data: false,
            })),
            reload_flags: Default::default(),
            player_ranks_request: Default::default(),
            save_thread_handle: WorldSaveThreadHandleState::Empty,
        }
    }

    pub(crate) fn owners<'a>(
        &'a mut self,
        copy_number_timer: &'a mut CopyNumberTimerState,
    ) -> WorldMainLoopStateOwners<'a> {
        WorldMainLoopStateOwners {
            initialization: &mut self.initialization,
            clocks: &mut self.clocks,
            tail_clocks: &mut self.tail_clocks,
            login_release: &mut self.login_release,
            largess: &mut self.largess,
            profile: &mut self.profile,
            copy_number_timer,
            process_message: &mut self.process_message,
            refresh_high_water: &mut self.refresh_high_water,
            collect_player_data: &mut self.collect_player_data,
            save_trigger: &mut self.save_trigger,
            save_lifecycle: &self.save_lifecycle,
            reload_flags: &self.reload_flags,
            player_ranks_request: &self.player_ranks_request,
            save_thread_handle: &mut self.save_thread_handle,
        }
    }
}

/// Process-runtime обвязка Realm `app::world_network`: два mut-accessor-а
/// `CGame` не вызываются одновременно, поэтому владельцы направлений
/// подаются стадиями, а ход собирается в исходном порядке — сначала
/// worker принятых GameServer, затем неблокирующий опрос Login.
pub struct WorldProcessNetworkRuntime {
    inner: RealmProcessNetworkRuntime,
}

impl WorldProcessNetworkRuntime {
    pub(crate) fn new() -> Self {
        Self {
            inner: RealmProcessNetworkRuntime::new(),
        }
    }

    pub(crate) async fn run_turn(
        &mut self,
        game: &mut CGame,
    ) -> Result<WorldProcessNetworkTurn, WorldProcessNetworkError> {
        self.inner
            .run_game_server_worker(game.process_game_server_mut(), legacy_tick_ms)
            .await?;
        let mut turn = WorldProcessNetworkTurn::default();
        turn.login = RealmProcessNetworkRuntime::poll_login_client(
            game.process_login_client_mut(),
            legacy_tick_ms,
        )
        .await;
        Ok(turn)
    }

    pub(crate) async fn release_server(
        &mut self,
        server: &mut CMyNetServer,
    ) -> Result<(), WorldProcessNetworkError> {
        self.inner.release_server(server, legacy_tick_ms).await
    }

    pub(crate) fn release_client(&mut self, client: &mut CMyNetClient) {
        self.inner.release_client(client);
    }
}

/// Долгоживущие concrete контексты трёх последовательных MainLoop DB-stage.
/// Все они строятся только после успешного `CGame::Init` из тех же setup,
/// resource, transport и FIFO owners, которые Init уже опубликовал.
pub struct WorldProcessMainLoopContexts {
    pub(crate) db_misc: TiberiusDbMiscContext,
 /// Держатель publish-точки для reload-конфигурации DbMisc: читается только
 /// копией в `WorldProcessResources`, эта сохраняет состав контекстов как в
 /// исходном process owner-е (общее состояние разделяют Arc-ы callbacks).
    #[allow(dead_code, reason = "keep-alive publish-точки DbMisc; активная копия — у ресурсов, см. world_process_resources")]
    pub(crate) db_misc_configuration: WorldDbMiscProcessConfiguration,
    pub(crate) jjc_worker: Arc<WorldJjcWeekClearWorker>,
    pub(crate) jjc: WorldJjcProcessContext,
    pub(crate) lei_ting_worker: Arc<WorldLeiTingResetWorker>,
    pub(crate) lei_ting: WorldLeiTingProcessContext,
    pub(crate) lei_ting_owner: CLeiTing,
    pub(crate) load_player_largess: Box<dyn FnMut(&mut CPlayer) + Send>,
}

impl WorldProcessMainLoopContexts {
    pub(crate) fn after_game_init(
        runtime: tokio::runtime::Handle,
        game: &CGame,
        init: &mut WorldProcessInitContext,
        resources: &mut WorldProcessResources,
        domains: &WorldProcessDomainOwners,
    ) -> Result<Self, WorldMainLoopContextBuildError> {
        let settings = init
            .database_settings()
            .ok_or(WorldMainLoopContextBuildError::MissingDatabaseSettings)?;
        let snapshot = resources.player_load_snapshot().read().clone();
        let largess = init
            .largess()
            .ok_or(WorldMainLoopContextBuildError::MissingLargessOwner)?;
        let (db_misc, db_misc_configuration) = init
            .take_db_misc_context(
                snapshot.goods.as_ref().clone(),
                snapshot.globe_setup.clone(),
                snapshot.gold_coin_index,
                domains.db_misc.output_publisher(),
                Box::new(|globe_setup: &GlobeSetupSnapshot, seller_money| {
                    CGame::get_opt_money_jin(globe_setup, Some(seller_money))
                        .map(|money| money.seller_money_after_fee)
                }),
            )
            .ok_or(WorldMainLoopContextBuildError::MissingDbMiscOwner)?;
        resources.install_db_misc_configuration(db_misc_configuration.clone());
        let started_at = init.started_at();
        let save_info_time_ms = game.save_info_time_ms();
        let log = domains.log.clone();

        let jjc_worker = Arc::new(WorldJjcWeekClearWorker::new(settings.clone()));
        let jjc = WorldJjcProcessContext::new(
            runtime.clone(),
            started_at,
            resources.runtime_directory(),
            &settings,
            Arc::clone(&jjc_worker),
            log.clone(),
            save_info_time_ms,
        );
        let lei_ting_worker = Arc::new(WorldLeiTingResetWorker::new(
            settings,
            Arc::new(snapshot.thing_setup),
            snapshot.globe_setup,
        ));
        let lei_ting = WorldLeiTingProcessContext::new(
            runtime,
            game.current_game_server_sender(),
            Arc::clone(&lei_ting_worker),
            log,
            started_at,
            save_info_time_ms,
        );

        Ok(Self {
            db_misc,
            db_misc_configuration,
            jjc_worker,
            jjc,
            lei_ting_worker,
            lei_ting,
            lei_ting_owner: CLeiTing::new(current_lei_ting_local_time()),
            load_player_largess: world_player_load_largess(
                largess,
                resources.player_load_snapshot(),
            ),
        })
    }
}

#[derive(Clone)]
pub struct WorldProcessControl {
    exit_requested: Arc<AtomicBool>,
}

impl WorldProcessControl {
    pub fn request_exit(&self) {
        self.exit_requested.store(true, Ordering::Release);
    }
}

pub enum WorldProcessMainLoopBlock {
    PostInit(WorldMainLoopContextBuildError),
    MissingOwner(&'static str),
    TeamSessionCountOutsideLegacyRange { count: usize },
    LargessCountOutsideLegacyRange { count: usize },
    Network(WorldProcessNetworkError),
    Game(Box<WorldMainLoopBlock<WorldPlatformTimeError>>),
}

impl fmt::Display for WorldProcessMainLoopBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PostInit(error) => write!(formatter, "владельцы после Init не собраны: {error}"),
            Self::MissingOwner(owner) => write!(formatter, "после Init отсутствует {owner}"),
            Self::TeamSessionCountOutsideLegacyRange { count } => write!(
                formatter,
                "размер World team-session map {count} не помещается в signed long"
            ),
            Self::LargessCountOutsideLegacyRange { count } => write!(
                formatter,
                "размер World Largess map {count} не помещается в unsigned long"
            ),
            Self::Network(error) => write!(formatter, "World transport остановлен: {error}"),
            Self::Game(block) => {
                let owner = match block.as_ref() {
                    WorldMainLoopBlock::Largess(_) => "Largess",
                    WorldMainLoopBlock::Refresh(_) => "RefeashInfoText",
                    WorldMainLoopBlock::Reload(_) => "reload",
                    WorldMainLoopBlock::Maintenance(_) => "maintenance",
                    WorldMainLoopBlock::MissingCountryLimit { parameter } => parameter,
                    WorldMainLoopBlock::SaveAllOrganizations { .. } => "SaveAllOrganizations",
                    WorldMainLoopBlock::ImmediateSave { .. } => "immediate save",
                    WorldMainLoopBlock::ProcessMessage(_) => "ProcessMessage",
                    WorldMainLoopBlock::PlayerDataQueue(_) => "player-data queue",
                    WorldMainLoopBlock::Timer(_) => "timer",
                    WorldMainLoopBlock::FactionWar(_) => "FactionWar",
                    WorldMainLoopBlock::LeiTing(_) => "LeiTing",
                    WorldMainLoopBlock::DbMisc(_) => "DbMisc",
                    WorldMainLoopBlock::Ping(_) => "ping",
                    WorldMainLoopBlock::Minute(_) => "minute lifecycle",
                    WorldMainLoopBlock::Jjc(_) => "JJC",
                    WorldMainLoopBlock::Tail(_) => "main-loop pacing",
                };
                write!(formatter, "CGame::MainLoop остановлен на участке {owner}")
            }
        }
    }
}

impl fmt::Debug for WorldProcessMainLoopBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl Error for WorldProcessMainLoopBlock {}

pub struct WorldProcessRuntime {
    pub(crate) runtime: tokio::runtime::Handle,
    pub(crate) runtime_directory: PathBuf,
    pub(crate) domains: WorldProcessDomainOwners,
    pub(crate) init: WorldProcessInitContext,
    pub(crate) resources: WorldProcessResources,
    pub(crate) network: WorldProcessNetworkRuntime,
    main_loop_state: WorldProcessMainLoopState,
    pub(crate) contexts: Option<WorldProcessMainLoopContexts>,
    pub(crate) save: Option<WorldProcessSaveRuntime>,
    pub(crate) post_init_error: Option<WorldMainLoopContextBuildError>,
    pub(crate) exit_requested: Arc<AtomicBool>,
    pub(crate) game_thread_exited: bool,
    pub(crate) window_close_requested: bool,
 /// Статический `ReMsg` исходно заполнен нулями; в точном EXE найден только reader,
 /// поэтому owner сохраняется отдельно и остаётся нулём до появления
 /// исходного producer-а.
    reback_messages: i32,
}

impl WorldProcessRuntime {
    pub fn new(
        runtime: tokio::runtime::Handle,
        runtime_directory: PathBuf,
    ) -> (Self, WorldProcessControl) {
        let mut resources = WorldProcessResources::new(runtime_directory.clone());
        resources.install_default_client_resource();
        let init = WorldProcessInitContext::new(runtime.clone(), resources.player_load_snapshot());
        let exit_requested = Arc::new(AtomicBool::new(false));
        let control = WorldProcessControl {
            exit_requested: Arc::clone(&exit_requested),
        };
        (
            Self {
                runtime,
                runtime_directory,
                domains: WorldProcessDomainOwners::new(0),
                init,
                resources,
                network: WorldProcessNetworkRuntime::new(),
                main_loop_state: WorldProcessMainLoopState::new(),
                contexts: None,
                save: None,
                post_init_error: None,
                exit_requested,
                game_thread_exited: false,
                window_close_requested: false,
                reback_messages: 0,
            },
            control,
        )
    }

    pub(crate) fn release_context(&mut self) -> WorldProcessReleaseContext<'_> {
        WorldProcessReleaseContext::new(
            self.runtime.clone(),
            &self.runtime_directory,
            &mut self.network,
            &mut self.domains,
            &mut self.init,
            &mut self.resources,
            self.save.as_mut(),
        )
    }

    pub(crate) async fn run_connected_main_loop(
        &mut self,
        game: &mut CGame,
    ) -> Result<i32, WorldProcessMainLoopBlock> {
        let network = self
            .network
            .run_turn(game)
            .await
            .map_err(WorldProcessMainLoopBlock::Network)?;
        report_world_network_turn(&network);

        let contexts = self
            .contexts
            .as_mut()
            .ok_or(WorldProcessMainLoopBlock::MissingOwner("MainLoop DB/context owners"))?;
        let save = self
            .save
            .as_mut()
            .ok_or(WorldProcessMainLoopBlock::MissingOwner("save runtime"))?;
        let largess = self
            .init
            .largess()
            .ok_or(WorldProcessMainLoopBlock::MissingOwner("Largess owner"))?;
        let team_sessions = game.team_session_count();
        let team_sessions = i32::try_from(team_sessions).map_err(|_| {
            WorldProcessMainLoopBlock::TeamSessionCountOutsideLegacyRange {
                count: team_sessions,
            }
        })?;
        let largess_entries = largess.entry_count();
        let largess_entries = u32::try_from(largess_entries).map_err(|_| {
            WorldProcessMainLoopBlock::LargessCountOutsideLegacyRange {
                count: largess_entries,
            }
        })?;
        let configuration = WorldMainLoopConfiguration {
            refresh_external_counts: WorldRefreshExternalCounts {
                team_sessions,
                largess_entries,
                reback_messages: self.reback_messages,
            },
        };

        let started_at = self.init.started_at();
        let player_load_snapshot = self.resources.player_load_snapshot();
        let mut get_tick = move || started_at.elapsed().as_millis() as u32;
        let mut get_save_point_time = move || {
            player_load_snapshot.read().globe_setup.save_point_time_ms()
        };
        let mut get_log_local_time = || {
            let now = chrono::Local::now();
            WorldLogLocalTime {
                year: now.year() as u16,
                month: now.month() as u16,
                day: now.day() as u16,
                hour: now.hour() as u16,
                minute: now.minute() as u16,
                second: now.second() as u16,
            }
        };
        let mut put_log_info = |payload: &[u8]| {
            eprintln!("WorldServer: {}", String::from_utf8_lossy(payload));
        };
        let mut get_auction_month_day = || chrono::Local::now().day() as i32;
        let random_state = &mut self.init.random_state;
        let mut random = |upper_bound: i32| {
            *random_state = random_state.wrapping_mul(214013).wrapping_add(2531011);
            let value = ((*random_state >> 16) & 0x7fff) as i32;
            if upper_bound > 0 { value % upper_bound } else { 0 }
        };
        let mut get_timer_local_time = TagTime::local_now;
        let mut refresh_union_owned_city =
            |game: &CGame, region_id, faction_id, union_id, country| {
                let outcome = game.refresh_owned_city_org_with_country(
                    region_id,
                    faction_id,
                    union_id,
                    country,
                );
                if !matches!(outcome, WorldOwnedCityRefreshOutcome::Refreshed(_)) {
                    eprintln!("WorldServer: city-owner {region_id} не обновлён: {outcome:?}");
                }
            };
        let mut observed_player_updates = Vec::new();
        let mut update_union_player = |player_id| observed_player_updates.push(player_id);
        let mut get_lei_ting_local_time = current_lei_ting_local_time;
        let mut wait = |milliseconds| {
            std::thread::sleep(std::time::Duration::from_millis(u64::from(milliseconds)));
        };
        let mut output_debug = |message: &'static str| eprintln!("WorldServer: {message}");

        let callbacks = self.domains.callbacks;
        let mut state = self
            .main_loop_state
            .owners(&mut self.domains.copy_number_timer);
        let mut owners = WorldMainLoopOwners {
            resources: &mut self.resources,
            load_player_largess: &mut *contexts.load_player_largess,
            organizing: &mut self.domains.organizing,
            country: &mut self.domains.country_handler,
            country_parameters: &mut self.domains.country_parameters,
            time_to_return: &mut self.domains.time_to_return,
            time_to_return_callbacks: callbacks.time_to_return,
            country_war: &mut self.domains.country_war,
            country_war_callbacks: callbacks.country_war,
            four_nation_war: &mut self.domains.four_nation_war,
            four_nation_war_callbacks: callbacks.four_nation_war,
            honor_ranks: &mut self.domains.honor_ranks,
            organizing_parameters: &mut self.domains.organizing_parameters,
            organizing_tax_callback: WorldTimerCallback::OrganizingTax,
            player_ranks: &mut self.domains.player_ranks,
            rs_player: self
                .init
                .player
                .as_mut()
                .ok_or(WorldProcessMainLoopBlock::MissingOwner("CRsPlayer"))?,
            player_database: self.init.player_connection.as_mut(),
            general_variables: self.domains.general_variables.as_mut(),
            gods_battle: &mut self.domains.gods_battle,
            skills: &mut self.domains.skills,
            rs_gods_battle: self.init.gods_battle.as_mut(),
            gods_battle_database: self.init.gods_battle_connection.as_mut(),
            auction_log: &mut self.domains.auction_log,
            auction_log_database: self.init.log_connection.as_mut(),
            session_factory: &mut self.domains.session_factory,
            increment_log: &mut self.domains.increment_log,
            timer: &mut self.domains.timer,
            faction_war: &mut self.domains.faction_war,
            attack_city: &mut self.domains.attack_city,
            attack_city_callbacks: callbacks.attack_city,
            village_war: &mut self.domains.village_war,
            goods_war: &mut self.domains.goods_war,
            village_war_callbacks: callbacks.village_war,
            lei_ting: &mut contexts.lei_ting_owner,
            lei_ting_reset_worker: &contexts.lei_ting_worker,
            tokio_runtime: self.runtime.clone(),
            db_misc: &mut self.domains.db_misc,
            net_sessions: &self.domains.net_sessions,
            union_application_runtime: &self.domains.union_application_runtime,
            jjc: &mut self.domains.jjc,
            jjc_week_clear_worker: &contexts.jjc_worker,
            lei_ting_context: &mut contexts.lei_ting,
            db_misc_context: &mut contexts.db_misc,
            jjc_context: &mut contexts.jjc,
            log: &mut self.domains.log,
        };
        let mut callbacks = WorldMainLoopCallbacks {
            get_tick: &mut get_tick,
            get_save_point_time: &mut get_save_point_time,
            save_runtime: save,
            get_log_local_time: &mut get_log_local_time,
            put_log_info: &mut put_log_info,
            get_auction_month_day: &mut get_auction_month_day,
            largess: &largess,
            write_log_queue: game.write_log_queue(),
            random: &mut random,
            get_timer_local_time: &mut get_timer_local_time,
            refresh_union_owned_city: &mut refresh_union_owned_city,
            update_union_player: &mut update_union_player,
            get_lei_ting_local_time: &mut get_lei_ting_local_time,
            wait: &mut wait,
            output_debug: &mut output_debug,
        };
        let result = game
            .main_loop(configuration, &mut state, &mut owners, &mut callbacks)
            .await
            .map(|report| report.legacy_result)
            .map_err(WorldProcessMainLoopBlock::Game);
        drop(callbacks);
        for player_id in observed_player_updates {
            eprintln!("WorldServer: UpdateFactionInfo применён игроку {player_id}");
        }
        for line in self.resources.drain_log_lines() {
            eprintln!("WorldServer: {}", String::from_utf8_lossy(&line));
        }
        for (title, message) in self.resources.drain_operator_notices() {
            eprintln!(
                "WorldServer: {}: {}",
                String::from_utf8_lossy(&title),
                String::from_utf8_lossy(&message)
            );
        }
        result
    }
}
