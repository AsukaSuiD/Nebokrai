//! Init-контекст процесса WorldServer: двенадцать Option DB-owner-ов и их
//! соединений, Largess Arc, single-instance guard и player-load БД пула.
//!
//! Источник контракта — та же точная пара, что у [`crate::app::world_runtime`]
//! (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`,
//! RSDS совпадает). Контракт [`WorldGameInitContext`] и realm-shaped состав
//! объявлены в [`crate::app::world_init_context`], тела DB-owner-ов
//! подтверждены их модулями в `persistence/`, `organizations/`, `activities/`
//! и `regions/`; этот файл — их process-проводка.
//!
//! Process-означенный LCG `world_lcg_random` (`_rand_seed`-формула CRT
//! `rand`) обслуживает Largess-выдачу player-load пула; второй экземпляр той
//! же формулы живёт в `WorldGameInitContext::random` и читает/пишет состояние
//! самого контекста.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr as UnixSocketAddr, UnixListener};
use std::sync::Arc;
use std::time::Instant;

use chrono::Datelike;
use parking_lot::RwLock;

use nebokrai_shared::resources::{CDaKongXiangQian, GlobeSetupSnapshot};

use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::activities::rsjjcsys::TiberiusRsJjcSys;
use crate::app::world_game::WorldPlayerLoadDataAdapter;
use crate::app::world_init_context::{
    WorldDbMiscProcessConfiguration, WorldGameInitContext, WorldPlayerLoadSnapshot,
    world_db_misc_context,
};
use crate::app::world_runtime::{
    WorldGameDatabaseInitialization, WorldGameDatabaseOwner, WorldGameInitOperatorNotice,
    WorldGameInitWorkerKind,
};
use crate::characters::player::CPlayer;
use crate::characters::playerloadworker::WorldPlayerDataLoadOwner;
use crate::content::cgoodsfactory::upgrade_equipment;
use crate::content::dbgoods::TiberiusDbGoods;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::organizations::dbcountry::TiberiusDbCountry;
use crate::organizations::rsenemyfactions::TiberiusRsEnemyFactions;
use crate::organizations::rsfaction::TiberiusRsFaction;
use crate::organizations::rsunion::TiberiusRsUnion;
use crate::persistence::dbmisc::{
    DbMiscOutputPublisher, TiberiusDbMiscContext, TiberiusDbMiscDatabase,
};
use crate::persistence::largess::TiberiusLargess;
use crate::persistence::rsplayer::{TiberiusPlayerLoadData, TiberiusRsPlayer};
use crate::persistence::rsgenvar::TiberiusRsGenVar;
use crate::persistence::rssetup::{
    LoadedSetupIds, TiberiusRsSetup, WorldDatabaseSettings, WorldTdsClient,
};
use crate::persistence::writelog::WorldWriteLogCommand;
use crate::regions::rsregion::{
    RegionParameterLoadTarget, RegionParametersLoadOutcome, RsRegionOwner, TiberiusRsRegion,
};

pub struct WorldProcessPlayerLoadDatabase {
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

pub struct WorldProcessInitContext {
    runtime: tokio::runtime::Handle,
    started_at: Instant,
    pub(crate) random_state: u32,
    instance_guard: Option<UnixListener>,
    pub(crate) database_settings: Option<WorldDatabaseSettings>,
    pub(crate) log_database_settings: Option<WorldDatabaseSettings>,
    pub(crate) player: Option<TiberiusRsPlayer>,
    pub(crate) player_connection: Option<WorldTdsClient>,
    pub(crate) setup: Option<TiberiusRsSetup>,
    pub(crate) gen_var: Option<TiberiusRsGenVar>,
    pub(crate) faction: Option<TiberiusRsFaction>,
    pub(crate) union: Option<TiberiusRsUnion>,
    pub(crate) enemy_factions: Option<TiberiusRsEnemyFactions>,
    pub(crate) rs_village_war_created: bool,
    pub(crate) rs_city_war_created: bool,
    pub(crate) region: Option<TiberiusRsRegion>,
    pub(crate) country: Option<TiberiusDbCountry>,
    pub(crate) country_connection: Option<WorldTdsClient>,
    pub(crate) goods_war_connection: Option<WorldTdsClient>,
    db_misc: Option<TiberiusDbMiscDatabase>,
    pub(crate) gods_battle: Option<TiberiusRsGodsBattle>,
    pub(crate) gods_battle_connection: Option<WorldTdsClient>,
    pub(crate) log_connection: Option<WorldTdsClient>,
    pub(crate) largess: Option<Arc<TiberiusLargess>>,
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

 // Три accessor-а ниже — инвентарь process-owner контекста без текущих
 // вызывателей; в старом пакете их покрывал module-wide allow(dead_code),
 // здесь им переходный `pub` как остальному API контекста.
    pub fn country_database_owner(&mut self) -> Option<&mut TiberiusDbCountry> {
        self.country.as_mut()
    }

    pub fn gods_battle_database_owner(&mut self) -> Option<&mut TiberiusRsGodsBattle> {
        self.gods_battle.as_mut()
    }

    pub fn player_database_parts(
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

    async fn load_region_parameters(&mut self, target: &mut dyn RegionParameterLoadTarget) -> bool {
        matches!(self.region
            .as_mut()
            .expect("CRsRegion создаётся до LoadRegionParam")
            .load_region_parameters(target)
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
