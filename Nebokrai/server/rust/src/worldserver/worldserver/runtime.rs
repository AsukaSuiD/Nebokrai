//! Process-owned ресурсы WorldServer, общие для init, reload и main loop.
//!
//! StringTable и встроенные setup-владельцы остаются у единственного `CGame`;
//! здесь собраны исторические process-global owners, передаваемые ему ссылками.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::future::Future;
use std::convert::Infallible;
use std::fmt;
use std::io;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr as UnixSocketAddr, UnixListener};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use parking_lot::{Mutex, RwLock};
use chrono::{Datelike, Timelike};

use crate::dbaccess::worlddb::dbgoods::TiberiusDbGoods;
use crate::dbaccess::worlddb::dbcountry::TiberiusDbCountry;
use crate::dbaccess::worlddb::dbmisc::{
    AuctionGoodsLoadOutcome, AuctionMoneyLoadOutcome, CDbMisc,
    TiberiusDbMiscCallbacks, TiberiusDbMiscContext, TiberiusDbMiscDatabase,
    TiberiusDbMiscRuntimeEvent,
};
use crate::dbaccess::worlddb::largess::TiberiusLargess;
use crate::dbaccess::worlddb::rsenemyfactions::TiberiusRsEnemyFactions;
use crate::dbaccess::worlddb::rsfaction::TiberiusRsFaction;
use crate::dbaccess::worlddb::rsgenvar::TiberiusRsGenVar;
use crate::dbaccess::worlddb::rsgodsbattle::TiberiusRsGodsBattle;
use crate::dbaccess::worlddb::rsjjcsys::{RsJjcSysOwner, TiberiusRsJjcSys};
use crate::dbaccess::worlddb::rsplayer::{
    LeiTingDatabaseResetRequest, TiberiusPlayerLoadData, TiberiusRsPlayer,
};
use crate::dbaccess::worlddb::rsregion::{
    RegionParametersLoadOutcome, RsRegionOwner, TiberiusRsRegion,
};
use crate::dbaccess::worlddb::rssetup::{
    LoadedSetupIds, TiberiusRsSetup, WorldDatabaseSettings, WorldTdsClient,
};
use crate::dbaccess::worlddb::rsunion::TiberiusRsUnion;
use crate::dbaccess::worlddb::writelogqueue::WorldWriteLogQueue;

use crate::public::clientresource::DefaultClientResourceOwner;
use crate::public::netsessionmanager::{CNetSessionManager, NetSessionManagerVariant};
use crate::public::auctionnode::CGoodsNode;
use crate::public::dakongxiangqian::CDaKongXiangQian;
use crate::public::auctionlog::CAuctionLog;
use crate::public::date::TagTime;
use crate::public::timer::CTimer;
use crate::nets::networld::message::CMessage;
use crate::nets::servers::ServerCommandHandle;
use crate::setup::timetoreturn::TimeToReturnCallbacks;
use crate::setup::timetoreturn::TimeToReturn;
use crate::setup::godsbattleconf::CGodsBattleConf;
use crate::setup::cbattlefairyexpconfig::CBattleFairyExpConfig;
use crate::setup::changebody::CChangeBodyConf;
use crate::setup::fairyexpconf::CFairyExpConf;
use crate::setup::gmlist::CGMList;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::goodsdestructionconfig::GoodsDestroySetup;
use crate::setup::honorelimilateconfig::HonorElimilateConfig;
use crate::setup::lingbao::CLingBaoSetup;
use crate::setup::leitingsetup::CThingSetup;
use crate::setup::logsystem::CLogSystem;
use crate::setup::monsterlist::{MonsterDropRegistry, MonsterRegistry};
use crate::setup::newskillmonsterlist::NewSkillMonsterConf;
use crate::setup::playerlist::CPlayerList;
use crate::setup::preciousboxconf::PreciousBoxConf;
use crate::setup::regionrouter::RegionRouter;
use crate::setup::regionsetup::CRegionSetup;
use crate::setup::synthesis::CSynthesis;
use crate::worldserver::appworld::goods::cbattlefairyproperty::CBattleFairyProperty;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    upgrade_equipment, GoodsBasePropertiesRegistry, GoodsNameIndex, GoodsOriginalNameIndex,
};
use crate::worldserver::appworld::player::{CPlayer, PlayerPropertyCoefficients};
use crate::worldserver::appworld::message::organsysmessage::WorldUnionApplicationRuntimeOwner;
use crate::worldserver::appworld::session::csessionfactory::CSessionFactory;
use crate::worldserver::appworld::country::countrywarsys::CountryWarCallbacks;
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::country::countryparam::CCountryParam;
use crate::worldserver::appworld::country::countrywarsys::CountryWarSys;
use crate::worldserver::appworld::goodswarmember::CGoodsWarMember;
use crate::worldserver::appworld::incrementlog::incrementlog::CIncrementLog;
use crate::worldserver::appworld::jjcsystem::{
    CJJcSystem, JjcLocalTime, JjcLogEvent, JjcRank, JjcRunContext, JjcSystemTime,
};
use crate::worldserver::appworld::leiting::{LeiTingContext, LeiTingLocalTime};
use crate::worldserver::appworld::leiting::CLeiTing;
use crate::worldserver::appworld::organizingsystem::attackcitysys::AttackCityCallbacks;
use crate::worldserver::appworld::organizingsystem::attackcitysys::CAttackCitySys;
use crate::worldserver::appworld::organizingsystem::factionwarsys::CFactionWarSys;
use crate::worldserver::appworld::organizingsystem::fournationwarsys::FourNationWarCallbacks;
use crate::worldserver::appworld::organizingsystem::fournationwarsys::CFourNationWarSys;
use crate::worldserver::appworld::organizingsystem::organizingctrl::COrganizingCtrl;
use crate::worldserver::appworld::organizingsystem::organizingparam::COrganizingParam;
use crate::worldserver::appworld::organizingsystem::villagewarsys::VillageWarCallbacks;
use crate::worldserver::appworld::organizingsystem::villagewarsys::CVillageWarSys;
use crate::worldserver::appworld::script::variablelist::CVariableList;
use crate::worldserver::appworld::skills::skillfactory::CSkillFactory;
use crate::worldserver::appworld::misc::CopyNumberTimerState;
use crate::worldserver::appworld::worldregion::WorldRegionResourceContext;

use super::game::{
    CGame, WorldGameDatabaseInitialization, WorldGameDatabaseOwner, WorldGameInitContext,
    WorldGameInitCallbacks, WorldGameInitOperatorNotice, WorldGameInitResult,
    WorldCollectPlayerDataRequestState, WorldGameInitWorkerKind, WorldJjcRuntimeContext,
    WorldLeiTingRuntimeContext, WorldMainLoopClockState, WorldMainLoopInitializationState,
    WorldMainLoopLargessState, WorldMainLoopLoginReleaseState, WorldMainLoopProfileState,
    WorldMainLoopStateOwners, WorldMainLoopTailClockState, WorldPlayerDataLoadOwner,
    WorldPlayerLoadDataAdapter, WorldPlayerRanksRequestState, WorldProcessMessageStageState,
    WorldReloadContext, WorldReloadProfileFlags, WorldRunSaveTriggerState,
    WorldSaveRuntimeContext, WorldSaveThreadHandleState,
    WorldSaveThreadJob, WorldSaveThreadLaunchRequest, WorldSaveThreadReport, save_thread_func,
};
use super::honorranks::CHonorRanks;
use super::playerranks::CPlayerRanks;
use super::worldserver::{WorldLogLocalTime, WorldLogTextOwner, WorldRefreshInfoHighWater};
use super::savedb::{
    SaveDataLifecycleState, SaveDataLocalTime, SaveDataLogPublisher,
    SaveDataMonitoringSnapshot,
};
use super::jjcmaintenanceworker::{
    WorldJjcWeekClearWorker, WorldJjcWeekClearWorkerEvent,
};
use super::leitingresetworker::{
    WorldLeiTingResetWorker, WorldLeiTingResetWorkerEvent,
};
use crate::worldserver::appworld::message::writelogmessage::WorldWriteLogCommand;

