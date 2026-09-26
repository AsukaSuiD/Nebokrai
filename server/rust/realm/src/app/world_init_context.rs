//! DB-слой Init-context рантайма WorldServer, перенесённый из
//! `worldserver/worldserver/runtime.rs` в Realm `app/`.
//!
//! Источник контракта — та же точная пара, что у [`crate::app::world_runtime`]
//! (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`,
//! RSDS совпадает; S_PUB32 `?Init@CGame@@QAEHXZ` `1:00017ee0` здесь только
//! привязка семейства post-init, тела DB-owner-ов подтверждаются их
//! собственными модулями в `persistence/` и `organizations/`).
//!
//! Сам держатель `WorldProcessInitContext` пока остаётся у старого пакета:
//! пять его owner-полей (`TiberiusDbCountry`, `TiberiusRsFaction`,
//! `TiberiusRsPlayer`, `TiberiusRsUnion`, `TiberiusLargess`) — concrete
//! реализации старого пакета, чьи трейты и data-семьи уже в Realm. Здесь живёт
//! realm-shaped состав этого Init-context: snapshot player-load, reloadable
//! конфигурация `DbMiscContext`, сборка самого контекста двенадцати DB-stage
//! и typed-доставка его событий. Seller-fee `get_opt_money_jin` остаётся
//! static-методом старого `CGame` и подставляется callback-ом, поэтому этот
//! файл не тянет игровой владелец и сохраняет reload-aware чтение Globe
//! snapshot.
//!
//! Волной C5-B сюда перенесён и сам контракт [`WorldGameInitContext`] из
//! `worldserver/worldserver/game.rs`. Единственный CGame-типизированный метод
//! исходной формы, `load_region_parameters`, получил готовый шов `&mut dyn
//! RegionParameterLoadTarget` (`regions/rsregion.rs`): process owner пробрасывает
//! target напрямую в `RsRegionOwner::load_region_parameters`, не зная типа игры.
//! Impl контракта остаётся у старого владельца (`runtime.rs`) до дорожки C5-DB.

use std::io;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;

use nebokrai_shared::resources::{CPlayerList, CThingSetup, GlobeSetupSnapshot};

use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::app::world_runtime::{
    WorldGameDatabaseInitialization, WorldGameDatabaseOwner, WorldGameInitOperatorNotice,
    WorldGameInitWorkerKind,
};
use crate::auction::auctionnode::CGoodsNode;
use crate::characters::honorranks::HonorRanksDbOwner;
use crate::characters::player::{CPlayer, PlayerPropertyCoefficients};
use crate::characters::playerloadworker::WorldPlayerDataLoadOwner;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::organizations::dbcountry::DbCountryOwner;
use crate::organizations::rsenemyfactions::RsEnemyFactionsOwner;
use crate::organizations::rsfaction::RsFactionOwner;
use crate::organizations::rsunion::RsUnionOwner;
use crate::persistence::dbmisc::{
    AuctionGoodsLoadOutcome, AuctionMoneyLoadOutcome, DbMiscOutputPublisher,
    TiberiusDbMiscCallbacks, TiberiusDbMiscContext, TiberiusDbMiscDatabase,
    TiberiusDbMiscRuntimeEvent,
};
use crate::persistence::rsgenvar::RsGenVarOwner;
use crate::persistence::rsplayer::RsPlayerOwner;
use crate::persistence::rssetup::{LoadedSetupIds, WorldTdsClient};
use crate::persistence::writelogqueue::WorldWriteLogQueue;
use crate::regions::rsregion::RegionParameterLoadTarget;

#[derive(Clone)]
pub struct WorldPlayerLoadSnapshot {
    pub thing_setup: CThingSetup,
    pub player_list: CPlayerList,
    pub globe_setup: GlobeSetupSnapshot,
    pub coefficients: PlayerPropertyCoefficients,
    pub goods: Arc<GoodsBasePropertiesRegistry>,
    pub gold_coin_index: u32,
    pub gold_coin_limit: u32,
    pub use_log_system: bool,
    pub write_log_queue: WorldWriteLogQueue,
}

/// Собирает долгоживущий `TiberiusDbMiscContext` и reloadable конфигурацию
/// настройками, которые `CGame::Init` уже опубликовал. `random_state` здесь
/// копия process-счётчика: контекст наращивает своё состояние LCG, не
/// разделяя мутацию с Init-context — исходный UXCriticalSection/seed input
/// на эти вызовы не влиял.
#[allow(clippy::too_many_arguments)]
pub fn world_db_misc_context(
    runtime: tokio::runtime::Handle,
    database: TiberiusDbMiscDatabase,
    registry: GoodsBasePropertiesRegistry,
    globe_setup: GlobeSetupSnapshot,
    gold_coin_index: u32,
    output: DbMiscOutputPublisher,
    started_at: Instant,
    random_state: u32,
    seller_fee: Box<dyn Fn(&GlobeSetupSnapshot, u32) -> Option<i32>>,
) -> (TiberiusDbMiscContext, WorldDbMiscProcessConfiguration) {
    let configuration = WorldDbMiscProcessConfiguration::new(
        globe_setup,
        gold_coin_index,
    );
    let shared = configuration.shared();
    let transfer_configuration = Arc::clone(&shared);
    let seller_configuration = Arc::clone(&shared);
    let gold_configuration = Arc::clone(&shared);
    let mut random_state = random_state;
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
            seller_fee(
                &seller_configuration.read().globe_setup,
                seller_money,
            )
        }),
        gold_coin_index: Box::new(move || gold_configuration.read().gold_coin_index),
        random: Box::new(move |upper_bound| {
            random_state = random_state.wrapping_mul(214013).wrapping_add(2531011);
            let value = ((random_state >> 16) & 0x7fff) as i32;
            if upper_bound > 0 { value % upper_bound } else { 0 }
        }),
    };
    let context = TiberiusDbMiscContext::new(runtime, database, registry, output, callbacks);
    (context, configuration)
}

