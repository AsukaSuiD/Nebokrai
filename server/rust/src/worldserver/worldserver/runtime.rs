//! Process-owned ресурсы WorldServer, общие для init, reload и main loop.
//!
//! StringTable и встроенные setup-владельцы остаются у единственного `CGame`;
//! здесь собраны исторические process-global owners, передаваемые ему ссылками.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::future::Future;
use std::convert::Infallible;
use std::fmt;
use std::io;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr as UnixSocketAddr, UnixListener};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use parking_lot::{Mutex, RwLock};
use chrono::{Datelike, Timelike};

use crate::dbaccess::worlddb::dbgoods::TiberiusDbGoods;
use crate::dbaccess::worlddb::dbcountry::TiberiusDbCountry;
use crate::dbaccess::worlddb::dbmisc::{
    CDbMisc, DbMiscOutputPublisher, TiberiusDbMiscContext, TiberiusDbMiscDatabase,
};
use crate::dbaccess::worlddb::largess::TiberiusLargess;
use crate::dbaccess::worlddb::rsenemyfactions::TiberiusRsEnemyFactions;
use crate::dbaccess::worlddb::rsfaction::TiberiusRsFaction;
use crate::dbaccess::worlddb::rsgenvar::TiberiusRsGenVar;
use crate::dbaccess::worlddb::rsgodsbattle::TiberiusRsGodsBattle;
use crate::dbaccess::worlddb::rsjjcsys::TiberiusRsJjcSys;
use crate::dbaccess::worlddb::rsplayer::{
    TiberiusPlayerLoadData, TiberiusRsPlayer,
};
use nebokrai_realm::activities::leitingreset::LeiTingDatabaseResetRequest;
pub(crate) use nebokrai_realm::app::world_network::{
    WorldProcessNetworkError, WorldProcessNetworkTurn, report_world_network_turn,
};
pub(crate) use nebokrai_realm::app::world_init_context::*;
pub(crate) use nebokrai_realm::app::world_main_loop_contexts::*;
use nebokrai_realm::app::world_network::WorldProcessNetworkRuntime as RealmProcessNetworkRuntime;
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
use crate::public::dakongxiangqian::CDaKongXiangQian;
use crate::public::auctionlog::CAuctionLog;
use crate::public::date::TagTime;
use crate::public::timer::CTimer;
use crate::nets::networld::mynetclient::CMyNetClient;
use crate::nets::networld::mynetserver::CMyNetServer;
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
use nebokrai_shared::resources::CPlayerList;
use crate::setup::preciousboxconf::PreciousBoxConf;
use crate::setup::regionrouter::RegionRouter;
use crate::setup::regionsetup::CRegionSetup;
use crate::setup::synthesis::CSynthesis;
use crate::worldserver::appworld::goods::cbattlefairyproperty::CBattleFairyProperty;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    release_goods_registry, upgrade_equipment, GoodsBasePropertiesRegistry, GoodsNameIndex,
    GoodsOriginalNameIndex,
};
use crate::worldserver::appworld::player::CPlayer;
use crate::worldserver::appworld::message::organsysmessage::WorldUnionApplicationRuntimeOwner;
use crate::worldserver::appworld::session::csessionfactory::CSessionFactory;
use crate::worldserver::appworld::country::countrywarsys::CountryWarCallbacks;
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::country::countryparam::CCountryParam;
use crate::worldserver::appworld::country::countrywarsys::CountryWarSys;
use crate::worldserver::appworld::goodswarmember::CGoodsWarMember;
use crate::worldserver::appworld::incrementlog::incrementlog::CIncrementLog;
use crate::worldserver::appworld::jjcsystem::CJJcSystem;
use crate::worldserver::appworld::leiting::CLeiTing;
use crate::worldserver::appworld::organizingsystem::attackcitysys::AttackCityCallbacks;
use crate::worldserver::appworld::organizingsystem::attackcitysys::CAttackCitySys;
use crate::worldserver::appworld::organizingsystem::factionwarsys::CFactionWarSys;
use crate::worldserver::appworld::organizingsystem::fournationwarsys::FourNationWarCallbacks;
use crate::worldserver::appworld::organizingsystem::fournationwarsys::CFourNationWarSys;
use crate::worldserver::appworld::organizingsystem::organizingctrl::COrganizingCtrl;
use crate::worldserver::appworld::organizingsystem::organizingparam::{
    COrganizingParam, OrganizingParamReleaseReport,
};
use crate::worldserver::appworld::organizingsystem::villagewarsys::VillageWarCallbacks;
use crate::worldserver::appworld::organizingsystem::villagewarsys::CVillageWarSys;
use crate::worldserver::appworld::script::variablelist::CVariableList;
use crate::worldserver::appworld::skills::skillfactory::CSkillFactory;
use crate::worldserver::appworld::misc::CopyNumberTimerState;
use crate::worldserver::appworld::worldregion::WorldRegionResourceContext;