/// Стабильные typed-ключи всех callback-ов единственного World timer-owner-а.
///
/// В EXE это разные адреса функций. Rust хранит их как значения одного enum,
/// чтобы календарные записи всех подсистем находились в общем `CTimer`, но не
/// превращались в нетипизированные числовые адреса или отдельные registries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldTimerCallback {
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
pub(crate) struct WorldTimerCallbacks {
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
pub(crate) struct WorldProcessDomainOwners {
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
    /// Единственные входная/выходная FIFO аукционного DB-конвейера.
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
        let started_at = context.started_at;
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

/// Единственный набор process-global accumulators полного World MainLoop.
pub(crate) struct WorldProcessMainLoopState {
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

/// Результат одного завершившегося системного save-worker-а.
pub(crate) struct WorldSaveWorkerCompletion {
    /// Незавершённый batch остаётся owned до Release вместо тихой потери.
    pub(crate) retained_job: Option<WorldSaveThreadJob>,
    /// Blocked-путь не изображает достигнутый `LeaveCriticalSection`.
    retained_serialization: Option<tokio::sync::OwnedMutexGuard<()>>,
}

/// Единственный process-owner `g_hSavingThread` и общей save-сериализации.
pub(crate) struct WorldSaveWorker {
    runtime: tokio::runtime::Handle,
    started_at: Instant,
    settings: WorldDatabaseSettings,
    largess: Arc<TiberiusLargess>,
    log: WorldLogTextOwner,
    serialization: Arc<tokio::sync::Mutex<()>>,
    handles: Vec<JoinHandle<WorldSaveWorkerCompletion>>,
    retained_jobs: Vec<WorldSaveThreadJob>,
    retained_serializations: Vec<tokio::sync::OwnedMutexGuard<()>>,
}

impl WorldSaveWorker {
    pub(crate) fn new(
        runtime: tokio::runtime::Handle,
        started_at: Instant,
        settings: WorldDatabaseSettings,
        largess: Arc<TiberiusLargess>,
        log: WorldLogTextOwner,
    ) -> Self {
        Self {
            runtime,
            started_at,
            settings,
            largess,
            log,
            serialization: Arc::new(tokio::sync::Mutex::new(())),
            handles: Vec::new(),
            retained_jobs: Vec::new(),
            retained_serializations: Vec::new(),
        }
    }

    pub(crate) fn serialization(&self) -> Arc<tokio::sync::Mutex<()>> {
        Arc::clone(&self.serialization)
    }

    /// Запускает очередной exact background role, не ожидая предыдущий поток.
    ///
    /// Исходный caller закрывал только kernel handle: уже запущенный save-thread
    /// продолжал работу и сериализовался внутри `SaveThreadFunc`. Rust хранит
    /// join-handle-ы до release, но собирает здесь лишь уже завершившиеся.
    pub(crate) fn launch(&mut self, job: WorldSaveThreadJob) -> WorldSaveThreadHandleState {
        self.collect_finished();

        let runtime = self.runtime.clone();
        let started_at = self.started_at;
        let settings = self.settings.clone();
        let mut largess = self.largess.clone_save_owner();
        let mut log = self.log.clone();
        let serialization = Arc::clone(&self.serialization);
        let pending_job = Arc::new(parking_lot::Mutex::new(Some(job)));
        let thread_job = Arc::clone(&pending_job);
        let handle = thread::Builder::new()
            .name("world-save".to_owned())
            .spawn(move || {
                let mut job = thread_job
                    .lock()
                    .take()
                    .expect("save job принадлежит единственному worker-у");
                let mut setup_database = TiberiusRsSetup::new_for_save(settings.clone());
                let mut variable_database = TiberiusRsGenVar::new(settings.clone());
                let mut player_database = TiberiusRsPlayer::new(&settings);
                let mut jjc_database = TiberiusRsJjcSys::new(&settings);
                let mut goods_database = TiberiusDbGoods::new(&settings);
                let mut union_database = TiberiusRsUnion::new(settings.clone());
                let mut faction_database = TiberiusRsFaction::new(settings.clone());
                let mut region_database = TiberiusRsRegion::new(settings.clone());
                let mut gods_battle_database = TiberiusRsGodsBattle::new(&settings);
                let mut enemy_factions_database = TiberiusRsEnemyFactions::new(settings.clone());
                let mut country_database = TiberiusDbCountry::default();
                let mut lifecycle = *job.lifecycle.lock();
                let shared_lifecycle = Arc::clone(&job.lifecycle);
                let write_log_queue = job.write_log_queue.clone();
                let server_name = job.server_name.clone();
                let server_id = job.server_id;
                let world_number_bits = job.world_number_bits;
                let save_info_time_ms = job.save_info_time_ms;
                let mut publisher = SaveDataLogPublisher::new(
                    false,
                    save_info_time_ms,
                    &mut log,
                    move || started_at.elapsed().as_millis() as u32,
                    || {
                        let now = chrono::Local::now();
                        WorldLogLocalTime {
                            year: now.year() as u16,
                            month: now.month() as u16,
                            day: now.day() as u16,
                            hour: now.hour() as u16,
                            minute: now.minute() as u16,
                            second: now.second() as u16,
                        }
                    },
                    |payload: &[u8]| {
                        eprintln!("WorldServer: {}", String::from_utf8_lossy(payload));
                    },
                );

                let completed = runtime.block_on(async {
                    let mut serialization = Some(serialization.lock_owned().await);
                    let report = save_thread_func(
                        &mut job.save,
                        &settings,
                        &mut lifecycle,
                        &job.variables,
                        &job.registry,
                        &mut job.honor_ranks,
                        job.gods_battle_faction_xyd,
                        &job.gods_battle_npc_factions,
                        job.use_old_save_largess_way,
                        &mut setup_database,
                        &mut variable_database,
                        &mut player_database,
                        &mut jjc_database,
                        &mut goods_database,
                        &mut union_database,
                        &mut faction_database,
                        &mut region_database,
                        &mut gods_battle_database,
                        &mut enemy_factions_database,
                        &mut country_database,
                        &mut largess,
                        &mut publisher,
                        move |snapshot| *shared_lifecycle.lock() = snapshot,
                        || drop(serialization.take()),
                        move || SaveDataMonitoringSnapshot {
                            server_name,
                            write_log_count: write_log_queue.len() as u32,
                            server_id,
                            world_number_bits,
                        },
                    )
                    .await;
                    let completed = matches!(&report, WorldSaveThreadReport::Complete { .. });
                    drop(report);
                    (completed, serialization)
                });

                WorldSaveWorkerCompletion {
                    retained_job: (!completed.0).then_some(job),
                    retained_serialization: completed.1,
                }
            });

        match handle {
            Ok(handle) => {
                self.handles.push(handle);
                WorldSaveThreadHandleState::Open
            }
            Err(error) => {
                eprintln!("WorldServer: не создан save-worker: {error}");
                if let Some(job) = pending_job.lock().take() {
                    self.retained_jobs.push(job);
                }
                WorldSaveThreadHandleState::Empty
            }
        }
    }

    fn collect_completion(&mut self, handle: JoinHandle<WorldSaveWorkerCompletion>) {
        match handle.join() {
            Ok(completion) => {
                if let Some(job) = completion.retained_job {
                    self.retained_jobs.push(job);
                }
                if let Some(serialization) = completion.retained_serialization {
                    self.retained_serializations.push(serialization);
                }
            }
            Err(_) => eprintln!("WorldServer: save-worker завершился panic"),
        }
    }

    fn collect_finished(&mut self) {
        let handles = std::mem::take(&mut self.handles);
        for handle in handles {
            if handle.is_finished() {
                self.collect_completion(handle);
            } else {
                self.handles.push(handle);
            }
        }
    }

    pub(crate) fn join(&mut self) -> WorldSaveThreadHandleState {
        let previous = if self.handles.is_empty() {
            WorldSaveThreadHandleState::Empty
        } else {
            WorldSaveThreadHandleState::Open
        };
        let handles = std::mem::take(&mut self.handles);
        for handle in handles {
            self.collect_completion(handle);
        }
        previous
    }
}

/// Process-level связка trigger-guard, текущего opaque handle и save-thread-ов.
pub(crate) struct WorldProcessSaveRuntime {
    worker: WorldSaveWorker,
    trigger_guard: Option<tokio::sync::OwnedMutexGuard<()>>,
}

impl WorldProcessSaveRuntime {
    pub(crate) fn new(worker: WorldSaveWorker) -> Self {
        Self {
            worker,
            trigger_guard: None,
        }
    }

