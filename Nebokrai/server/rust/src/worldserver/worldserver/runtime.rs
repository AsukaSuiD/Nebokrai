//! Process-owned ресурсы WorldServer, общие для init, reload и main loop.
//!
//! StringTable и встроенные setup-владельцы остаются у единственного `CGame`;
//! здесь собраны исторические process-global owners, передаваемые ему ссылками.

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::convert::Infallible;
use std::io;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr as UnixSocketAddr, UnixListener};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use parking_lot::RwLock;
use chrono::{Datelike, Timelike};

use crate::dbaccess::worlddb::dbgoods::TiberiusDbGoods;
use crate::dbaccess::worlddb::dbcountry::TiberiusDbCountry;
use crate::dbaccess::worlddb::dbmisc::{CDbMisc, TiberiusDbMiscDatabase};
use crate::dbaccess::worlddb::largess::TiberiusLargess;
use crate::dbaccess::worlddb::rsenemyfactions::TiberiusRsEnemyFactions;
use crate::dbaccess::worlddb::rsfaction::TiberiusRsFaction;
use crate::dbaccess::worlddb::rsgenvar::TiberiusRsGenVar;
use crate::dbaccess::worlddb::rsgodsbattle::TiberiusRsGodsBattle;
use crate::dbaccess::worlddb::rsjjcsys::TiberiusRsJjcSys;
use crate::dbaccess::worlddb::rsplayer::{TiberiusPlayerLoadData, TiberiusRsPlayer};
use crate::dbaccess::worlddb::rsregion::{
    RegionParametersLoadOutcome, RsRegionOwner, TiberiusRsRegion,
};
use crate::dbaccess::worlddb::rssetup::{
    LoadedSetupIds, TiberiusRsSetup, WorldDatabaseSettings, WorldTdsClient,
};
use crate::dbaccess::worlddb::rsunion::TiberiusRsUnion;
use crate::dbaccess::worlddb::writelogqueue::WorldWriteLogQueue;

use crate::public::clientresource::DefaultClientResourceOwner;
use crate::public::dakongxiangqian::CDaKongXiangQian;
use crate::public::auctionlog::CAuctionLog;
use crate::public::date::TagTime;
use crate::public::timer::CTimer;
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
use crate::worldserver::appworld::country::countrywarsys::CountryWarCallbacks;
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::country::countryparam::CCountryParam;
use crate::worldserver::appworld::country::countrywarsys::CountryWarSys;
use crate::worldserver::appworld::goodswarmember::CGoodsWarMember;
use crate::worldserver::appworld::incrementlog::incrementlog::CIncrementLog;
use crate::worldserver::appworld::jjcsystem::CJJcSystem;
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
    WorldGameInitWorkerKind, WorldPlayerDataLoadOwner, WorldPlayerLoadDataAdapter,
    WorldReloadContext, WorldSaveThreadHandleState, WorldSaveThreadJob, WorldSaveThreadReport,
    save_thread_func,
};
use super::honorranks::CHonorRanks;
use super::playerranks::CPlayerRanks;
use super::worldserver::{WorldLogLocalTime, WorldLogTextOwner};
use super::savedb::{SaveDataLogPublisher, SaveDataMonitoringSnapshot};
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
    handle: Option<JoinHandle<WorldSaveWorkerCompletion>>,
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
            handle: None,
            retained_jobs: Vec::new(),
            retained_serializations: Vec::new(),
        }
    }

    pub(crate) fn serialization(&self) -> Arc<tokio::sync::Mutex<()>> {
        Arc::clone(&self.serialization)
    }

    /// Закрывает прежний завершившийся handle и запускает exact background role.
    pub(crate) fn launch(&mut self, job: WorldSaveThreadJob) -> WorldSaveThreadHandleState {
        self.collect_previous();

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
                self.handle = Some(handle);
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

    fn collect_previous(&mut self) {
        let Some(handle) = self.handle.take() else {
            return;
        };
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

    pub(crate) fn join(&mut self) -> WorldSaveThreadHandleState {
        let previous = if self.handle.is_some() {
            WorldSaveThreadHandleState::Open
        } else {
            WorldSaveThreadHandleState::Empty
        };
        self.collect_previous();
        previous
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

    pub(crate) fn db_misc(&mut self) -> Option<&mut TiberiusDbMiscDatabase> {
        self.db_misc.as_mut()
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