use super::game::{
    CGame, WorldGameDatabaseInitialization, WorldGameDatabaseOwner, WorldGameInitContext,
    WorldGameInitCallbacks, WorldGameInitOperatorNotice, WorldGameInitResult,
    WorldCollectPlayerDataRequestState, WorldGameInitWorkerKind, WorldGameReleaseContext,
    WorldGameReleaseDatabaseOwner, WorldGameReleaseOptionalOwner, WorldGameReleaseVoidOwner,
    WorldJjcRuntimeContext,
    WorldLeiTingRuntimeContext, WorldMainLoopClockState, WorldMainLoopInitializationState,
    WorldGameThreadRuntime, WorldMainLoopBlock, WorldMainLoopCallbacks,
    WorldMainLoopConfiguration, WorldMainLoopOwners, WorldMainLoopResourceContext,
    WorldMainLoopResourceSnapshot, WorldRefreshExternalCounts,
    WorldMainLoopLargessState, WorldMainLoopLoginReleaseState, WorldMainLoopProfileState,
    WorldMainLoopStateOwners, WorldMainLoopTailClockState,
    WorldPlayerLoadDataAdapter, WorldPlayerRanksRequestState, WorldProcessMessageStageState,
    WorldRegionOwner, WorldReloadContext, WorldReloadProfileFlags, WorldRunSaveTriggerState,
    WorldSaveRuntimeContext, WorldSaveThreadHandleState,
    WorldSaveThreadJob, WorldSaveThreadLaunchRequest, WorldSaveThreadReport, save_thread_func,
};
use super::honorranks::CHonorRanks;
use super::playerranks::{CPlayerRanks, PlayerRanksReleaseReport};
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
use super::playerloadworker::WorldPlayerDataLoadOwner;
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

/// Process-runtime обвязка Realm `app::world_network`: два mut-accessor-а
/// `CGame` не вызываются одновременно, поэтому владельцы направлений
/// подаются стадиями, а ход собирается в исходном порядке — сначала
/// worker принятых GameServer, затем неблокирующий опрос Login.
pub(crate) struct WorldProcessNetworkRuntime {
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
            .run_game_server_worker(
                game.process_game_server_mut(),
                super::game::legacy_tick_ms,
            )
            .await?;
        let mut turn = WorldProcessNetworkTurn::default();
        turn.login = RealmProcessNetworkRuntime::poll_login_client(
            game.process_login_client_mut(),
            super::game::legacy_tick_ms,
        )
        .await;
        Ok(turn)
    }

    pub(crate) async fn release_server(
        &mut self,
        server: &mut CMyNetServer,
    ) -> Result<(), WorldProcessNetworkError> {
        self.inner
            .release_server(server, super::game::legacy_tick_ms)
            .await
    }

    pub(crate) fn release_client(&mut self, client: &mut CMyNetClient) {
        self.inner.release_client(client);
    }
}