    pub(crate) fn wait_for_barrier(&self) {
        let runtime = self.worker.runtime.clone();
        let serialization = self.worker.serialization();
        tokio::task::block_in_place(|| {
            let guard = runtime.block_on(serialization.lock_owned());
            drop(guard);
        });
    }

    pub(crate) fn join(&mut self) -> WorldSaveThreadHandleState {
        self.worker.join()
    }
}

impl WorldSaveRuntimeContext for WorldProcessSaveRuntime {
    fn try_enter_trigger(&mut self) -> bool {
        if self.trigger_guard.is_some() {
            return false;
        }
        match self.worker.serialization().try_lock_owned() {
            Ok(guard) => {
                self.trigger_guard = Some(guard);
                true
            }
            Err(_) => false,
        }
    }

    fn leave_trigger(&mut self) {
        drop(self.trigger_guard.take());
    }

    fn launch(
        &mut self,
        _request: &WorldSaveThreadLaunchRequest,
        job: WorldSaveThreadJob,
    ) -> WorldSaveThreadHandleState {
        self.worker.launch(job)
    }
}

#[derive(Clone)]
pub(crate) struct WorldPlayerLoadSnapshot {
    pub(crate) thing_setup: CThingSetup,
    pub(crate) player_list: CPlayerList,
    pub(crate) globe_setup: GlobeSetupSnapshot,
    pub(crate) coefficients: PlayerPropertyCoefficients,
    pub(crate) goods: GoodsBasePropertiesRegistry,
    pub(crate) gold_coin_index: u32,
    pub(crate) gold_coin_limit: u32,
    pub(crate) use_log_system: bool,
    pub(crate) write_log_queue: WorldWriteLogQueue,
}

/// Самостоятельный DB-owner одного фонового player-load потока.
pub(crate) struct WorldProcessPlayerLoadDatabase {
    player: TiberiusRsPlayer,
    jjc: TiberiusRsJjcSys,
    goods: TiberiusDbGoods,
    snapshot: Arc<RwLock<WorldPlayerLoadSnapshot>>,
    changed_goods_indices: BTreeMap<u32, u32>,
    dakong_addon_types: BTreeSet<i32>,
}

impl WorldProcessPlayerLoadDatabase {
    pub(crate) fn new(
        settings: &WorldDatabaseSettings,
        snapshot: Arc<RwLock<WorldPlayerLoadSnapshot>>,
    ) -> Self {
        let mut dakong_addon_types = BTreeSet::new();
        CDaKongXiangQian::get_add_type(&mut dakong_addon_types);
        Self {
            player: TiberiusRsPlayer::new(settings),
            jjc: TiberiusRsJjcSys::new(settings),
            goods: TiberiusDbGoods::new(settings),
            snapshot,
            // Exact World binary только конструирует/читает static map; кроме
            // CRT teardown записей в неё нет, поэтому shipped process начинает
            // и остаётся с пустой таблицей замен индексов.
            changed_goods_indices: BTreeMap::new(),
            dakong_addon_types,
        }
    }
}

pub(crate) fn world_player_load_largess(
    largess: Arc<TiberiusLargess>,
    snapshot: Arc<RwLock<WorldPlayerLoadSnapshot>>,
) -> Box<dyn FnMut(&mut CPlayer) + Send> {
    let mut random_state = 1u32;
    Box::new(move |player| {
        let snapshot = snapshot.read().clone();
        let mut random = |upper_bound: i32| {
            random_state = random_state.wrapping_mul(214013).wrapping_add(2531011);
            let value = ((random_state >> 16) & 0x7fff) as i32;
            if upper_bound > 0 {
                value % upper_bound
            } else {
                0
            }
        };
        let mut upgrade = |goods: &mut _, target_level| {
            let _ = upgrade_equipment(Some(goods), target_level);
        };
        match largess.load_largess(
            player,
            &snapshot.goods,
            snapshot.gold_coin_index,
            snapshot.gold_coin_limit,
            snapshot.use_log_system,
            &mut random,
            &mut upgrade,
        ) {
            Ok(mut report) => {
                if let Some(record) = report.write_log.take() {
                    snapshot
                        .write_log_queue
                        .push(WorldWriteLogCommand::LargessLog(record));
                }
            }
            Err(error) => {
                eprintln!("WorldServer: выдача Largess остановлена: {error:?}");
            }
        }
    })
}

/// Platform/DB owner полного `CGame::Init`, отделённый от reload-ресурсов.
pub(crate) struct WorldProcessInitContext {
    runtime: tokio::runtime::Handle,
    started_at: Instant,
    random_state: u32,
    instance_guard: Option<UnixListener>,
    database_settings: Option<WorldDatabaseSettings>,
    log_database_settings: Option<WorldDatabaseSettings>,
    player: Option<TiberiusRsPlayer>,
    player_connection: Option<WorldTdsClient>,
    setup: Option<TiberiusRsSetup>,
    gen_var: Option<TiberiusRsGenVar>,
    faction: Option<TiberiusRsFaction>,
    union: Option<TiberiusRsUnion>,
    enemy_factions: Option<TiberiusRsEnemyFactions>,
    region: Option<TiberiusRsRegion>,
    country: Option<TiberiusDbCountry>,
    country_connection: Option<WorldTdsClient>,
    goods_war_connection: Option<WorldTdsClient>,
    db_misc: Option<TiberiusDbMiscDatabase>,
    gods_battle: Option<TiberiusRsGodsBattle>,
    gods_battle_connection: Option<WorldTdsClient>,
    log_connection: Option<WorldTdsClient>,
    largess: Option<Arc<TiberiusLargess>>,
    player_load_snapshot: Arc<RwLock<WorldPlayerLoadSnapshot>>,
}

impl WorldProcessInitContext {
    pub(crate) fn new(
        runtime: tokio::runtime::Handle,
        player_load_snapshot: Arc<RwLock<WorldPlayerLoadSnapshot>>,
    ) -> Self {
        Self {
            runtime,
            started_at: Instant::now(),
            random_state: 1,
            instance_guard: None,
            database_settings: None,
            log_database_settings: None,
            player: None,
            player_connection: None,
            setup: None,
            gen_var: None,
            faction: None,
            union: None,
            enemy_factions: None,
            region: None,
            country: None,
            country_connection: None,
            goods_war_connection: None,
            db_misc: None,
            gods_battle: None,
            gods_battle_connection: None,
            log_connection: None,
            largess: None,
            player_load_snapshot,
        }
    }

    pub(crate) fn take_db_misc_context(
        &mut self,
        registry: GoodsBasePropertiesRegistry,
        globe_setup: GlobeSetupSnapshot,
        gold_coin_index: u32,
        output: crate::dbaccess::worlddb::dbmisc::DbMiscOutputPublisher,
    ) -> Option<(TiberiusDbMiscContext, WorldDbMiscProcessConfiguration)> {
        let database = self.db_misc.take()?;
        let configuration = WorldDbMiscProcessConfiguration::new(
            globe_setup,
            gold_coin_index,
        );
        let shared = configuration.shared();
        let transfer_configuration = Arc::clone(&shared);
        let seller_configuration = Arc::clone(&shared);
        let gold_configuration = Arc::clone(&shared);
        let started_at = self.started_at;
        let mut random_state = self.random_state;
        let callbacks = TiberiusDbMiscCallbacks {
            transfer_money_interval_ms: Box::new(move || {
                transfer_configuration
                    .read()
                    .globe_setup
                    .transfer_money_interval_ms()
            }),
            current_tick_ms: Box::new(move || started_at.elapsed().as_millis() as u32),
            report_reconnect: Box::new(|| {
                eprintln!("WorldServer: аукционный DB-owner переподключается")
            }),
            report_runtime_event: Box::new(report_db_misc_runtime_event),
            seller_money_after_fee: Box::new(move |goods: &CGoodsNode| {
                let seller_money = goods.database_write_fields().seller_money;
                CGame::get_opt_money_jin(
                    &seller_configuration.read().globe_setup,
                    Some(seller_money),
                )
                .map(|money| money.seller_money_after_fee)
            }),
            gold_coin_index: Box::new(move || gold_configuration.read().gold_coin_index),
            random: Box::new(move |upper_bound| {
                random_state = random_state.wrapping_mul(214013).wrapping_add(2531011);
                let value = ((random_state >> 16) & 0x7fff) as i32;
                if upper_bound > 0 { value % upper_bound } else { 0 }
            }),
        };
        let context = TiberiusDbMiscContext::new(
            self.runtime.clone(),
            database,
            registry,
            output,
            callbacks,
        );
        Some((context, configuration))
    }

