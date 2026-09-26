//! Worker-вход `SaveThreadFunc` и RAII/trigger seam сохранения, перенесённые
//! из `src/worldserver/worldserver/game.rs`. Источник контракта — та же точная
//! пара, что у [`crate::persistence::savedb`] (`.exe/Nworldserver.exe` +
//! `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`, RSDS совпадает; S_PUB32
//! `?SaveThreadFunc@@YGIPAX@Z` `1:00000e30`).
//!
//! Body выполняет start-лог, полный [`do_save_data_lifecycle`] через
//! [`WorldDbDataSaveSession`], точку исходных `CoUninitialize`/unlock и end-лог
//! в прежнем порядке. `CoInitialize/CoUninitialize` не имеют Linux
//! runtime-аналога: действующие DB-owner-ы используют Tiberius, поэтому COM
//! apartment был заменяемым техническим механизмом, а не наблюдаемым серверным
//! контрактом. Единственный внешний hook — `SendErrLog` в Login — остаётся у
//! прежнего owner-а в game.rs и приходит closure-параметром, потому что его
//! `CMessage`-edge принадлежит Realm `app/worldserver` (волна monitoring-message завершена).
//!
//! Guards фиксируют точки исходных unlock: `release` замещает unlock успешного
//! пути, `stop_outer_owner` — blocked-ветку, которая исходно не достигала
//! `LeaveCriticalSection`. В Rust это типизированные токены контракта без
//! собственного исполняемого кода.
//!
//! Launch-характеристики `__beginthreadex` (`WorldSaveThreadHandleState`,
//! `WorldSaveThreadLaunchRequest`, `prepare_save_thread_launch`) лежат в
//! [`crate::app::worldserver`], frozen-вход `WorldSaveThreadJob` и lifecycle — в
//! [`crate::persistence::savedb`]; здесь динамическая граница
//! [`WorldSaveRuntimeContext`] между game-триггером и process save-owner-ом.
//! Триггер-отчёты `WorldRunSave*` живут наблюдаемостью ветвей `CGame::Run`
//! в [`crate::app::world_save_reports`] (волна C5-A); старый пакет закрепляет
//! их generic-формы alias-ами на `CGame` до её волны.

use nebokrai_shared::network::ClientSendQueue;

use crate::activities::rsgodsbattle::{
    GodsBattleFactionXydSnapshot, GodsBattleNpcFactionSnapshot, RsGodsBattleOwner,
};
use crate::activities::rsjjcsys::RsJjcSysOwner;
use crate::app::worldserver::{WorldSaveThreadHandleState, WorldSaveThreadLaunchRequest};
use crate::characters::honorranks::CHonorRanks;
use crate::characters::player::CPlayer;
use crate::content::dbgoods::DbGoodsOwner;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::variablelist::VariableListSaveSource;
use crate::organizations::dbcountry::DbCountryOwner;
use crate::organizations::rsenemyfactions::RsEnemyFactionsOwner;
use crate::organizations::rsfaction::RsFactionOwner;
use crate::organizations::rsunion::RsUnionOwner;
use crate::persistence::largess::LargessOwner;
use crate::persistence::rsgenvar::RsGenVarOwner;
use crate::persistence::rsplayer::RsPlayerOwner;
use crate::persistence::rssetup::{RsSetupOwner, WorldDatabaseSettings};
use crate::persistence::savedata::{WorldDbDataSaveSession, WorldSaveDataOwner};
use crate::persistence::savedb::{
    DoSaveDataLifecycleReport, SaveDataFinalDisposition, SaveDataFinalReport,
    SaveDataLifecycleState, SaveDataLogEvent, SaveDataLogPublishBlock,
    SaveDataLogPublishDisposition, SaveDataLogSink, SaveDataLogTarget,
    SaveDataMonitoringReport, SaveDataMonitoringSnapshot, WorldSaveThreadJob,
    do_save_data_lifecycle,
};
use crate::regions::rsregion::RsRegionOwner;

pub struct WorldSaveThreadGuard<'save> {
    save: &'save mut WorldSaveDataOwner,
}

impl<'save> WorldSaveThreadGuard<'save> {
    pub fn into_save_owner(self) -> &'save mut WorldSaveDataOwner {
        self.save
    }

    pub fn release(self) {}

    pub fn stop_outer_owner(self) {}
}

pub struct WorldRunSaveGuard<'game, G> {
    pub game: &'game mut G,
}

impl<G> WorldRunSaveGuard<'_, G> {
    pub fn release(self) {}

    pub fn stop_outer_owner(self) {}
}

pub enum WorldSaveThreadReport<'save> {
    BlockedStartLog {
        guard: WorldSaveThreadGuard<'save>,
        block: SaveDataLogPublishBlock,
    },
    BlockedLifecycle {
        guard: WorldSaveThreadGuard<'save>,
        start_log: SaveDataLogPublishDisposition,
        lifecycle: DoSaveDataLifecycleReport,
    },
    BlockedEndLog {
        start_log: SaveDataLogPublishDisposition,
        lifecycle: DoSaveDataLifecycleReport,
        block: SaveDataLogPublishBlock,
    },
    Complete {
        start_log: SaveDataLogPublishDisposition,
        lifecycle: DoSaveDataLifecycleReport,
        end_log: SaveDataLogPublishDisposition,
        exit_code: u32,
    },
}