pub(crate) struct WorldProcessReleaseContext<'a> {
    runtime: tokio::runtime::Handle,
    runtime_directory: &'a Path,
    network: &'a mut WorldProcessNetworkRuntime,
    domains: &'a mut WorldProcessDomainOwners,
    init: &'a mut WorldProcessInitContext,
    resources: &'a mut WorldProcessResources,
    save: Option<&'a mut WorldProcessSaveRuntime>,
}

impl<'a> WorldProcessReleaseContext<'a> {
    pub(crate) fn new(
        runtime: tokio::runtime::Handle,
        runtime_directory: &'a Path,
        network: &'a mut WorldProcessNetworkRuntime,
        domains: &'a mut WorldProcessDomainOwners,
        init: &'a mut WorldProcessInitContext,
        resources: &'a mut WorldProcessResources,
        save: Option<&'a mut WorldProcessSaveRuntime>,
    ) -> Self {
        Self {
            runtime,
            runtime_directory,
            network,
            domains,
            init,
            resources,
            save,
        }
    }
}

impl WorldGameReleaseContext for WorldProcessReleaseContext<'_> {
    fn put_debug_string(&mut self, payload: &'static [u8]) {
        eprintln!("WorldServer: {}", String::from_utf8_lossy(payload));
    }

    fn save_city_region(&mut self, region_id: i32, region: &mut WorldRegionOwner) {
        match region.save_to_resource_directory(self.runtime_directory) {
            Ok(0) => eprintln!(
                "WorldServer: файл региона {region_id} не открыт во время shutdown"
            ),
            Ok(_) => {}
            Err(error) => eprintln!(
                "WorldServer: регион {region_id} не сохранён во время shutdown: {error:?}"
            ),
        }
    }

    fn exit_network_server_worker(&mut self, server: &mut CMyNetServer) {
        let runtime = self.runtime.clone();
        let result = tokio::task::block_in_place(|| {
            runtime.block_on(self.network.release_server(server))
        });
        if let Err(error) = result {
            eprintln!("WorldServer: GameServer network-worker не завершён: {error}");
        }
    }

    fn exit_network_client_worker(&mut self, client: &mut CMyNetClient) {
        self.network.release_client(client);
    }

    fn release_void_owner(&mut self, owner: WorldGameReleaseVoidOwner) {
        match owner {
            WorldGameReleaseVoidOwner::UninitializeTimeToReturn => {
                self.domains.time_to_return = TimeToReturn::new();
            }
            WorldGameReleaseVoidOwner::ReleaseCountryHandler => {
                let previous = std::mem::take(&mut self.domains.country_handler);
                let _ = previous.release();
            }
            WorldGameReleaseVoidOwner::ReleaseOrganizingController => {
                let previous = std::mem::replace(
                    &mut self.domains.organizing,
                    COrganizingCtrl::with_reached_callback_state(),
                );
                let _ = previous.release(&mut self.domains.timer);
            }
            WorldGameReleaseVoidOwner::ReleaseAttackCity => {
                self.domains.attack_city = CAttackCitySys::new();
            }
            WorldGameReleaseVoidOwner::ReleaseVillageWar => {
                self.domains.village_war = CVillageWarSys::new();
            }
            WorldGameReleaseVoidOwner::ReleaseFactionWar => {
                self.domains.faction_war = CFactionWarSys::new(0);
            }
            WorldGameReleaseVoidOwner::ReleaseTimer => {
                self.domains.timer = CTimer::new();
            }
            WorldGameReleaseVoidOwner::ReleaseGoodsFactory => {
                release_goods_registry(
                    Arc::make_mut(&mut self.resources.goods),
                    Arc::make_mut(&mut self.resources.goods_by_original_name),
                    &mut self.resources.goods_by_name,
                );
            }
            WorldGameReleaseVoidOwner::ReleaseNetSessionManager => {
                let _ = self.domains.net_sessions.release();
            }
            WorldGameReleaseVoidOwner::UninitializeLargess => {
                drop(self.init.largess.take());
            }
            WorldGameReleaseVoidOwner::UninitializeDatabaseLayer => {
                drop(self.init.player_connection.take());
                drop(self.init.country_connection.take());
                drop(self.init.goods_war_connection.take());
                drop(self.init.gods_battle_connection.take());
                drop(self.init.log_connection.take());
                drop(self.init.database_settings.take());
                drop(self.init.log_database_settings.take());
            }
            WorldGameReleaseVoidOwner::ReleaseGoodsLinks
            | WorldGameReleaseVoidOwner::UninitializeIncrementLog
            | WorldGameReleaseVoidOwner::ReleaseWordsFilter
            | WorldGameReleaseVoidOwner::ReleaseQuestSystem
            | WorldGameReleaseVoidOwner::ClearSkillCache
            | WorldGameReleaseVoidOwner::ClearSkillUsageCache
            | WorldGameReleaseVoidOwner::CleanupSocket
            | WorldGameReleaseVoidOwner::ReleaseBaseMessage
            | WorldGameReleaseVoidOwner::RequestWriteLogWorkerExit => {}
        }
    }

    fn release_optional_owner(&mut self, owner: WorldGameReleaseOptionalOwner) -> bool {
        match owner {
            WorldGameReleaseOptionalOwner::GeneralVariableList => {
                self.domains.general_variables.take().is_some()
            }
            WorldGameReleaseOptionalOwner::DefaultClientResource => {
                self.resources.default_client_resource.clear()
            }
            WorldGameReleaseOptionalOwner::FunctionListFileData
            | WorldGameReleaseOptionalOwner::VariableListFileData
            | WorldGameReleaseOptionalOwner::ScriptFileData
            | WorldGameReleaseOptionalOwner::DupliRegionSetup => false,
        }
    }

    fn release_database_owner(&mut self, owner: WorldGameReleaseDatabaseOwner) -> bool {
        match owner {
            WorldGameReleaseDatabaseOwner::RsPlayer => self.init.player.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsSetup => self.init.setup.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsGenVar => self.init.gen_var.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsFaction => self.init.faction.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsUnion => self.init.union.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsEnemyFactions => {
                self.init.enemy_factions.take().is_some()
            }
            WorldGameReleaseDatabaseOwner::RsVillageWar => {
                std::mem::take(&mut self.init.rs_village_war_created)
            }
            WorldGameReleaseDatabaseOwner::RsCityWar => {
                std::mem::take(&mut self.init.rs_city_war_created)
            }
            WorldGameReleaseDatabaseOwner::GoodsWarMember => false,
            WorldGameReleaseDatabaseOwner::RsRegion => self.init.region.take().is_some(),
            WorldGameReleaseDatabaseOwner::DbCountry => self.init.country.take().is_some(),
            WorldGameReleaseDatabaseOwner::RsGodsBattle => {
                self.init.gods_battle.take().is_some()
            }
        }
    }

    fn release_player_ranks(&mut self) -> PlayerRanksReleaseReport {
        self.domains.player_ranks.release(&mut self.domains.timer)
    }

    fn release_organizing_parameters(&mut self) -> OrganizingParamReleaseReport {
        std::mem::take(&mut self.domains.organizing_parameters)
            .release(&mut self.domains.timer)
    }

    fn join_save_worker(&mut self) -> WorldSaveThreadHandleState {
        self.save
            .as_deref_mut()
            .map_or(WorldSaveThreadHandleState::Empty, WorldProcessSaveRuntime::join)
    }
}