    pub(crate) fn database_settings(&self) -> Option<WorldDatabaseSettings> {
        self.database_settings.clone()
    }

    pub(crate) const fn started_at(&self) -> Instant {
        self.started_at
    }

    pub(crate) fn country_database_owner(&mut self) -> Option<&mut TiberiusDbCountry> {
        self.country.as_mut()
    }

    pub(crate) fn gods_battle_database_owner(&mut self) -> Option<&mut TiberiusRsGodsBattle> {
        self.gods_battle.as_mut()
    }

    pub(crate) fn player_database_parts(
        &mut self,
    ) -> (Option<&mut TiberiusRsPlayer>, Option<&mut WorldTdsClient>) {
        (self.player.as_mut(), self.player_connection.as_mut())
    }

    pub(crate) fn largess(&self) -> Option<Arc<TiberiusLargess>> {
        self.largess.as_ref().map(Arc::clone)
    }
}

#[derive(Clone)]
struct WorldDbMiscProcessConfigurationSnapshot {
    globe_setup: GlobeSetupSnapshot,
    gold_coin_index: u32,
}

/// Reloadable значения, которые один долгоживущий `DbMiscContext` читает в
/// каждом MainLoop turn, не сохраняя расходящуюся копию Globe/goods ID.
#[derive(Clone)]
pub(crate) struct WorldDbMiscProcessConfiguration {
    state: Arc<RwLock<WorldDbMiscProcessConfigurationSnapshot>>,
}

impl WorldDbMiscProcessConfiguration {
    fn new(globe_setup: GlobeSetupSnapshot, gold_coin_index: u32) -> Self {
        Self {
            state: Arc::new(RwLock::new(WorldDbMiscProcessConfigurationSnapshot {
                globe_setup,
                gold_coin_index,
            })),
        }
    }

    fn shared(&self) -> Arc<RwLock<WorldDbMiscProcessConfigurationSnapshot>> {
        Arc::clone(&self.state)
    }

    pub(crate) fn publish(
        &self,
        globe_setup: &GlobeSetupSnapshot,
        gold_coin_index: u32,
    ) {
        *self.state.write() = WorldDbMiscProcessConfigurationSnapshot {
            globe_setup: globe_setup.clone(),
            gold_coin_index,
        };
    }
}

fn report_db_misc_runtime_event(event: TiberiusDbMiscRuntimeEvent<'_>) {
    match event {
        TiberiusDbMiscRuntimeEvent::MissingNormalConnection(operation) => {
            eprintln!("WorldServer: отсутствует аукционное DB-соединение для {operation:?}")
        }
        TiberiusDbMiscRuntimeEvent::NormalConnectionFailed(error) => {
            eprintln!("WorldServer: аукционное DB-переподключение не выполнено: {error:?}")
        }
        TiberiusDbMiscRuntimeEvent::ConnectionCheck(state) => {
            eprintln!("WorldServer: аукционное DB-соединение неактивно: {state:?}")
        }
        TiberiusDbMiscRuntimeEvent::Write(outcome) => {
            eprintln!("WorldServer: аукционная DB-запись не выполнена: {outcome:?}")
        }
        TiberiusDbMiscRuntimeEvent::GoodsLoad(outcome) => {
            match outcome {
                AuctionGoodsLoadOutcome::Loaded(_) => {}
                AuctionGoodsLoadOutcome::ReturnedFalse(error) => {
                    eprintln!("WorldServer: аукционные товары не загружены: {error}")
                }
                AuctionGoodsLoadOutcome::BlockedMissingFact(reason) => eprintln!(
                    "WorldServer: загрузка аукционных товаров остановлена: исходное продолжение не доказано ({reason:?})"
                ),
            }
        }
        TiberiusDbMiscRuntimeEvent::MoneyLoad(outcome) => {
            match outcome {
                AuctionMoneyLoadOutcome::Loaded { .. } => {}
                AuctionMoneyLoadOutcome::ReturnedFalse(error) => {
                    eprintln!("WorldServer: аукционные деньги не загружены: {error}")
                }
                AuctionMoneyLoadOutcome::BlockedMissingFact(reason) => eprintln!(
                    "WorldServer: загрузка аукционных денег остановлена: исходное продолжение не доказано ({reason:?})"
                ),
            }
        }
        TiberiusDbMiscRuntimeEvent::OwnerListFailed(error) => {
            eprintln!("WorldServer: список владельцев аукциона не загружен: {error:?}")
        }
    }
}

#[derive(Clone)]
struct WorldRuntimeLog {
    owner: WorldLogTextOwner,
    started_at: Instant,
    save_info_time_ms: u32,
}

impl WorldRuntimeLog {
    fn add(&self, payload: &[u8]) {
        let started_at = self.started_at;
        let _ = self.owner.add_log_text(
            payload,
            self.save_info_time_ms,
            move || started_at.elapsed().as_millis() as u32,
            current_world_log_time,
            |line| eprintln!("WorldServer: {}", String::from_utf8_lossy(line)),
        );
    }
}

fn current_world_log_time() -> WorldLogLocalTime {
    let now = chrono::Local::now();
    WorldLogLocalTime {
        year: now.year() as u16,
        month: now.month() as u16,
        day: now.day() as u16,
        hour: now.hour() as u16,
        minute: now.minute() as u16,
        second: now.second() as u16,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlatformTimeError {
    LocalTimeUnavailable { timestamp: i64 },
    NormalizedTimestampOutsideLegacyRange { timestamp: i64 },
}

impl fmt::Display for WorldPlatformTimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalTimeUnavailable { timestamp } => {
                write!(formatter, "localtime не представил timestamp {timestamp}")
            }
            Self::NormalizedTimestampOutsideLegacyRange { timestamp } => write!(
                formatter,
                "mktime вернул {timestamp}, непредставимый 32-битным time_t оригинала"
            ),
        }
    }
}

impl Error for WorldPlatformTimeError {}

fn local_tm(timestamp: i64) -> Result<libc::tm, WorldPlatformTimeError> {
    let timestamp = timestamp as libc::time_t;
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();
    // `localtime_r` — потокобезопасная системная замена MSVC `_localtime`;
    // указатели живут только внутри этого вызова и результат сразу копируется.
    let result = unsafe { libc::localtime_r(&timestamp, local.as_mut_ptr()) };
    if result.is_null() {
        return Err(WorldPlatformTimeError::LocalTimeUnavailable {
            timestamp: timestamp as i64,
        });
    }
    // `localtime_r` при non-null результате полностью инициализировал `tm`.
    Ok(unsafe { local.assume_init() })
}

fn jjc_local_time(timestamp: i32) -> JjcLocalTime {
    let local = local_tm(i64::from(timestamp))
        .expect("signed 32-битный JJC timestamp обязан представляться системным localtime");
    JjcLocalTime {
        second: local.tm_sec,
        minute: local.tm_min,
        hour: local.tm_hour,
        month_day: local.tm_mday,
        month: local.tm_mon,
        year_since_1900: local.tm_year,
        week_day: local.tm_wday,
        year_day: local.tm_yday,
        daylight_saving: local.tm_isdst,
    }
}

pub(crate) fn current_lei_ting_local_time() -> LeiTingLocalTime {
    let timestamp = chrono::Local::now().timestamp();
    let local = local_tm(timestamp)
        .expect("текущее системное время обязано представляться localtime");
    lei_ting_time_from_tm(&local)
}

fn lei_ting_time_from_tm(local: &libc::tm) -> LeiTingLocalTime {
    LeiTingLocalTime {
        second: local.tm_sec,
        minute: local.tm_min,
        hour: local.tm_hour,
        month_day: local.tm_mday,
        month: local.tm_mon,
        year_since_1900: local.tm_year,
        week_day: local.tm_wday,
        year_day: local.tm_yday,
        daylight_saving: local.tm_isdst,
    }
}

fn normalize_lei_ting_time(
    local: &mut LeiTingLocalTime,
) -> Result<i32, WorldPlatformTimeError> {
    let mut native = libc::tm {
        tm_sec: local.second,
        tm_min: local.minute,
        tm_hour: local.hour,
        tm_mday: local.month_day,
        tm_mon: local.month,
        tm_year: local.year_since_1900,
        tm_wday: local.week_day,
        tm_yday: local.year_day,
        tm_isdst: local.daylight_saving,
        ..unsafe { std::mem::zeroed() }
    };
    // `_mktime` в EXE одновременно нормализовал все девять полей `tm`.
    let timestamp = unsafe { libc::mktime(&mut native) };
    *local = lei_ting_time_from_tm(&native);
    i32::try_from(timestamp).map_err(|_| {
        WorldPlatformTimeError::NormalizedTimestampOutsideLegacyRange {
            timestamp: timestamp as i64,
        }
    })
}

/// Concrete platform/DB/log owner периодического `CJJcSystem::Run`.
pub(crate) struct WorldJjcProcessContext {
    runtime: tokio::runtime::Handle,
    started_at: Instant,
    config_path: PathBuf,
    database: TiberiusRsJjcSys,
    worker: Arc<WorldJjcWeekClearWorker>,
    log: WorldRuntimeLog,
}

impl WorldJjcProcessContext {
    pub(crate) fn new(
        runtime: tokio::runtime::Handle,
        started_at: Instant,
        runtime_directory: &Path,
        settings: &WorldDatabaseSettings,
        worker: Arc<WorldJjcWeekClearWorker>,
        log: WorldLogTextOwner,
        save_info_time_ms: u32,
    ) -> Self {
        Self {
            runtime,
            started_at,
            config_path: runtime_directory.join("setup").join("JJcConfig.ini"),
            database: TiberiusRsJjcSys::new(settings),
            worker,
            log: WorldRuntimeLog {
                owner: log,
                started_at,
                save_info_time_ms,
            },
        }
    }
}

impl JjcRunContext for WorldJjcProcessContext {
    fn current_time_seconds(&mut self) -> i32 {
        chrono::Local::now().timestamp() as i32
    }