#[derive(Clone)]
struct WorldDbMiscProcessConfigurationSnapshot {
    globe_setup: GlobeSetupSnapshot,
    gold_coin_index: u32,
}

/// Reloadable значения, которые один долгоживущий `DbMiscContext` читает в
/// каждом MainLoop turn, не сохраняя расходящуюся копию Globe/goods ID.
#[derive(Clone)]
pub struct WorldDbMiscProcessConfiguration {
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

    pub fn publish(
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

/// Typed-доставка событий аукционного DB-контекста в process-лог: те же
/// строки, что писала прежняя glue-обвязка, но источником остаётся
/// `TiberiusDbMiscRuntimeEvent` из `persistence/dbmisc`.
pub fn report_db_misc_runtime_event(event: TiberiusDbMiscRuntimeEvent<'_>) {
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

/// Контракт process-уровня `CGame::Init` между драйвером хода
/// (`app/world_runtime`) и concrete owner-ами процесса.
///
/// Перенесён из `worldserver/worldserver/game.rs` волной C5-B. CGame-typed
/// параметр `load_region_parameters` исходной формы заменён готовым швом
/// `&mut dyn RegionParameterLoadTarget`; impl остаётся у старого process
/// owner-а (`runtime.rs`) до дорожки C5-DB.
#[allow(
    async_fn_in_trait,
    reason = "буквальный перенос pub(crate)-контракта init-стадий: единственные \
              потребители — драйвер хода и impl process-owner-а; dyn-объект и \
              Send-ограничения контрактом не требуются"
)]
pub trait WorldGameInitContext {
    type Block;
    type PlayerDatabase: RsPlayerOwner<CPlayer> + HonorRanksDbOwner;
    type EnemyFactionsDatabase: RsEnemyFactionsOwner;
    type GeneralVariableDatabase: RsGenVarOwner;
    type UnionDatabase: RsUnionOwner;
    type FactionDatabase: RsFactionOwner;
    type CountryDatabase: DbCountryOwner;
    type PlayerLoadDatabase: WorldPlayerDataLoadOwner<CPlayer> + Send + 'static;
    type PlayerLoadLargess: FnMut(&mut CPlayer) + Send + 'static;
    type PlayerLoadClock: FnMut() -> u32 + Send + 'static;

    fn install_crash_reporter(&mut self);
    fn current_time_seconds(&mut self) -> i64;
    fn seed_random(&mut self, seed: u32);
    fn random(&mut self, upper_bound: i32) -> i32;
    fn put_debug_string(&mut self, payload: &[u8]);
    fn claim_single_instance(&mut self, title: &[u8]) -> bool;
    fn notify_operator(&mut self, notice: &WorldGameInitOperatorNotice);

    fn initialize_database_layer(
        &mut self,
        initialization: WorldGameDatabaseInitialization,
    ) -> Result<(), Self::Block>;
    async fn create_database_owner(
        &mut self,
        owner: WorldGameDatabaseOwner,
    ) -> Result<(), Self::Block>;
    async fn create_rs_setup_owner(&mut self) -> Result<LoadedSetupIds, Self::Block>;

    async fn load_region_parameters(
        &mut self,
        target: &mut dyn RegionParameterLoadTarget,
    ) -> bool;
    fn player_database(
        &mut self,
    ) -> (&mut Self::PlayerDatabase, Option<&mut WorldTdsClient>);
    fn enemy_factions_database(&mut self) -> &mut Self::EnemyFactionsDatabase;
    fn general_variable_database(&mut self) -> &mut Self::GeneralVariableDatabase;
    fn organizing_databases(&mut self) -> (&mut Self::UnionDatabase, &mut Self::FactionDatabase);
    fn country_database(
        &mut self,
    ) -> (&mut Self::CountryDatabase, Option<&mut WorldTdsClient>);
    fn goods_war_database_connection(&mut self) -> Option<&mut WorldTdsClient>;
 /// Возвращает созданный `CRSGodsBattle`; до соответствующего create-event
 /// owner закономерно отсутствует.
    fn gods_battle_database(&mut self) -> Option<&mut TiberiusRsGodsBattle>;
    fn increment_log_database(&mut self) -> Option<&mut WorldTdsClient>;
    fn auction_log_database(&mut self) -> Option<&mut WorldTdsClient>;
 /// Даёт каждому concrete worker-у собственные Send-owner-ы; handle остаётся
 /// внутри единственного `CGame` и освобождается его Release.
    fn player_load_worker_runtime(
        &mut self,
        worker_index: u32,
    ) -> (
        tokio::runtime::Handle,
        Self::PlayerLoadDatabase,
        Self::PlayerLoadLargess,
        Self::PlayerLoadClock,
    );
    fn write_log_worker_runtime(&mut self) -> tokio::runtime::Handle;
    fn report_worker_spawn_error(&mut self, kind: WorldGameInitWorkerKind, error: &io::Error);
}