pub trait WorldSaveRuntimeContext {
    fn try_enter_trigger(&mut self) -> bool;
    fn leave_trigger(&mut self);
    fn launch(
        &mut self,
        request: &WorldSaveThreadLaunchRequest,
        job: WorldSaveThreadJob,
    ) -> WorldSaveThreadHandleState;
}

fn save_data_lifecycle_completed(report: &DoSaveDataLifecycleReport) -> bool {
    matches!(
        report,
        DoSaveDataLifecycleReport::Final {
            report: SaveDataFinalReport {
                disposition: SaveDataFinalDisposition::Complete(_),
                ..
            },
            ..
        }
    )
}

fn save_thread_log_event(payload: &'static [u8]) -> SaveDataLogEvent {
    SaveDataLogEvent {
        target: SaveDataLogTarget::AddLogText,
        payload: payload.to_vec(),
    }
}

/// Выполняет body `SaveThreadFunc` внутри уже созданного worker-thread.
#[allow(
    clippy::too_many_arguments,
    reason = "SaveThreadFunc передаёт прежние process-global owner-ы явно"
)]
pub async fn save_thread_func<
    'save,
    S,
    O,
    V,
    P,
    J,
    G,
    U,
    F,
    R,
    B,
    E,
    C,
    L,
    Log,
    PublishState,
    ReleaseSerialization,
    GetMonitoring,
    SendErrLog,
>(
    save: &'save mut WorldSaveDataOwner,
    settings: &WorldDatabaseSettings,
    state: &mut SaveDataLifecycleState,
    variables: &S,
    registry: &GoodsBasePropertiesRegistry,
    honor_ranks: &mut CHonorRanks,
    gods_battle_faction_xyd: GodsBattleFactionXydSnapshot,
    gods_battle_npc_factions: &[GodsBattleNpcFactionSnapshot],
    use_old_save_largess_way: bool,
    setup_database: &mut O,
    variable_database: &mut V,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    union_database: &mut U,
    faction_database: &mut F,
    region_database: &mut R,
    gods_battle_database: &mut B,
    enemy_factions_database: &mut E,
    country_database: &mut C,
    largess: &mut L,
    log_sink: &mut Log,
    publish_state: PublishState,
    release_serialization: ReleaseSerialization,
    get_monitoring: GetMonitoring,
    send_err_log: SendErrLog,
) -> WorldSaveThreadReport<'save>
where
    S: VariableListSaveSource,
    O: RsSetupOwner,
    V: RsGenVarOwner,
    P: RsPlayerOwner<CPlayer>,
    J: RsJjcSysOwner,
    G: DbGoodsOwner<CPlayer>,
    U: RsUnionOwner,
    F: RsFactionOwner,
    R: RsRegionOwner,
    B: RsGodsBattleOwner,
    E: RsEnemyFactionsOwner,
    C: DbCountryOwner,
    L: LargessOwner,
    Log: SaveDataLogSink,
    PublishState: FnMut(SaveDataLifecycleState),
    ReleaseSerialization: FnOnce(),
    GetMonitoring: FnOnce() -> SaveDataMonitoringSnapshot,
    SendErrLog: FnOnce(Option<&ClientSendQueue>, &SaveDataMonitoringReport),
{
    let guard = WorldSaveThreadGuard { save };
    let start_log = match log_sink.publish(&save_thread_log_event(b"SaveThread Starting...")) {
        SaveDataLogPublishDisposition::BlockedMissingFact(block) => {
            return WorldSaveThreadReport::BlockedStartLog { guard, block };
        }
        disposition => disposition,
    };

 // DB batch и cloneable Login FIFO были сняты атомарно в trigger-позиции;
 // дальнейший MainLoop уже не разделяет с worker-ом mutable game-owner.
    let login_sender = guard.save.login_sender.as_deref();
    let lifecycle = {
        let mut session = WorldDbDataSaveSession {
            data: &mut guard.save.data,
        };
        do_save_data_lifecycle(
            settings,
            state,
            &mut session,
            variables,
            registry,
            honor_ranks,
            gods_battle_faction_xyd,
            gods_battle_npc_factions,
            use_old_save_largess_way,
            setup_database,
            variable_database,
            player_database,
            jjc_database,
            goods_database,
            union_database,
            faction_database,
            region_database,
            gods_battle_database,
            enemy_factions_database,
            country_database,
            largess,
            log_sink,
            publish_state,
            get_monitoring,
            |monitoring| send_err_log(login_sender, monitoring),
        )
        .await
    };

    if !save_data_lifecycle_completed(&lifecycle) {
        return WorldSaveThreadReport::BlockedLifecycle {
            guard,
            start_log,
            lifecycle,
        };
    }

 // Эта точка одновременно заменяет CoUninitialize и исходный unlock.
    guard.release();
    release_serialization();
    let end_log = match log_sink.publish(&save_thread_log_event(b"SaveThread end...")) {
        SaveDataLogPublishDisposition::BlockedMissingFact(block) => {
            return WorldSaveThreadReport::BlockedEndLog {
                start_log,
                lifecycle,
                block,
            };
        }
        disposition => disposition,
    };

    WorldSaveThreadReport::Complete {
        start_log,
        lifecycle,
        end_log,
        exit_code: 0,
    }
}