    fn local_time(&mut self, timestamp: i32) -> JjcLocalTime {
        jjc_local_time(timestamp)
    }

    fn system_time(&mut self) -> JjcSystemTime {
        let now = chrono::Local::now();
        JjcSystemTime {
            year: now.year() as u16,
            month: now.month() as u16,
            week_day: now.weekday().num_days_from_sunday() as u16,
            day: now.day() as u16,
            hour: now.hour() as u16,
            minute: now.minute() as u16,
            second: now.second() as u16,
            milliseconds: now.timestamp_subsec_millis() as u16,
        }
    }

    fn tick_count_ms(&mut self) -> u32 {
        self.started_at.elapsed().as_millis() as u32
    }

    fn load_jjc_rank(&mut self, ranks: &mut Vec<JjcRank>) -> bool {
        self.database.load_jjc_rank(ranks)
    }

    fn start_jjc_week_clear(&mut self) -> bool {
        self.worker.dispatch(self.runtime.clone()).is_ok()
    }

    fn clear_jjc_season(&mut self) -> bool {
        self.worker.clear_season(self.runtime.clone())
    }

    fn write_private_profile_string(
        &mut self,
        section: &[u8],
        key: &[u8],
        value: &[u8],
    ) -> bool {
        replace_ini_value(&self.config_path, section, key, value).is_ok()
    }

    fn log(&mut self, event: JjcLogEvent) {
        match event {
            JjcLogEvent::Closed => self.log.add(b"JJc is Closed."),
            JjcLogEvent::RankLoaded { success, item_count } => self.log.add(
                format!(
                    "Load JJc ranks from DB ({item_count} items) {}.",
                    if success { "OK" } else { "Fail" }
                )
                .as_bytes(),
            ),
            JjcLogEvent::WeekUpdateStarted { local_time } => self.log.add(
                format!(
                    "JJc week update start at {:04}-{:02}-{:02} {:02}:{:02}:{:02}.",
                    local_time.year,
                    local_time.month,
                    local_time.day,
                    local_time.hour,
                    local_time.minute,
                    local_time.second,
                )
                .as_bytes(),
            ),
            JjcLogEvent::WeekUpdateThreadFailed => {
                self.log.add(b"JJc week update thread can't begin.")
            }
            JjcLogEvent::SeasonUpdateStarted => self.log.add(b"JJc season update start."),
            JjcLogEvent::WeekOrSeasonUpdateFinished { elapsed_ms } => self.log.add(
                format!("JJc week or season update finished in {elapsed_ms} ms.").as_bytes(),
            ),
            JjcLogEvent::WeekClearDatabaseSlow { elapsed_ms } => self.log.add(
                format!("JJc week database clear used {elapsed_ms} ms.").as_bytes(),
            ),
            event => eprintln!("WorldServer: JJC runtime-событие {event:?}"),
        }
    }
}

impl WorldJjcRuntimeContext for WorldJjcProcessContext {
    fn on_week_clear_spawn_failed(&mut self, error: io::Error) {
        eprintln!("WorldServer: не создан JJC week-clear worker: {error}");
    }

    fn on_week_clear_worker_event(&mut self, event: WorldJjcWeekClearWorkerEvent) {
        eprintln!("WorldServer: JJC DB worker: {event:?}");
    }
}

/// Concrete platform/transport/log owner суточного `CLeiTing::Run`.
pub(crate) struct WorldLeiTingProcessContext {
    runtime: tokio::runtime::Handle,
    sender: Option<ServerCommandHandle>,
    worker: Arc<WorldLeiTingResetWorker>,
    log: WorldRuntimeLog,
}

impl WorldLeiTingProcessContext {
    pub(crate) fn new(
        runtime: tokio::runtime::Handle,
        sender: Option<ServerCommandHandle>,
        worker: Arc<WorldLeiTingResetWorker>,
        log: WorldLogTextOwner,
        started_at: Instant,
        save_info_time_ms: u32,
    ) -> Self {
        Self {
            runtime,
            sender,
            worker,
            log: WorldRuntimeLog {
                owner: log,
                started_at,
                save_info_time_ms,
            },
        }
    }
}

impl LeiTingContext for WorldLeiTingProcessContext {
    type Block = WorldPlatformTimeError;

    fn add_update_start_log(&mut self) {
        let now = chrono::Local::now();
        self.log.add(
            format!(
                "UpdateLeiTing Start at :{}-{}-{} {}:{}:{} \r\n",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
            )
            .as_bytes(),
        );
    }

    fn local_time_from_timestamp(
        &mut self,
        timestamp: u32,
    ) -> Result<LeiTingLocalTime, Self::Block> {
        let signed_timestamp = i64::from(timestamp as i32);
        local_tm(signed_timestamp).map(|local| lei_ting_time_from_tm(&local))
    }

    fn current_week_day(&mut self) -> u16 {
        chrono::Local::now().weekday().num_days_from_sunday() as u16
    }

    fn send_all(&mut self, message: &CMessage) {
        let _ = message.send_all(self.sender.as_ref());
    }

    fn add_database_begin_log(&mut self) {
        self.log
            .add(b"UpdateLeiTing Start, ResetAllLeitingInDB Begin");
    }

    fn mktime(&mut self, local_time: &mut LeiTingLocalTime) -> Result<i32, Self::Block> {
        normalize_lei_ting_time(local_time)
    }