pub(crate) struct WorldSaveWorkerCompletion {
    pub(crate) retained_job: Option<WorldSaveThreadJob>,
    retained_serialization: Option<tokio::sync::OwnedMutexGuard<()>>,
}

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

 /// Запускает очередной background role, не ожидая предыдущий поток.
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

    pub(crate) fn after_game_init(
        runtime: tokio::runtime::Handle,
        init: &WorldProcessInitContext,
        domains: &WorldProcessDomainOwners,
    ) -> Result<Self, WorldMainLoopContextBuildError> {
        let settings = init
            .database_settings()
            .ok_or(WorldMainLoopContextBuildError::MissingDatabaseSettings)?;
        let largess = init
            .largess()
            .ok_or(WorldMainLoopContextBuildError::MissingLargessOwner)?;
        Ok(Self::new(WorldSaveWorker::new(
            runtime,
            init.started_at(),
            settings,
            largess,
            domains.log.clone(),
        )))
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
 // World binary только конструирует/читает static map; кроме
 // CRT teardown записей в неё нет, поэтому shipped process начинает
 // и остаётся с пустой таблицей замен индексов.
            changed_goods_indices: BTreeMap::new(),
            dakong_addon_types,
        }
    }
}

fn world_lcg_random(random_state: &Cell<u32>, upper_bound: i32) -> i32 {
    let state = random_state
        .get()
        .wrapping_mul(214013)
        .wrapping_add(2531011);
    random_state.set(state);
    let value = ((state >> 16) & 0x7fff) as i32;
    if upper_bound > 0 {
        value % upper_bound
    } else {
        0
    }
}