    fn reset_all_lei_ting_in_database(&mut self, update_kind: u32, stamp: i32) {
        let request = LeiTingDatabaseResetRequest { update_kind, stamp };
        if let Err(error) = self.worker.dispatch(request, self.runtime.clone()) {
            self.on_database_reset_spawn_failed(request, error);
        }
    }

    fn add_update_end_log(&mut self) {
        self.log.add(b"LeiTing All Update End");
    }
}

impl WorldLeiTingRuntimeContext for WorldLeiTingProcessContext {
    fn on_database_reset_spawn_failed(
        &mut self,
        request: LeiTingDatabaseResetRequest,
        error: io::Error,
    ) {
        eprintln!(
            "WorldServer: не создан LeiTing DB worker для kind {} stamp {}: {error}",
            request.update_kind, request.stamp
        );
    }

    fn on_database_reset_worker_event(&mut self, event: WorldLeiTingResetWorkerEvent) {
        match event {
            WorldLeiTingResetWorkerEvent::Started(_) => {
                self.log.add(b"Strictest Enforcement update thread begin.")
            }
            WorldLeiTingResetWorkerEvent::Finished { outcome, .. } => {
                eprintln!("WorldServer: LeiTing DB worker завершён: {outcome:?}")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldMainLoopContextBuildError {
    MissingDatabaseSettings,
    MissingDbMiscOwner,
}

impl fmt::Display for WorldMainLoopContextBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDatabaseSettings => {
                formatter.write_str("World DB settings не опубликованы после Init")
            }
            Self::MissingDbMiscOwner => {
                formatter.write_str("World DbMisc owner не опубликован после Init")
            }
        }
    }
}

impl Error for WorldMainLoopContextBuildError {}

/// Долгоживущие concrete контексты трёх последовательных MainLoop DB-stage.
/// Все они строятся только после успешного `CGame::Init` из тех же setup,
/// resource, transport и FIFO owners, которые Init уже опубликовал.
pub(crate) struct WorldProcessMainLoopContexts {
    pub(crate) db_misc: TiberiusDbMiscContext,
    pub(crate) db_misc_configuration: WorldDbMiscProcessConfiguration,
    pub(crate) jjc_worker: Arc<WorldJjcWeekClearWorker>,
    pub(crate) jjc: WorldJjcProcessContext,
    pub(crate) lei_ting_worker: Arc<WorldLeiTingResetWorker>,
    pub(crate) lei_ting: WorldLeiTingProcessContext,
    pub(crate) lei_ting_owner: CLeiTing,
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
        let snapshot = resources.player_load_snapshot.read().clone();
        let (db_misc, db_misc_configuration) = init
            .take_db_misc_context(
                snapshot.goods.clone(),
                snapshot.globe_setup.clone(),
                snapshot.gold_coin_index,
                domains.db_misc.output_publisher(),
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
        })
    }
}

fn replace_ini_value(
    path: &Path,
    section: &[u8],
    key: &[u8],
    value: &[u8],
) -> io::Result<()> {
    let source = std::fs::read(path).unwrap_or_default();
    let mut section_start = None;
    let mut section_end = source.len();
    let mut value_range = None;
    let mut current_section_matches = false;
    let mut line_start = 0;

    while line_start < source.len() {
        let line_end = source[line_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(source.len(), |offset| line_start + offset);
        let content_end = line_end
            - usize::from(line_end > line_start && source[line_end - 1] == b'\r');
        let line = &source[line_start..content_end];
        let trimmed = trim_ascii(line);
        if trimmed.starts_with(b"[") && trimmed.ends_with(b"]") {
            if current_section_matches {
                section_end = line_start;
                break;
            }
            current_section_matches = trimmed[1..trimmed.len() - 1]
                .eq_ignore_ascii_case(section);
            if current_section_matches {
                section_start = Some(if line_end < source.len() { line_end + 1 } else { line_end });
            }
        } else if current_section_matches
            && let Some(equal) = line.iter().position(|byte| *byte == b'=')
            && trim_ascii(&line[..equal]).eq_ignore_ascii_case(key)
        {
            let mut value_start = line_start + equal + 1;
            while value_start < content_end && source[value_start].is_ascii_whitespace() {
                value_start += 1;
            }
            value_range = Some(value_start..content_end);
            break;
        }
        line_start = if line_end < source.len() { line_end + 1 } else { source.len() };
    }

    let mut updated = source;
    if let Some(range) = value_range {
        updated.splice(range, value.iter().copied());
    } else if let Some(insert_at) = section_start.map(|_| section_end) {
        let mut line = key.to_vec();
        line.extend_from_slice(b"=");
        line.extend_from_slice(value);
        line.extend_from_slice(b"\r\n");
        updated.splice(insert_at..insert_at, line);
    } else {
        if !updated.is_empty() && !updated.ends_with(b"\n") {
            updated.extend_from_slice(b"\r\n");
        }
        updated.extend_from_slice(b"[");
        updated.extend_from_slice(section);
        updated.extend_from_slice(b"]\r\n");
        updated.extend_from_slice(key);
        updated.extend_from_slice(b"=");
        updated.extend_from_slice(value);
        updated.extend_from_slice(b"\r\n");
    }
    std::fs::write(path, updated)
}

fn trim_ascii(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}

impl WorldGameInitContext for WorldProcessInitContext {
    type Block = Infallible;
    type PlayerDatabase = TiberiusRsPlayer;
    type EnemyFactionsDatabase = TiberiusRsEnemyFactions;
    type GeneralVariableDatabase = TiberiusRsGenVar;
    type UnionDatabase = TiberiusRsUnion;
    type FactionDatabase = TiberiusRsFaction;
    type CountryDatabase = TiberiusDbCountry;
    type PlayerLoadDatabase = WorldProcessPlayerLoadDatabase;
    type PlayerLoadLargess = Box<dyn FnMut(&mut CPlayer) + Send>;
    type PlayerLoadClock = Box<dyn FnMut() -> u32 + Send>;

    fn install_crash_reporter(&mut self) {
        std::panic::set_hook(Box::new(|panic| {
            eprintln!("WorldServer: аварийное завершение: {panic}");
        }));
    }

    fn current_time_seconds(&mut self) -> i64 {
        chrono::Local::now().timestamp()
    }

    fn seed_random(&mut self, seed: u32) {
        self.random_state = seed;
    }

    fn random(&mut self, upper_bound: i32) -> i32 {
        self.random_state = self
            .random_state
            .wrapping_mul(214013)
            .wrapping_add(2531011);
        let value = ((self.random_state >> 16) & 0x7fff) as i32;
        if upper_bound > 0 { value % upper_bound } else { 0 }
    }

    fn put_debug_string(&mut self, payload: &[u8]) {
        eprintln!("WorldServer: {}", String::from_utf8_lossy(payload));
    }

    fn claim_single_instance(&mut self, title: &[u8]) -> bool {
        if self.instance_guard.is_some() {
            return false;
        }
        let Ok(address) = UnixSocketAddr::from_abstract_name(title) else {
            return false;
        };
        match UnixListener::bind_addr(&address) {
            Ok(listener) => {
                self.instance_guard = Some(listener);
                true
            }
            Err(_) => false,
        }
    }

    fn notify_operator(&mut self, notice: &WorldGameInitOperatorNotice) {
        eprintln!(
            "WorldServer: {}: {}",
            String::from_utf8_lossy(&notice.title),
            String::from_utf8_lossy(&notice.message)
        );
    }

    fn initialize_database_layer(
        &mut self,
        initialization: WorldGameDatabaseInitialization,
    ) -> Result<(), Self::Block> {
        self.database_settings = Some(initialization.settings.clone());
        self.log_database_settings = Some(initialization.log_settings);
        self.largess = Some(Arc::new(TiberiusLargess::new(
            initialization.load_largess_time_ms,
            initialization.incoming_cost_settings,
            initialization.cost_settings,
            BTreeMap::new(),
        )));
        Ok(())
    }

    async fn create_database_owner(
        &mut self,
        owner: WorldGameDatabaseOwner,
    ) -> Result<(), Self::Block> {
        let settings = self
            .database_settings
            .as_ref()
            .expect("database layer создаётся раньше DB-owner-ов")
            .clone();
        match owner {
            WorldGameDatabaseOwner::RsPlayer => {
                self.player = Some(TiberiusRsPlayer::new(&settings));
                self.player_connection = settings.connect().await.ok();
                if let Some(log_settings) = self.log_database_settings.as_ref() {
                    self.log_connection = log_settings.connect().await.ok();
                }
            }
            WorldGameDatabaseOwner::RsGenVar => {
                self.gen_var = Some(TiberiusRsGenVar::new(settings));
            }
            WorldGameDatabaseOwner::RsFaction => {
                self.faction = Some(TiberiusRsFaction::new(settings));
            }
            WorldGameDatabaseOwner::RsUnion => {
                self.union = Some(TiberiusRsUnion::new(settings));
            }
            WorldGameDatabaseOwner::RsEnemyFactions => {
                self.enemy_factions = Some(TiberiusRsEnemyFactions::new(settings));
            }
            WorldGameDatabaseOwner::RsRegion => {
                self.region = Some(TiberiusRsRegion::new(settings));
            }
            WorldGameDatabaseOwner::DbMisc => {
                let mut database = TiberiusDbMiscDatabase::new(&settings);
                if let Err(error) = database.initialize_normal_connection().await {
                    // Конструктор старого CDbMisc также сохранял owner после
                    // неуспешного CreateNormalCn: следующий MainLoop batch
                    // повторял reconnect через тот же контекст.
                    eprintln!(
                        "WorldServer: начальное соединение аукционного DB-owner-а не открыто: {error:?}"
                    );
                }
                self.db_misc = Some(database);
            }
            WorldGameDatabaseOwner::DbCountry => {
                self.country = Some(TiberiusDbCountry::default());
                self.country_connection = settings.connect().await.ok();
            }
            WorldGameDatabaseOwner::RsGodsBattle => {
                self.gods_battle = Some(TiberiusRsGodsBattle::new(&settings));
                self.gods_battle_connection = settings.connect().await.ok();
            }
            WorldGameDatabaseOwner::GoodsWarMember => {
                self.goods_war_connection = settings.connect().await.ok();
            }
            WorldGameDatabaseOwner::RsVillageWar
            | WorldGameDatabaseOwner::RsCityWar => {}
        }
        Ok(())
    }

    async fn create_rs_setup_owner(&mut self) -> Result<LoadedSetupIds, Self::Block> {
        let settings = self
            .database_settings
            .as_ref()
            .expect("database layer создаётся раньше CRsSetup")
            .clone();
        let (owner, ids) = TiberiusRsSetup::initialize(settings).await;
        self.setup = Some(owner);
        Ok(ids)
    }

    async fn load_region_parameters(&mut self, game: &mut CGame) -> bool {
        matches!(self.region
            .as_mut()
            .expect("CRsRegion создаётся до LoadRegionParam")
            .load_region_parameters(game)
            .await, RegionParametersLoadOutcome::ReturnedTrue { .. })
    }

    fn player_database(&mut self) -> (&mut Self::PlayerDatabase, Option<&mut WorldTdsClient>) {
        (
            self.player.as_mut().expect("CRsPlayer уже создан"),
            self.player_connection.as_mut(),
        )
    }

    fn enemy_factions_database(&mut self) -> &mut Self::EnemyFactionsDatabase {
        self.enemy_factions
            .as_mut()
            .expect("CRsEnemyFactions уже создан")
    }

    fn general_variable_database(&mut self) -> &mut Self::GeneralVariableDatabase {
        self.gen_var.as_mut().expect("CRsGenVar уже создан")
    }

    fn organizing_databases(&mut self) -> (&mut Self::UnionDatabase, &mut Self::FactionDatabase) {
        (
            self.union.as_mut().expect("CRsUnion уже создан"),
            self.faction.as_mut().expect("CRsFaction уже создан"),
        )
    }

    fn country_database(
        &mut self,
    ) -> (&mut Self::CountryDatabase, Option<&mut WorldTdsClient>) {
        (
            self.country.as_mut().expect("CDBCountry уже создан"),
            self.country_connection.as_mut(),
        )
    }

    fn goods_war_database_connection(&mut self) -> Option<&mut WorldTdsClient> {
        self.goods_war_connection.as_mut()
    }

    fn gods_battle_database(&mut self) -> Option<&mut TiberiusRsGodsBattle> {
        self.gods_battle.as_mut()
    }

    fn increment_log_database(&mut self) -> Option<&mut WorldTdsClient> {
        self.log_connection.as_mut()
    }

    fn auction_log_database(&mut self) -> Option<&mut WorldTdsClient> {
        self.log_connection.as_mut()
    }

    fn player_load_worker_runtime(
        &mut self,
        _worker_index: u32,
    ) -> (
        tokio::runtime::Handle,
        Self::PlayerLoadDatabase,
        Self::PlayerLoadLargess,
        Self::PlayerLoadClock,
    ) {
        let settings = self
            .database_settings
            .as_ref()
            .expect("database layer создаётся раньше player-load worker");
        let snapshot = Arc::clone(&self.player_load_snapshot);
        let database = WorldProcessPlayerLoadDatabase::new(settings, Arc::clone(&snapshot));
        let largess = world_player_load_largess(
            self.largess.as_ref().expect("CLargess уже создан").clone(),
            snapshot,
        );
        let started_at = self.started_at;
        let clock: Self::PlayerLoadClock = Box::new(move || {
            started_at.elapsed().as_millis() as u32
        });
        (self.runtime.clone(), database, largess, clock)
    }

    fn write_log_worker_runtime(&mut self) -> tokio::runtime::Handle {
        self.runtime.clone()
    }

    fn report_worker_spawn_error(&mut self, kind: WorldGameInitWorkerKind, error: &io::Error) {
        eprintln!("WorldServer: не запущен worker {kind:?}: {error}");
    }
}

impl WorldPlayerDataLoadOwner for WorldProcessPlayerLoadDatabase {
    fn load_player_data<'a>(
        &'a mut self,
        player: &'a mut CPlayer,
    ) -> impl Future<Output = bool> + 'a {
        async move {
            let snapshot = self.snapshot.read().clone();
            let mut get_week_day = || chrono::Local::now().weekday().num_days_from_sunday() as u16;
            let mut loader = TiberiusPlayerLoadData {
                player_owner: &mut self.player,
                active_transaction: None,
                thing_setup: &snapshot.thing_setup,
                get_week_day: &mut get_week_day,
                jjc_owner: &mut self.jjc,
                goods_owner: &mut self.goods,
                goods_registry: &snapshot.goods,
                changed_goods_indices: &self.changed_goods_indices,
                dakong_addon_types: &self.dakong_addon_types,
            };
            let mut player_list = snapshot.player_list.clone();
            let mut adapter = WorldPlayerLoadDataAdapter::new(
                &mut loader,
                &mut player_list,
                &snapshot.globe_setup,
                &snapshot.coefficients,
            );
            adapter.load_player_data(player).await
        }
    }
}

pub(crate) struct WorldProcessResources {
    runtime_directory: PathBuf,
    default_client_resource: DefaultClientResourceOwner,
    goods: GoodsBasePropertiesRegistry,
    goods_by_original_name: GoodsOriginalNameIndex,
    goods_by_name: GoodsNameIndex,
    monsters: MonsterRegistry,
    monster_drops: MonsterDropRegistry,
    log_system: CLogSystem,
    region_setup: CRegionSetup,
    gm_list: CGMList,
    globe_setup: GlobeSetupSnapshot,
    region_router: RegionRouter,
    player_list: CPlayerList,
    goods_destroy: GoodsDestroySetup,
    new_skill_monsters: NewSkillMonsterConf,
    battle_fairy_exp: CBattleFairyExpConfig,
    battle_fairy_property: CBattleFairyProperty,
    synthesis: CSynthesis,
    honor_eliminate: HonorElimilateConfig,
    fairy_exp: CFairyExpConf,
    da_kong_xiang_qian: CDaKongXiangQian,
    change_body: CChangeBodyConf,
    precious_box: PreciousBoxConf,
    ling_bao: CLingBaoSetup,
    region_monsters: i32,
    region_npcs: i32,
    log_lines: Vec<Vec<u8>>,
    operator_notices: Vec<(Vec<u8>, Vec<u8>)>,
    player_load_snapshot: Arc<RwLock<WorldPlayerLoadSnapshot>>,
    db_misc_configuration: Option<WorldDbMiscProcessConfiguration>,
}