pub(crate) fn world_player_load_largess(
    largess: Arc<TiberiusLargess>,
    snapshot: Arc<RwLock<WorldPlayerLoadSnapshot>>,
) -> Box<dyn FnMut(&mut CPlayer) + Send> {
 // Machine random (`0x453560`) — process-global CRT rand; create/upgrade
 // пути делят одну последовательность, поэтому состояние общее через Cell.
    let random_state = Cell::new(1u32);
    Box::new(move |player| {
        let snapshot = snapshot.read().clone();
        let mut random = |upper_bound: i32| {
            world_lcg_random(&random_state, upper_bound)
        };
        let mut upgrade = |goods: &mut _, target_level| {
            let mut random = |upper_bound: i32| {
                world_lcg_random(&random_state, upper_bound)
            };
            let _ = upgrade_equipment(Some(goods), &snapshot.goods, target_level, &mut random);
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
    rs_village_war_created: bool,
    rs_city_war_created: bool,
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
            rs_village_war_created: false,
            rs_city_war_created: false,
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
        output: DbMiscOutputPublisher,
        seller_fee: Box<dyn Fn(&GlobeSetupSnapshot, u32) -> Option<i32>>,
    ) -> Option<(TiberiusDbMiscContext, WorldDbMiscProcessConfiguration)> {
        let database = self.db_misc.take()?;
        Some(world_db_misc_context(
            self.runtime.clone(),
            database,
            registry,
            globe_setup,
            gold_coin_index,
            output,
            self.started_at,
            self.random_state,
            seller_fee,
        ))
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

impl WorldJjcRuntimeContext for WorldJjcProcessContext {
    fn on_week_clear_spawn_failed(&mut self, error: io::Error) {
        eprintln!("WorldServer: не создан JJC week-clear worker: {error}");
    }

    fn on_week_clear_worker_event(&mut self, event: WorldJjcWeekClearWorkerEvent) {
        eprintln!("WorldServer: JJC DB worker: {event:?}");
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
                self.add_log(b"Strictest Enforcement update thread begin.")
            }
            WorldLeiTingResetWorkerEvent::Finished { outcome, .. } => {
                eprintln!("WorldServer: LeiTing DB worker завершён: {outcome:?}")
            }
        }
    }
}


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
        let snapshot = resources.player_load_snapshot.read().clone();
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
            WorldGameDatabaseOwner::RsVillageWar => {
                self.rs_village_war_created = true;
            }
            WorldGameDatabaseOwner::RsCityWar => {
                self.rs_city_war_created = true;
            }
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

impl WorldPlayerDataLoadOwner<CPlayer> for WorldProcessPlayerLoadDatabase {
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
    goods: Arc<GoodsBasePropertiesRegistry>,
    goods_by_original_name: Arc<GoodsOriginalNameIndex>,
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
            goods: Arc::new(GoodsBasePropertiesRegistry::default()),
            gold_coin_index: 0,
            gold_coin_limit: 0,
            use_log_system: false,
            write_log_queue: WorldWriteLogQueue::default(),
        }));
        Self {
            runtime_directory,
            default_client_resource: Default::default(),
            goods: Arc::new(Default::default()),
            goods_by_original_name: Arc::new(Default::default()),
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
            Arc::make_mut(&mut self.goods),
            Arc::make_mut(&mut self.goods_by_original_name),
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
            goods: Arc::clone(&self.goods),
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

impl WorldMainLoopResourceContext for WorldProcessResources {
    fn main_loop_resource_snapshot(&self) -> WorldMainLoopResourceSnapshot {
        let gold_coin_index = self.player_load_snapshot.read().gold_coin_index;
        WorldMainLoopResourceSnapshot {
            registry: Arc::clone(&self.goods),
            original_name_index: Arc::clone(&self.goods_by_original_name),
            coefficients: self.globe_setup.player_property_coefficients().into(),
            player_list: self.player_list.clone(),
            globe_setup: self.globe_setup.clone(),
            region_router: self.region_router.clone(),
            log_system: self.log_system.clone(),
            gold_coin_index,
        }
    }
}

#[derive(Clone)]
pub(crate) struct WorldProcessControl {
    exit_requested: Arc<AtomicBool>,
}

impl WorldProcessControl {
    pub(crate) fn request_exit(&self) {
        self.exit_requested.store(true, Ordering::Release);
    }
}

pub(crate) enum WorldProcessMainLoopBlock {
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

pub(crate) struct WorldProcessRuntime {
    runtime: tokio::runtime::Handle,
    runtime_directory: PathBuf,
    domains: WorldProcessDomainOwners,
    init: WorldProcessInitContext,
    resources: WorldProcessResources,
    network: WorldProcessNetworkRuntime,
    main_loop_state: WorldProcessMainLoopState,
    contexts: Option<WorldProcessMainLoopContexts>,
    save: Option<WorldProcessSaveRuntime>,
    post_init_error: Option<WorldMainLoopContextBuildError>,
    exit_requested: Arc<AtomicBool>,
    game_thread_exited: bool,
    window_close_requested: bool,
 /// Статический `ReMsg` исходно заполнен нулями; в точном EXE найден только reader,
 /// поэтому owner сохраняется отдельно и остаётся нулём до появления
 /// исходного producer-а.
    reback_messages: i32,
}

impl WorldProcessRuntime {
    pub(crate) fn new(
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

    fn release_context(&mut self) -> WorldProcessReleaseContext<'_> {
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

    async fn run_connected_main_loop(
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

        let started_at = self.init.started_at;
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
                if !matches!(outcome, super::game::WorldOwnedCityRefreshOutcome::Refreshed(_)) {
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

impl WorldGameReleaseContext for WorldProcessRuntime {
    fn put_debug_string(&mut self, payload: &'static [u8]) {
        self.release_context().put_debug_string(payload)
    }

    fn save_city_region(&mut self, region_id: i32, region: &mut WorldRegionOwner) {
        self.release_context().save_city_region(region_id, region)
    }

    fn exit_network_server_worker(&mut self, server: &mut CMyNetServer) {
        self.release_context().exit_network_server_worker(server)
    }

    fn exit_network_client_worker(&mut self, client: &mut CMyNetClient) {
        self.release_context().exit_network_client_worker(client)
    }

    fn release_void_owner(&mut self, owner: WorldGameReleaseVoidOwner) {
        self.release_context().release_void_owner(owner)
    }

    fn release_optional_owner(&mut self, owner: WorldGameReleaseOptionalOwner) -> bool {
        self.release_context().release_optional_owner(owner)
    }

    fn release_database_owner(&mut self, owner: WorldGameReleaseDatabaseOwner) -> bool {
        self.release_context().release_database_owner(owner)
    }

    fn release_player_ranks(&mut self) -> PlayerRanksReleaseReport {
        self.release_context().release_player_ranks()
    }

    fn release_organizing_parameters(&mut self) -> OrganizingParamReleaseReport {
        self.release_context().release_organizing_parameters()
    }

    fn join_save_worker(&mut self) -> WorldSaveThreadHandleState {
        self.release_context().join_save_worker()
    }
}

impl WorldGameThreadRuntime for WorldProcessRuntime {
    type InitBlock = Infallible;
    type MainLoopBlock = WorldProcessMainLoopBlock;

    fn initialize_game<'game>(
        &'game mut self,
        game: &'game mut CGame,
    ) -> Pin<Box<dyn Future<Output = WorldGameInitResult<Self::InitBlock>> + 'game>> {
        Box::pin(async move {
            let result = self
                .domains
                .initialize_game(
                    game,
                    &self.runtime_directory,
                    &mut self.init,
                    &mut self.resources,
                )
                .await;
            if result.is_ok() {
                match WorldProcessSaveRuntime::after_game_init(
                    self.runtime.clone(),
                    &self.init,
                    &self.domains,
                ) {
                    Ok(save) => self.save = Some(save),
                    Err(error) => self.post_init_error = Some(error),
                }
                if self.post_init_error.is_none() {
                    match WorldProcessMainLoopContexts::after_game_init(
                        self.runtime.clone(),
                        game,
                        &mut self.init,
                        &mut self.resources,
                        &self.domains,
                    ) {
                        Ok(contexts) => self.contexts = Some(contexts),
                        Err(error) => self.post_init_error = Some(error),
                    }
                }
            }
            result
        })
    }

    fn game_thread_exit_requested(&self) -> bool {
        self.exit_requested.load(Ordering::Acquire)
    }

    fn run_main_loop<'game>(
        &'game mut self,
        game: &'game mut CGame,
    ) -> Pin<Box<dyn Future<Output = Result<i32, Self::MainLoopBlock>> + 'game>> {
        Box::pin(async move {
            if let Some(error) = self.post_init_error.take() {
                return Err(WorldProcessMainLoopBlock::PostInit(error));
            }
            self.run_connected_main_loop(game).await
        })
    }

    fn wait_for_save_barrier(&mut self) {
        if let Some(save) = self.save.as_ref() {
            save.wait_for_barrier();
        }
    }

    fn take_goods_war_member(&mut self) -> CGoodsWarMember {
        std::mem::replace(
            &mut self.domains.goods_war,
            CGoodsWarMember::with_reached_empty_state(),
        )
    }

    fn restore_goods_war_member(&mut self, owner: CGoodsWarMember) {
        self.domains.goods_war = owner;
    }

    fn take_increment_log(&mut self) -> CIncrementLog {
        std::mem::replace(&mut self.domains.increment_log, CIncrementLog::new())
    }

    fn restore_increment_log(&mut self, owner: CIncrementLog) {
        self.domains.increment_log = owner;
    }

    fn take_skill_factory(&mut self) -> CSkillFactory {
        std::mem::take(&mut self.domains.skills)
    }

    fn restore_skill_factory(&mut self, owner: CSkillFactory) {
        self.domains.skills = owner;
    }

    fn signal_game_thread_exit(&mut self) {
        self.game_thread_exited = true;
    }

    fn request_window_close(&mut self) {
        self.window_close_requested = true;
    }
}