impl WorldProcessResources {
    pub(crate) fn new(runtime_directory: PathBuf) -> Self {
        let globe_setup = GlobeSetupSnapshot::default();
        let player_load_snapshot = Arc::new(RwLock::new(WorldPlayerLoadSnapshot {
            thing_setup: CThingSetup::default(),
            player_list: CPlayerList::default(),
            globe_setup: globe_setup.clone(),
            coefficients: globe_setup.player_property_coefficients().into(),
            goods: GoodsBasePropertiesRegistry::default(),
            gold_coin_index: 0,
            gold_coin_limit: 0,
            use_log_system: false,
            write_log_queue: WorldWriteLogQueue::default(),
        }));
        Self {
            runtime_directory,
            default_client_resource: Default::default(),
            goods: Default::default(),
            goods_by_original_name: Default::default(),
            goods_by_name: Default::default(),
            monsters: Default::default(),
            monster_drops: Default::default(),
            log_system: Default::default(),
            region_setup: Default::default(),
            gm_list: Default::default(),
            globe_setup,
            region_router: Default::default(),
            player_list: Default::default(),
            goods_destroy: Default::default(),
            new_skill_monsters: Default::default(),
            battle_fairy_exp: Default::default(),
            battle_fairy_property: CBattleFairyProperty::with_constructor_defaults(),
            synthesis: Default::default(),
            honor_eliminate: Default::default(),
            fairy_exp: Default::default(),
            da_kong_xiang_qian: Default::default(),
            change_body: Default::default(),
            precious_box: Default::default(),
            ling_bao: Default::default(),
            region_monsters: 0,
            region_npcs: 0,
            log_lines: Vec::new(),
            operator_notices: Vec::new(),
            player_load_snapshot,
            db_misc_configuration: None,
        }
    }

    pub(crate) fn install_default_client_resource(&mut self) {
        let _ = self
            .default_client_resource
            .replace_from_world_directory(&self.runtime_directory);
    }

    pub(crate) fn drain_log_lines(&mut self) -> impl Iterator<Item = Vec<u8>> + '_ {
        self.log_lines.drain(..)
    }

    pub(crate) fn drain_operator_notices(
        &mut self,
    ) -> impl Iterator<Item = (Vec<u8>, Vec<u8>)> + '_ {
        self.operator_notices.drain(..)
    }

    pub(crate) fn player_load_snapshot(&self) -> Arc<RwLock<WorldPlayerLoadSnapshot>> {
        Arc::clone(&self.player_load_snapshot)
    }

    pub(crate) fn runtime_directory(&self) -> &Path {
        &self.runtime_directory
    }

    pub(crate) fn install_db_misc_configuration(
        &mut self,
        configuration: WorldDbMiscProcessConfiguration,
    ) {
        self.db_misc_configuration = Some(configuration);
    }
}

impl WorldRegionResourceContext for WorldProcessResources {
    fn default_client_resource(&mut self) -> &mut DefaultClientResourceOwner {
        &mut self.default_client_resource
    }

    fn region_monster_num_scale(&mut self) -> f32 {
        self.globe_setup.monster_number_scale()
    }
}

impl WorldReloadContext for WorldProcessResources {
    fn runtime_directory(&self) -> &Path {
        &self.runtime_directory
    }

    fn goods_registries(
        &mut self,
    ) -> (
        &mut GoodsBasePropertiesRegistry,
        &mut GoodsOriginalNameIndex,
        &mut GoodsNameIndex,
    ) {
        (
            &mut self.goods,
            &mut self.goods_by_original_name,
            &mut self.goods_by_name,
        )
    }

    fn monster_registries(&mut self) -> (&mut MonsterRegistry, &mut MonsterDropRegistry) {
        (&mut self.monsters, &mut self.monster_drops)
    }

    fn log_system(&mut self) -> &mut CLogSystem { &mut self.log_system }
    fn region_setup(&mut self) -> &mut CRegionSetup { &mut self.region_setup }
    fn gm_list(&mut self) -> &mut CGMList { &mut self.gm_list }
    fn globe_setup(&mut self) -> &mut GlobeSetupSnapshot { &mut self.globe_setup }
    fn region_router(&mut self) -> &mut RegionRouter { &mut self.region_router }

    fn globe_setup_and_router(&mut self) -> (&GlobeSetupSnapshot, &RegionRouter) {
        (&self.globe_setup, &self.region_router)
    }

    fn player_list(&mut self) -> &mut CPlayerList { &mut self.player_list }
    fn goods_destroy_setup(&mut self) -> &mut GoodsDestroySetup { &mut self.goods_destroy }
    fn new_skill_monster_conf(&mut self) -> &mut NewSkillMonsterConf { &mut self.new_skill_monsters }
    fn battle_fairy_exp_config(&mut self) -> &mut CBattleFairyExpConfig { &mut self.battle_fairy_exp }
    fn battle_fairy_property(&mut self) -> &mut CBattleFairyProperty { &mut self.battle_fairy_property }
    fn synthesis(&mut self) -> &mut CSynthesis { &mut self.synthesis }
    fn honor_eliminate_config(&mut self) -> &mut HonorElimilateConfig { &mut self.honor_eliminate }
    fn fairy_exp_conf(&mut self) -> &mut CFairyExpConf { &mut self.fairy_exp }
    fn da_kong_xiang_qian(&mut self) -> &mut CDaKongXiangQian { &mut self.da_kong_xiang_qian }
    fn change_body_conf(&mut self) -> &mut CChangeBodyConf { &mut self.change_body }
    fn precious_box_conf(&mut self) -> &mut PreciousBoxConf { &mut self.precious_box }
    fn ling_bao_setup(&mut self) -> &mut CLingBaoSetup { &mut self.ling_bao }

    fn publish_player_load_snapshot(
        &mut self,
        thing_setup: &CThingSetup,
        gold_coin_index: u32,
        gold_coin_limit: u32,
        use_log_system: bool,
        write_log_queue: WorldWriteLogQueue,
    ) {
        *self.player_load_snapshot.write() = WorldPlayerLoadSnapshot {
            thing_setup: thing_setup.clone(),
            player_list: self.player_list.clone(),
            globe_setup: self.globe_setup.clone(),
            coefficients: self.globe_setup.player_property_coefficients().into(),
            goods: self.goods.clone(),
            gold_coin_index,
            gold_coin_limit,
            use_log_system,
            write_log_queue,
        };
        if let Some(configuration) = self.db_misc_configuration.as_ref() {
            configuration.publish(&self.globe_setup, gold_coin_index);
        }
    }

    fn query_goods_id_by_original_name(&mut self, original_name: &[u8]) -> u32 {
        self.goods_by_original_name.get(original_name).copied().unwrap_or(0)
    }

    fn query_goods_name(&mut self, goods_id: u32) -> Option<Vec<u8>> {
        self.goods
            .get(&goods_id)
            .and_then(Option::as_ref)
            .map(|properties| properties.get_name().to_vec())
    }

    fn four_nation_country_names(&mut self) -> [Vec<u8>; 5] {
        std::array::from_fn(|index| {
            self.globe_setup
                .country_name(index as u8)
                .unwrap_or_default()
                .to_vec()
        })
    }

    fn add_log_text(&mut self, payload: &[u8]) { self.log_lines.push(payload.to_vec()); }

    fn notify_reload_operator(&mut self, title: &[u8], message: &[u8]) {
        self.operator_notices.push((title.to_vec(), message.to_vec()));
    }

    fn add_region_object_counts(&mut self, monsters: i32, npcs: i32) -> (i32, i32) {
        self.region_monsters = self.region_monsters.wrapping_add(monsters);
        self.region_npcs = self.region_npcs.wrapping_add(npcs);
        (self.region_monsters, self.region_npcs)
    }

    fn region_object_counts(&mut self) -> (i32, i32) {
        (self.region_monsters, self.region_npcs)
    }
}
