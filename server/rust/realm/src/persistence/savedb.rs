//! Оркестрация сохранения из `worldserver/savedb.cpp/.h`, перенесённая в Realm
//! `persistence/`. Источник контракта — та же точная пара, что у
//! [`crate::persistence::savedata`] (`.exe/Nworldserver.exe` +
//! `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`, RSDS совпадает).
//!
//! Машинно подтверждённые точки (S_PUB32 `.exe/Nworldserver.exe`, первая
//! секция): свободный `?DoSaveData@@YAXXZ` `1:0001b610`, variadic gate
//! `?ShowSaveInfo@@YAXPBDZZ` `1:00000720` с глобалом `?g_bShowSaveInfo@@3_NA`
//! и Login-ответ `?SendErrLog@@YAXDJJPBD@Z` `1:00000f30`; worker-вход
//! `?SaveThreadFunc@@YGIPAX@Z` `1:00000e30` собирает job у process-owner-а.
//!
//! Владелец собирает snapshots игроков, стран, регионов, товаров, организаций,
//! JJC, GodsBattle, variables и Largess и передаёт их соответствующим World DB
//! owners. Порядок стадий, отдельные соединения, SQL/procedures и отсутствие
//! общей транзакции сохранены; поздняя ошибка не откатывает успешный префикс.
//!
//! Судьба записи после ошибки зависит от исходной позиции pop: уже извлечённая
//! теряется, оставленная до успешного результата остаётся в очереди. Save worker
//! публикует barrier, а `Release` обязательно ждёт его до уничтожения игровых
//! данных. `Mutex`, `Condvar`, `JoinHandle`, typed snapshots и Tiberius заменяют
//! Win32/ADO инфраструктуру без изменения error mapping.
//!
//! `show_save_info` перенесён из `worldserver/worldserver/game.rs` вместе со
//! своей save-публикующей семьёй: это тот же gate над
//! [`crate::app::worldserver::WorldLogTextOwner`], а его единственный
//! потребитель в действующем коде — save-оркестрация (`SaveDataLogPublisher`).
//! `WorldSaveThreadJob` перенесён из того же game.rs, потому что его
//! `lifecycle` собран из типов этого модуля.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::sync::Arc;

use chrono::{Datelike, Local, Timelike};
use parking_lot::Mutex;
use rustix::time::{ClockId, clock_gettime};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::activities::rsgodsbattle::{
    GodsBattleFactionXydSnapshot, GodsBattleNpcFactionSnapshot, GodsBattleSaveOperation,
    RsGodsBattleOwner,
};
use crate::activities::rsjjcsys::RsJjcSysOwner;
use crate::app::worldserver::{
    AddLogTextBlock, AddLogTextDisposition, SaveLogTextDisposition, WorldLogLocalTime,
    WorldLogTextOwner,
};
use crate::characters::honordb::HonorRanksDbDataSnapshot;
use crate::characters::honorranks::CHonorRanks;
use crate::characters::player::{CPlayer, PlayerDbProjectionBlock};
use crate::content::dbgoods::DbGoodsOwner;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::variablelist::{CVariableList, VariableListSaveSource, save_var_data};
use crate::organizations::dbcountry::{CountrySaveSnapshot, DbCountryOwner};
use crate::organizations::rsenemyfactions::{
    EnemyFactionNullEntryBlock, EnemyFactionSaveSnapshot, EnemyFactionsSaveOutcome,
    RsEnemyFactionsOwner,
};
use crate::organizations::rsfaction::{
    FactionSaveBlock, FactionSaveOutcome, FactionSaveProjectionBlock, FactionSaveSnapshot,
    RsFactionOwner,
};
use crate::organizations::rsunion::{
    RsUnionOwner, UnionSaveBlock, UnionSaveOutcome, UnionSaveSnapshot,
};
use crate::persistence::largess::{LargessOwner, SaveLoadDetailsOutcome};
use crate::persistence::rsgenvar::{GenVarSaveOutcome, RsGenVarOwner};
use crate::persistence::rsplayer::{
    HonorRanksSaveBlock, HonorRanksSaveOutcome, PlayerCreateBlock, PlayerCreateOutcome,
    PlayerCreationSnapshot, PlayerDeleteOutcome, PlayerDeleteTimeBlock, PlayerSaveBlock,
    PlayerSaveOutcome, PlayerSaveSnapshot, RsPlayerOwner,
};
use crate::persistence::rssetup::{RsSetupOwner, WorldDatabaseSettings, WorldTdsClient};
use crate::persistence::savedata::{
    DeletionPlayerSnapshot, WorldDbDataSaveSession, WorldSaveDataOwner,
};
use crate::persistence::writelogqueue::WorldWriteLogQueue;
use crate::regions::rsregion::{RegionSaveSnapshot, RsRegionOwner};

const BEGIN_TRANSACTION_SQL: &str = "BEGIN TRAN";
const COMMIT_TRANSACTION_SQL: &str = "COMMIT";
const ROLLBACK_TRANSACTION_SQL: &str = "ROLLBACK";

fn legacy_snapshot_count(phase: &'static str, count: usize) -> Result<u32, WorldSnapshotSaveBlock> {
    u32::try_from(count)
        .map_err(|_| WorldSnapshotSaveBlock::ContainerCountOverflow { phase, count })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveDataLogTarget {
    AddLogText,
    ShowSaveInfo,
}

/// Уже Готовый ANSI payload одного фазового log-вызова.
///
/// `Debug` намеренно скрывает байты: подробные player-сообщения содержат
/// account/name и не должны случайно попадать в диагностический вывод отчёта.
#[derive(Clone, Eq, PartialEq)]
pub struct SaveDataLogEvent {
    pub target: SaveDataLogTarget,
    pub payload: Vec<u8>,
}

impl fmt::Debug for SaveDataLogEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SaveDataLogEvent")
            .field("target", &self.target)
            .field("payload_len", &self.payload.len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveDataLogPublishBlock {
    AddLogText {
        target: SaveDataLogTarget,
        rotation: SaveLogTextDisposition,
        local_time: WorldLogLocalTime,
        block: AddLogTextBlock,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveDataLogPublishDisposition {
    Suppressed,
    Written {
        target: SaveDataLogTarget,
        rotation: SaveLogTextDisposition,
    },
    BlockedMissingFact(SaveDataLogPublishBlock),
}

impl SaveDataLogPublishDisposition {
    pub const fn is_blocked(self) -> bool {
        matches!(self, Self::BlockedMissingFact(_))
    }
}

pub trait SaveDataLogSink {
    fn publish(&mut self, event: &SaveDataLogEvent) -> SaveDataLogPublishDisposition;
}

/// Владеющий состоянием мост фаз `DoSaveData` к двум готовым log-owner-ам.
///
/// Clock и file callbacks хранятся одним owner-ом, чтобы последовательные
/// события наблюдали общий `WorldLogTextOwner` и не получали нового порядка.
pub struct SaveDataLogPublisher<'a, GetTick, GetLocalTime, PutLogInfo> {
    show_save_info_enabled: bool,
    save_info_time_ms: u32,
    log: &'a mut WorldLogTextOwner,
    get_tick: GetTick,
    get_local_time: GetLocalTime,
    put_log_info: PutLogInfo,
}

impl<'a, GetTick, GetLocalTime, PutLogInfo>
    SaveDataLogPublisher<'a, GetTick, GetLocalTime, PutLogInfo>
where
    GetTick: FnMut() -> u32,
    GetLocalTime: FnMut() -> WorldLogLocalTime,
    PutLogInfo: FnMut(&[u8]),
{
    pub fn new(
        show_save_info_enabled: bool,
        save_info_time_ms: u32,
        log: &'a mut WorldLogTextOwner,
        get_tick: GetTick,
        get_local_time: GetLocalTime,
        put_log_info: PutLogInfo,
    ) -> Self {
        Self {
            show_save_info_enabled,
            save_info_time_ms,
            log,
            get_tick,
            get_local_time,
            put_log_info,
        }
    }

    pub fn publish(&mut self, event: &SaveDataLogEvent) -> SaveDataLogPublishDisposition {
        match event.target {
            SaveDataLogTarget::AddLogText => {
                let disposition = self.log.add_log_text(
                    &event.payload,
                    self.save_info_time_ms,
                    &mut self.get_tick,
                    &mut self.get_local_time,
                    &mut self.put_log_info,
                );
                map_add_log_text_disposition(event.target, disposition)
            }
            SaveDataLogTarget::ShowSaveInfo => match show_save_info(
                self.show_save_info_enabled,
                &event.payload,
                self.save_info_time_ms,
                self.log,
                &mut self.get_tick,
                &mut self.get_local_time,
                &mut self.put_log_info,
            ) {
                ShowSaveInfoDisposition::Suppressed => SaveDataLogPublishDisposition::Suppressed,
                ShowSaveInfoDisposition::Logged(disposition) => {
                    map_add_log_text_disposition(event.target, disposition)
                }
            },
        }
    }
}

impl<GetTick, GetLocalTime, PutLogInfo> SaveDataLogSink
    for SaveDataLogPublisher<'_, GetTick, GetLocalTime, PutLogInfo>
where
    GetTick: FnMut() -> u32,
    GetLocalTime: FnMut() -> WorldLogLocalTime,
    PutLogInfo: FnMut(&[u8]),
{
    fn publish(&mut self, event: &SaveDataLogEvent) -> SaveDataLogPublishDisposition {
        SaveDataLogPublisher::publish(self, event)
    }
}

fn publish_save_data_log(
    sink: &mut impl SaveDataLogSink,
    event: &SaveDataLogEvent,
) -> Result<(), SaveDataLogPublishBlock> {
    match sink.publish(event) {
        SaveDataLogPublishDisposition::Suppressed
        | SaveDataLogPublishDisposition::Written { .. } => Ok(()),
        SaveDataLogPublishDisposition::BlockedMissingFact(block) => Err(block),
    }
}

fn map_add_log_text_disposition(
    target: SaveDataLogTarget,
    disposition: AddLogTextDisposition,
) -> SaveDataLogPublishDisposition {
    match disposition {
        AddLogTextDisposition::Written { rotation, .. } => {
            SaveDataLogPublishDisposition::Written { target, rotation }
        }
        AddLogTextDisposition::BlockedMissingFact {
            rotation,
            local_time,
            block,
        } => {
            SaveDataLogPublishDisposition::BlockedMissingFact(SaveDataLogPublishBlock::AddLogText {
                target,
                rotation,
                local_time,
                block,
            })
        }
    }
}

fn add_log_event(payload: impl Into<Vec<u8>>) -> SaveDataLogEvent {
    SaveDataLogEvent {
        target: SaveDataLogTarget::AddLogText,
        payload: payload.into(),
    }
}

fn show_save_info_event(payload: impl Into<Vec<u8>>) -> SaveDataLogEvent {
    SaveDataLogEvent {
        target: SaveDataLogTarget::ShowSaveInfo,
        payload: payload.into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShowSaveInfoDisposition {
    Suppressed,
    Logged(AddLogTextDisposition),
}

/// Выполняет gate и два последовательных formatting-владельца.
///
/// `formatted_message` — результат первого variadic call-site форматирования
/// без конечного NUL. При выключенном `show_save_info` значение сознательно не
/// читается, как и в исходном теле.
pub fn show_save_info<GetTick, GetLocalTime, PutLogInfo>(
    show_save_info: bool,
    formatted_message: &[u8],
    save_info_time_ms: u32,
    log: &mut WorldLogTextOwner,
    get_tick: GetTick,
    get_local_time: GetLocalTime,
    put_log_info: PutLogInfo,
) -> ShowSaveInfoDisposition
where
    GetTick: FnMut() -> u32,
    GetLocalTime: FnMut() -> WorldLogLocalTime,
    PutLogInfo: FnMut(&[u8]),
{
    if !show_save_info {
        return ShowSaveInfoDisposition::Suppressed;
    }

    let formatted_message = legacy_c_string(formatted_message);
    ShowSaveInfoDisposition::Logged(log.add_log_text_no_arguments(
        formatted_message,
        save_info_time_ms,
        get_tick,
        get_local_time,
        put_log_info,
    ))
}

fn legacy_c_string(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}

fn player_success_log(prefix: &[u8], player: &CPlayer) -> SaveDataLogEvent {
    let mut payload = Vec::with_capacity(
        prefix.len() + player.get_account().len() + player.get_name().len() + 24,
    );
    payload.extend_from_slice(prefix);
    payload.extend_from_slice(legacy_c_string(player.get_account()));
    payload.push(b'.');
    payload.extend_from_slice(legacy_c_string(player.get_name()));
    payload.push(b'.');
    payload.extend_from_slice(player.get_id().to_string().as_bytes());
    show_save_info_event(payload)
}

#[derive(Clone, Copy, Debug)]
pub struct SetupIdSnapshot {
    pub player_id: u32,
    pub leave_world_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldSnapshotSaveBlock {
    MissingHonorRanks,
    ContainerCountOverflow { phase: &'static str, count: usize },
}

#[derive(Debug)]
pub enum SetupIdSaveFinish {
    Commit {
        error: Option<tiberius::error::Error>,
    },
    Rollback {
        error: Option<tiberius::error::Error>,
    },
}

#[derive(Debug)]
pub struct SetupIdSaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub player_saved: bool,
    pub leave_world_saved: Option<bool>,
    pub finish: SetupIdSaveFinish,
    pub log_events: Vec<SaveDataLogEvent>,
}

#[derive(Debug)]
pub enum GeneralVariableSaveDisposition {
    Commit {
        error: Option<tiberius::error::Error>,
    },
    Rollback {
        error: Option<tiberius::error::Error>,
    },
}

#[derive(Debug)]
pub struct GeneralVariableSaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub disposition: GeneralVariableSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

#[derive(Debug)]
pub enum FailedTransactionFinish {
    Rollback {
        error: Option<tiberius::error::Error>,
    },
    NoTransaction,
}

#[derive(Debug)]
pub enum NewCharacterSaveDisposition {
    Commit {
        error: Option<tiberius::error::Error>,
    },
    Failure(FailedTransactionFinish),
    NullPlayer,
    BlockedMissingFact(PlayerCreateBlock),
}

#[derive(Debug)]
pub struct NewCharacterSaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub disposition: NewCharacterSaveDisposition,
}

#[derive(Debug)]
pub enum NewCharactersSaveDisposition {
    Complete,
    BlockedLog {
        entry_index: usize,
        entry: NewCharacterSaveReport,
        block: SaveDataLogPublishBlock,
    },
    BlockedProjection {
        entry_index: usize,
        block: PlayerDbProjectionBlock,
    },
    BlockedCreate {
        entry_index: usize,
        begin_error: Option<tiberius::error::Error>,
        block: PlayerCreateBlock,
    },
}

#[derive(Debug)]
pub struct NewCharactersSaveReport {
    pub logged_count: u32,
    pub entries: Vec<NewCharacterSaveReport>,
    pub disposition: NewCharactersSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

#[derive(Debug)]
pub enum RestoreCharacterSaveDisposition {
    Commit {
        error: Option<tiberius::error::Error>,
    },
    Failure(FailedTransactionFinish),
}

#[derive(Debug)]
pub struct RestoreCharacterSaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub disposition: RestoreCharacterSaveDisposition,
}

#[derive(Debug)]
pub struct RestoreCharactersSaveReport {
    pub logged_count: u32,
    pub entries: Vec<RestoreCharacterSaveReport>,
    pub disposition: RestoreCharactersSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

#[derive(Debug)]
pub enum RestoreCharactersSaveDisposition {
    Complete,
    BlockedLog {
        entry_index: usize,
        entry: Box<RestoreCharacterSaveReport>,
        block: SaveDataLogPublishBlock,
    },
}

#[derive(Debug)]
pub enum DeleteCharacterSaveDisposition {
    Commit {
        error: Option<tiberius::error::Error>,
    },
    Failure(FailedTransactionFinish),
    BlockedMissingFact(PlayerDeleteTimeBlock),
}

#[derive(Debug)]
pub struct DeleteCharacterSaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub disposition: DeleteCharacterSaveDisposition,
}

#[derive(Debug)]
pub enum DeleteCharactersSaveDisposition {
    Complete,
    BlockedLog {
        entry_index: usize,
        entry: DeleteCharacterSaveReport,
        block: SaveDataLogPublishBlock,
    },
    BlockedMissingFact {
        entry_index: usize,
        begin_error: Option<tiberius::error::Error>,
        block: PlayerDeleteTimeBlock,
    },
}

#[derive(Debug)]
pub struct DeleteCharactersSaveReport {
    pub logged_count: u32,
    pub entries: Vec<DeleteCharacterSaveReport>,
    pub disposition: DeleteCharactersSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

pub struct DeleteUnionListSnapshot<'entries> {
    pub union_ids: &'entries [i32],
    pub logged_count: u32,
}

#[derive(Debug)]
pub struct DeleteUnionEntrySaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub delete_succeeded: bool,
    pub commit_error: Option<tiberius::error::Error>,
}

#[derive(Clone, Copy, Debug)]
pub struct DeleteUnionClear {
    pub logged_count: u32,
}

#[derive(Debug)]
pub struct DeleteUnionSaveReport {
    pub entries: Vec<DeleteUnionEntrySaveReport>,
    pub clear: DeleteUnionClear,
    pub log_events: Vec<SaveDataLogEvent>,
}

pub struct DeleteFactionListSnapshot<'entries> {
    pub faction_ids: &'entries [i32],
    pub logged_count: u32,
}

#[derive(Debug)]
pub struct DeleteFactionEntrySaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub delete_succeeded: bool,
    pub commit_error: Option<tiberius::error::Error>,
}

#[derive(Clone, Copy, Debug)]
pub struct DeleteFactionClear {
    pub logged_count: u32,
}

#[derive(Debug)]
pub struct DeleteFactionSaveReport {
    pub entries: Vec<DeleteFactionEntrySaveReport>,
    pub clear: DeleteFactionClear,
    pub log_events: Vec<SaveDataLogEvent>,
}

pub struct SaveFactionListSnapshot<'entries, 'faction> {
    pub factions: &'entries mut [Option<FactionSaveSnapshot<'faction>>],
    pub logged_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum SaveFactionEntryCleanup {
    RemoveNode,
    RemoveNodeThenDestroySnapshot,
}

#[derive(Debug)]
pub struct SaveFactionEntrySaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub save_returned: bool,
    pub commit_error: Option<tiberius::error::Error>,
    pub cleanup: SaveFactionEntryCleanup,
}

#[derive(Clone, Copy, Debug)]
pub struct SaveFactionFinalClear {
    pub logged_count: u32,
}

#[derive(Debug)]
pub enum SaveFactionSaveDisposition {
    Complete(SaveFactionFinalClear),
    BlockedLog {
        logged_count: u32,
        block: SaveDataLogPublishBlock,
    },
    BlockedMissingFact {
        entry_index: usize,
        begin_error: Option<tiberius::error::Error>,
        block: SaveFactionPhaseBlock,
    },
}

#[derive(Debug)]
pub enum SaveFactionPhaseBlock {
    Projection(FactionSaveProjectionBlock),
    Owner(FactionSaveBlock),
}

#[derive(Debug)]
pub struct SaveFactionSaveReport {
    pub entries: Vec<SaveFactionEntrySaveReport>,
    pub disposition: SaveFactionSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

pub struct SaveUnionListSnapshot<'entries, 'union> {
    pub unions: &'entries mut [Option<UnionSaveSnapshot<'union>>],
    pub logged_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum SaveUnionEntryCleanup {
    RemoveNode,
    RemoveNodeThenDestroySnapshot,
}

#[derive(Debug)]
pub struct SaveUnionEntrySaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub save_returned: bool,
    pub commit_error: Option<tiberius::error::Error>,
    pub cleanup: SaveUnionEntryCleanup,
}

#[derive(Clone, Copy, Debug)]
pub struct SaveUnionFinalClear {
    pub logged_count: u32,
}

#[derive(Debug)]
pub enum SaveUnionSaveDisposition {
    Complete(SaveUnionFinalClear),
    BlockedLog {
        logged_count: u32,
        block: SaveDataLogPublishBlock,
    },
    BlockedMissingFact {
        entry_index: usize,
        begin_error: Option<tiberius::error::Error>,
        block: UnionSaveBlock,
    },
}

#[derive(Debug)]
pub struct SaveUnionSaveReport {
    pub entries: Vec<SaveUnionEntrySaveReport>,
    pub disposition: SaveUnionSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

pub struct SaveRegionListSnapshot<'entries> {
    pub regions: &'entries [Option<RegionSaveSnapshot>],
    pub logged_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum SaveRegionEntryCleanup {
    RetainNode,
    DestroySnapshotAndRetainNode,
}

#[derive(Debug)]
pub struct SaveRegionEntrySaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub save_returned: bool,
    pub commit_error: Option<tiberius::error::Error>,
    pub cleanup: SaveRegionEntryCleanup,
}

#[derive(Clone, Copy, Debug)]
pub struct SaveRegionFinalClear {
    pub logged_count: u32,
}

#[derive(Debug)]
pub struct SaveRegionSaveReport {
    pub entries: Vec<SaveRegionEntrySaveReport>,
    pub clear: SaveRegionFinalClear,
    pub log_events: Vec<SaveDataLogEvent>,
}

#[derive(Debug)]
pub enum HonorRanksSaveDisposition {
    Commit {
        error: Option<tiberius::error::Error>,
    },
    Failure(FailedTransactionFinish),
    BlockedMissingFact(HonorRanksSaveBlock),
}

#[derive(Debug)]
pub struct HonorRanksTransactionSaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub insert_succeeded: bool,
    pub save_returned: Option<bool>,
    pub disposition: HonorRanksSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

#[derive(Debug)]
pub enum GodsBattleSaveDisposition {
    Commit {
        error: Option<tiberius::error::Error>,
    },
    Failure(FailedTransactionFinish),
}

#[derive(Debug)]
pub struct GodsBattleTransactionSaveReport {
    pub operation: GodsBattleSaveOperation,
    pub begin_error: Option<tiberius::error::Error>,
    pub save_returned: bool,
    pub disposition: GodsBattleSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

#[derive(Clone, Copy, Debug)]
pub enum EnemyFactionsFinalCleanup {
    DestroyValuesThenClearNodes,
}

#[derive(Debug)]
pub enum EnemyFactionsTransactionSaveDisposition {
    Complete {
        commit_error: Option<tiberius::error::Error>,
        cleanup: EnemyFactionsFinalCleanup,
    },
    BlockedMissingFact(EnemyFactionNullEntryBlock),
}

#[derive(Debug)]
pub struct EnemyFactionsTransactionSaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub save_returned: Option<bool>,
    pub disposition: EnemyFactionsTransactionSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

pub struct SaveCountryListSnapshot<'entries> {
    pub countries: &'entries [Option<CountrySaveSnapshot>],
    pub logged_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum SaveCountryEntryCleanup {
    RemoveNode,
    RemoveNodeThenDestroySnapshot,
}

#[derive(Debug)]
pub struct SaveCountryEntrySaveReport {
    pub begin_error: Option<tiberius::error::Error>,
    pub save_returned: bool,
    pub commit_error: Option<tiberius::error::Error>,
    pub cleanup: SaveCountryEntryCleanup,
}

#[derive(Clone, Copy, Debug)]
pub struct SaveCountryFinalClear {
    pub logged_count: u32,
}

#[derive(Debug)]
pub struct SaveCountrySaveReport {
    pub entries: Vec<SaveCountryEntrySaveReport>,
    pub clear: SaveCountryFinalClear,
    pub log_events: Vec<SaveDataLogEvent>,
}

/// Неизменяемые поля non-null `CPlayer*`, действующие LoadDetails-фазой.
///
/// `Debug` сознательно не действует, чтобы CD-key не попадал в обычный log.
#[derive(Clone, Copy)]
pub struct LoadDetailsPlayerSnapshot<'player> {
    pub player_id: i32,
    pub cd_key: &'player [u8],
}

pub struct LoadDetailsPlayerMapSnapshot<'entries, 'player> {
    pub players: &'entries BTreeMap<u32, Option<LoadDetailsPlayerSnapshot<'player>>>,
    pub use_old_save_largess_way: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum LoadDetailsSaveWay {
    Old,
    New,
}

#[derive(Debug)]
pub enum LoadDetailsEntrySaveReport {
    NullPlayer { map_key: u32 },
    OldWay { map_key: u32, save_returned: bool },
    NewWay {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        save_returned: bool,
        commit_error: Option<tiberius::error::Error>,
    },
}

#[derive(Debug)]
pub enum LoadDetailsSaveDisposition {
    CompleteRetainPlayerMap,
}

#[derive(Debug)]
pub struct LoadDetailsSaveReport {
    pub way: LoadDetailsSaveWay,
    pub entries: Vec<LoadDetailsEntrySaveReport>,
    pub disposition: LoadDetailsSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

#[derive(Debug)]
pub struct LoadDetailsWorldSaveReport {
    pub logged_count: u32,
    pub phase: LoadDetailsSaveReport,
}

pub struct SaveCharacterPlayerSnapshot<'player, 'snapshot> {
    pub player: &'player CPlayer,
    pub save: PlayerSaveSnapshot<'snapshot, 'snapshot, 'snapshot>,
}

pub struct SaveCharacterPlayerMapSnapshot<'entries, 'player, 'snapshot> {
    pub players:
        &'entries BTreeMap<u32, Option<SaveCharacterPlayerSnapshot<'player, 'snapshot>>>,
    pub logged_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum SaveCharacterSuccessCleanup {
    DestroyPlayerThenEraseEntryUnderLock,
}

#[derive(Debug)]
pub enum SaveCharacterEntrySaveReport {
    NullPlayer { map_key: u32 },
    Saved {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        commit_error: Option<tiberius::error::Error>,
        cleanup: SaveCharacterSuccessCleanup,
    },
    Failed {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        finish: FailedTransactionFinish,
    },
}

#[derive(Debug)]
pub enum SaveCharacterSaveDisposition {
    Complete { logged_count: u32 },
    BlockedLog {
        map_key: u32,
        entry: Box<SaveCharacterEntrySaveReport>,
        block: SaveDataLogPublishBlock,
    },
    BlockedMissingFact {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        block: SaveCharacterPhaseBlock,
    },
}

#[derive(Debug)]
pub enum SaveCharacterPhaseBlock {
    Projection(PlayerDbProjectionBlock),
    Save(PlayerSaveBlock),
}

#[derive(Debug)]
pub struct SaveCharacterSaveReport {
    pub entries: Vec<SaveCharacterEntrySaveReport>,
    pub disposition: SaveCharacterSaveDisposition,
    pub log_events: Vec<SaveDataLogEvent>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoSaveDataPhase {
    SetupIds,
    GeneralVariables,
    NewCharacters,
    RestoreCharacters,
    DeleteCharacters,
    DeleteUnions,
    DeleteFactions,
    SaveFactions,
    SaveUnions,
    SaveRegions,
    HonorRanks,
    GodsBattleFactionXyd,
    GodsBattleNpcFactions,
    EnemyFactions,
    Countries,
    LoadDetails,
    SaveCharacters,
}

#[derive(Debug, Default)]
pub struct DoSaveDataThroughUnionsPhases {
    pub setup_ids: Option<SetupIdSaveReport>,
    pub general_variables: Option<GeneralVariableSaveReport>,
    pub new_characters: Option<NewCharactersSaveReport>,
    pub restore_characters: Option<RestoreCharactersSaveReport>,
    pub delete_characters: Option<DeleteCharactersSaveReport>,
    pub delete_unions: Option<DeleteUnionSaveReport>,
    pub delete_factions: Option<DeleteFactionSaveReport>,
    pub save_factions: Option<SaveFactionSaveReport>,
    pub save_unions: Option<SaveUnionSaveReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveDataEarlyCounters {
    pub created: u32,
    pub cancel_deleted: u32,
    pub sign_deleted: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveDataLogCheckpoint {
    SaveVariablesStart,
    PhaseStart,
    PhaseEvent(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoSaveDataThroughUnionsDisposition {
    BlockedSnapshot {
        phase: DoSaveDataPhase,
        block: WorldSnapshotSaveBlock,
    },
    BlockedPhase(DoSaveDataPhase),
    BlockedLog {
        phase: DoSaveDataPhase,
        checkpoint: SaveDataLogCheckpoint,
        block: SaveDataLogPublishBlock,
    },
    ContinueWithRegion(SaveDataEarlyCounters),
}

#[derive(Debug)]
pub struct DoSaveDataThroughUnionsReport {
    pub phases: DoSaveDataThroughUnionsPhases,
    pub disposition: DoSaveDataThroughUnionsDisposition,
}

#[derive(Debug, Default)]
pub struct DoSaveDataAfterUnionsPhases {
    pub regions: Option<SaveRegionSaveReport>,
    pub honor_ranks: Option<HonorRanksTransactionSaveReport>,
    pub gods_battle_faction_xyd: Option<GodsBattleTransactionSaveReport>,
    pub gods_battle_npc_factions: Option<GodsBattleTransactionSaveReport>,
    pub enemy_factions: Option<EnemyFactionsTransactionSaveReport>,
    pub countries: Option<SaveCountrySaveReport>,
    pub load_details: Option<LoadDetailsWorldSaveReport>,
    pub save_characters: Option<SaveCharacterSaveReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoSaveDataAfterUnionsDisposition {
    BlockedSnapshot {
        phase: DoSaveDataPhase,
        block: WorldSnapshotSaveBlock,
    },
    BlockedPhase(DoSaveDataPhase),
    BlockedLog {
        phase: DoSaveDataPhase,
        checkpoint: SaveDataLogCheckpoint,
        block: SaveDataLogPublishBlock,
    },
    Complete(SaveDataCounters),
}

#[derive(Debug)]
pub struct DoSaveDataAfterUnionsReport {
    pub phases: DoSaveDataAfterUnionsPhases,
    pub disposition: DoSaveDataAfterUnionsDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoSaveDataPhasesDisposition {
    BlockedThroughUnions,
    BlockedAfterUnions,
    Complete(SaveDataCounters),
}

#[derive(Debug)]
pub struct DoSaveDataPhasesReport {
    pub through_unions: DoSaveDataThroughUnionsReport,
    pub after_unions: Option<DoSaveDataAfterUnionsReport>,
    pub disposition: DoSaveDataPhasesDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveDataCounters {
    pub created: u32,
    pub cancel_deleted: u32,
    pub sign_deleted: u32,
    pub saved: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveDataFinalPath {
    Completed(SaveDataCounters),
    ConnectionOpenFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveDataConnectionFinish {
    CloseLogEndThenRelease,
    LogConnectFailureThenRelease,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveDataLocalTime {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

fn capture_save_data_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u64).wrapping_mul(1_000);
    let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}

fn capture_save_data_local_time() -> SaveDataLocalTime {
    let local = Local::now();
    SaveDataLocalTime {
        year: local.year() as u16,
        month: local.month() as u16,
        day_of_week: local.weekday().num_days_from_sunday() as u16,
        day: local.day() as u16,
        hour: local.hour() as u16,
        minute: local.minute() as u16,
        second: local.second() as u16,
        milliseconds: local.nanosecond().div_euclid(1_000_000) as u16,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SaveDataLifecycleState {
    pub this_save_start_tick_ms: u32,
    pub last_save_tick_ms: u32,
    pub last_save_time: SaveDataLocalTime,
    pub is_saving_data: bool,
}

/// Полный frozen-вход одного системного `SaveThreadFunc`.
///
/// Изменяемые данные следующего MainLoop-прохода остаются у process-owner-ов;
/// worker получает только снимки, которые исходный поток читал после launch.
pub struct WorldSaveThreadJob {
    pub save: WorldSaveDataOwner,
    pub variables: CVariableList,
    pub registry: GoodsBasePropertiesRegistry,
    pub honor_ranks: CHonorRanks,
    pub gods_battle_faction_xyd: GodsBattleFactionXydSnapshot,
    pub gods_battle_npc_factions: Vec<GodsBattleNpcFactionSnapshot>,
    pub use_old_save_largess_way: bool,
    pub save_info_time_ms: u32,
    pub lifecycle: Arc<Mutex<SaveDataLifecycleState>>,
    pub server_name: Vec<u8>,
    pub server_id: i32,
    pub world_number_bits: u32,
    pub write_log_queue: WorldWriteLogQueue,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SaveDataMonitoringReport {
    pub message_type: i8,
    pub server_id: i32,
    pub world_number_bits: u32,
    pub text: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SaveDataFinalDisposition {
    Complete(SaveDataMonitoringReport),
}

#[derive(Debug, Eq, PartialEq)]
pub struct SaveDataFinalReport {
    pub connection_finish: SaveDataConnectionFinish,
    pub summary_log: Vec<u8>,
    pub elapsed_ms: u32,
    pub disposition: SaveDataFinalDisposition,
}

pub struct SaveDataFinalStart {
    pub connection_finish: SaveDataConnectionFinish,
    pub summary_log: Vec<u8>,
    pub elapsed_ms: u32,
    end_tick_ms: u32,
}

pub struct SaveDataFinalSnapshot {
    pub path: SaveDataFinalPath,
    pub started_at_tick_ms: u32,
}

impl SaveDataFinalSnapshot {
    pub const fn connection_finish(&self) -> SaveDataConnectionFinish {
        match self.path {
            SaveDataFinalPath::Completed(_) => SaveDataConnectionFinish::CloseLogEndThenRelease,
            SaveDataFinalPath::ConnectionOpenFailed => {
                SaveDataConnectionFinish::LogConnectFailureThenRelease
            }
        }
    }
}

/// Значения, которые tail читает после обновления time-global полей.
///
/// `Debug` не действует, чтобы byte-string имени сервера не размножался в log.
pub struct SaveDataMonitoringSnapshot {
    pub server_name: Vec<u8>,
    pub write_log_count: u32,
    pub server_id: i32,
    pub world_number_bits: u32,
}

#[derive(Debug)]
pub enum SaveDataConnectionError {
    Connect(io::Error),
    Tds(tiberius::error::Error),
}

impl fmt::Display for SaveDataConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => {
                write!(formatter, "не открыто соединение World save DB: {error}")
            }
            Self::Tds(error) => write!(formatter, "ошибка TDS World save DB: {error}"),
        }
    }
}

impl Error for SaveDataConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
        }
    }
}

impl From<tiberius::error::Error> for SaveDataConnectionError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Tds(error)
    }
}

pub enum DoSaveDataStart {
    Opened {
        connection: WorldTdsClient,
        started_at_tick_ms: u32,
    },
    ConnectionOpenFailed {
        error: SaveDataConnectionError,
        final_snapshot: SaveDataFinalSnapshot,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveDataConnectionLogCheckpoint {
    SaveDataEnd,
    ConnectToDatabaseFailed,
}

#[derive(Debug)]
pub struct SaveDataLifecycleEvidence {
    pub phases: Option<DoSaveDataPhasesReport>,
    pub open_error: Option<SaveDataConnectionError>,
    pub close_error: Option<tiberius::error::Error>,
}

pub enum DoSaveDataLifecycleReport {
    BlockedPhases {
        phases: DoSaveDataPhasesReport,
        connection: WorldTdsClient,
        started_at_tick_ms: u32,
    },
    BlockedConnectionLog {
        evidence: SaveDataLifecycleEvidence,
        final_snapshot: SaveDataFinalSnapshot,
        checkpoint: SaveDataConnectionLogCheckpoint,
        block: SaveDataLogPublishBlock,
    },
    BlockedSummaryLog {
        evidence: SaveDataLifecycleEvidence,
        final_start: SaveDataFinalStart,
        block: SaveDataLogPublishBlock,
    },
    Final {
        evidence: SaveDataLifecycleEvidence,
        report: SaveDataFinalReport,
    },
}

/// Снимает start tick и выполняет исходную `CreateCn/OpenCn`-границу.
///
/// При успехе save-флаг ставится до возврата, поэтому caller обязан следующим
/// observable-действием опубликовать `Save Variables Start...`. При ошибке
/// прежнее значение флага сохраняется; `final_snapshot` ведёт в общий хвост с
/// нулевыми счётчиками и тем же local start tick.
pub async fn begin_do_save_data<PublishState>(
    settings: &WorldDatabaseSettings,
    state: &mut SaveDataLifecycleState,
    publish_state: &mut PublishState,
) -> DoSaveDataStart
where
    PublishState: FnMut(SaveDataLifecycleState),
{
    let started_at_tick_ms = capture_save_data_tick_ms();
    state.this_save_start_tick_ms = started_at_tick_ms;
    publish_state(*state);

    match open_save_data_connection(settings).await {
        Ok(connection) => {
            state.is_saving_data = true;
            publish_state(*state);
            DoSaveDataStart::Opened {
                connection,
                started_at_tick_ms,
            }
        }
        Err(error) => DoSaveDataStart::ConnectionOpenFailed {
            error,
            final_snapshot: SaveDataFinalSnapshot {
                path: SaveDataFinalPath::ConnectionOpenFailed,
                started_at_tick_ms,
            },
        },
    }
}

async fn open_save_data_connection(
    settings: &WorldDatabaseSettings,
) -> Result<WorldTdsClient, SaveDataConnectionError> {
    let config = settings.tds_config();
    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(SaveDataConnectionError::Connect)?;
    tcp.set_nodelay(true)
        .map_err(SaveDataConnectionError::Connect)?;
    tiberius::Client::connect(config, tcp.compat_write())
        .await
        .map_err(SaveDataConnectionError::Tds)
}

/// Выполняет setup-ID участок на уже открытом World DB соединении.
///
/// Ошибки транзакционных команд возвращаются в отчёте, но не меняют порядок и
/// ветвление UPDATE: именно так caller игнорировал результаты ADO wrapper-ов.
pub async fn save_setup_ids<O: RsSetupOwner>(
    setup: &mut O,
    connection: &mut WorldTdsClient,
    snapshot: SetupIdSnapshot,
) -> SetupIdSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let player_saved = setup.save_player_id(connection, snapshot.player_id).await;
    let leave_world_saved = if player_saved {
        Some(
            setup
                .save_leave_world_id(connection, snapshot.leave_world_id)
                .await,
        )
    } else {
        None
    };

    let finish = if leave_world_saved == Some(true) {
        SetupIdSaveFinish::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        }
    } else {
        SetupIdSaveFinish::Rollback {
            error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                .await
                .err(),
        }
    };
    let log_events = vec![match &finish {
        SetupIdSaveFinish::Commit { .. } => {
            show_save_info_event(b"++Save PlayerID SUCCESS!".to_vec())
        }
        SetupIdSaveFinish::Rollback { .. } => add_log_event(b"--Save PlayerID FAILED!".to_vec()),
    }];

    SetupIdSaveReport {
        begin_error,
        player_saved,
        leave_world_saved,
        finish,
        log_events,
    }
}

pub async fn save_setup_ids_from_world_snapshot<O: RsSetupOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    setup: &mut O,
    connection: &mut WorldTdsClient,
) -> Result<SetupIdSaveReport, WorldSnapshotSaveBlock> {
    let (player_id, leave_world_id) = world.setup_ids();
    Ok(save_setup_ids(
        setup,
        connection,
        SetupIdSnapshot {
            player_id,
            leave_world_id,
        },
    )
    .await)
}

/// Выполняет VarData-участок на том же уже открытом World DB соединении.
///
/// `Commit` означает исходный `++Save VarDate SUCCESS!`, даже если commit
/// отказал; `Rollback` — `--Save VarDate FAILED!`, даже если rollback отказал.
/// `BlockedMissingFact` не завершает неизвестную транзакцию по догадке, поэтому
/// caller обязан остановиться на отчёте и не переиспользовать connection.
pub async fn save_general_variables<S, O>(
    variables: &S,
    database: &mut O,
    connection: &mut WorldTdsClient,
) -> GeneralVariableSaveReport
where
    S: VariableListSaveSource,
    O: RsGenVarOwner,
{
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let mut log_events = vec![add_log_event(b"Save VarDate Start...".to_vec())];
    let disposition = match save_var_data(variables, database, connection).await {
        GenVarSaveOutcome::Saved => GeneralVariableSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        },
        GenVarSaveOutcome::Failed => GeneralVariableSaveDisposition::Rollback {
            error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                .await
                .err(),
        },
    };
    match &disposition {
        GeneralVariableSaveDisposition::Commit { .. } => {
            log_events.push(show_save_info_event(b"++Save VarDate SUCCESS!".to_vec()));
        }
        GeneralVariableSaveDisposition::Rollback { .. } => {
            log_events.push(add_log_event(b"--Save VarDate FAILED!".to_vec()));
        }
    }

    GeneralVariableSaveReport {
        begin_error,
        disposition,
        log_events,
    }
}

/// Выполняет транзакционную часть одного элемента `liDBCreationPlayer`.
///
/// `Commit` требует от связанный list-owner-а удалить player и текущий node под
/// `g_CriticalSectionSavePlayerList`. `Failure` и `NullPlayer` сохраняют node и
/// разрешают перейти к следующему. После `BlockedMissingFact` connection и
/// очередь нельзя использовать дальше до решения локальной границы.
pub async fn save_new_character_entry<P, J, G>(
    snapshot: Option<&PlayerCreationSnapshot<'_, '_>>,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
) -> NewCharacterSaveReport
where
    P: RsPlayerOwner<CPlayer>,
    J: RsJjcSysOwner,
    G: DbGoodsOwner<CPlayer>,
{
    let Some(snapshot) = snapshot else {
        return NewCharacterSaveReport {
            begin_error: None,
            disposition: NewCharacterSaveDisposition::NullPlayer,
        };
    };

    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let disposition = match player_database
        .create_player(
            Some(snapshot),
            Some(&mut *connection),
            jjc_database,
            goods_database,
        )
        .await
    {
        PlayerCreateOutcome::ReturnedTrue => NewCharacterSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        },
        PlayerCreateOutcome::ReturnedFalse => {
            NewCharacterSaveDisposition::Failure(if begin_succeeded {
                FailedTransactionFinish::Rollback {
                    error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                        .await
                        .err(),
                }
            } else {
                FailedTransactionFinish::NoTransaction
            })
        }
        PlayerCreateOutcome::BlockedMissingFact(block) => {
            NewCharacterSaveDisposition::BlockedMissingFact(block)
        }
    };

    NewCharacterSaveReport {
        begin_error,
        disposition,
    }
}

/// Выполняет транзакционную часть одного ID из `lDBRestorePlayer`.
///
/// `Commit` требует от связанный list-owner-а удалить текущий node под
/// `g_CriticalSectionSavePlayerList`; `Failure` сохраняет его. Обе исходные
/// ветви продолжают обход со следующим ID.
pub async fn save_restore_character_entry<P: RsPlayerOwner<CPlayer>>(
    player_id: u32,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
) -> RestoreCharacterSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let disposition = if player_database
        .restore_player(player_id, Some(&mut *connection))
        .await
    {
        RestoreCharacterSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        }
    } else {
        RestoreCharacterSaveDisposition::Failure(if begin_succeeded {
            FailedTransactionFinish::Rollback {
                error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                    .await
                    .err(),
            }
        } else {
            FailedTransactionFinish::NoTransaction
        })
    };

    RestoreCharacterSaveReport {
        begin_error,
        disposition,
    }
}

/// Выполняет транзакционную часть одного `tagDeletionPlayer`.
///
/// `Commit` требует удалить текущий node под
/// `g_CriticalSectionSavePlayerList`; `Failure` сохраняет его и разрешает
/// перейти к следующему. После `BlockedMissingFact` connection и очередь
/// нельзя использовать дальше до решения локальной time-границы.
pub async fn save_delete_character_entry<P: RsPlayerOwner<CPlayer>>(
    snapshot: DeletionPlayerSnapshot,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
) -> DeleteCharacterSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let disposition = match player_database
        .delete_player(
            snapshot.player_id,
            snapshot.deletion_time,
            Some(&mut *connection),
        )
        .await
    {
        PlayerDeleteOutcome::ReturnedTrue => DeleteCharacterSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        },
        PlayerDeleteOutcome::ReturnedFalse => {
            DeleteCharacterSaveDisposition::Failure(if begin_succeeded {
                FailedTransactionFinish::Rollback {
                    error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                        .await
                        .err(),
                }
            } else {
                FailedTransactionFinish::NoTransaction
            })
        }
        PlayerDeleteOutcome::BlockedMissingFact(block) => {
            DeleteCharacterSaveDisposition::BlockedMissingFact(block)
        }
    };

    DeleteCharacterSaveReport {
        begin_error,
        disposition,
    }
}

pub async fn save_new_characters_from_world_snapshot<P, J, G>(
    world: &mut WorldDbDataSaveSession<'_>,
    registry: &GoodsBasePropertiesRegistry,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<NewCharactersSaveReport, WorldSnapshotSaveBlock>
where
    P: RsPlayerOwner<CPlayer>,
    J: RsJjcSysOwner,
    G: DbGoodsOwner<CPlayer>,
{
    let logged_count = legacy_snapshot_count("New Character", world.creation_players_len())?;
    let mut entries = Vec::with_capacity(world.creation_players_len());
    let mut log_events = vec![add_log_event(b"Save New Charactor Start...".to_vec())];
    let mut cursor = 0usize;
    let mut entry_index = 0usize;

    while cursor < world.creation_players_len() {
        let Some(player) = world.creation_player(cursor) else {
            unreachable!("cursor проверен против frozen creation-list length");
        };
        let projection = match player.db_projection(registry) {
            Ok(projection) => projection,
            Err(block) => {
                return Ok(NewCharactersSaveReport {
                    logged_count,
                    entries,
                    log_events,
                    disposition: NewCharactersSaveDisposition::BlockedProjection {
                        entry_index,
                        block,
                    },
                });
            }
        };
        let snapshot = match projection.creation_snapshot() {
            Ok(snapshot) => snapshot,
            Err(block) => {
                return Ok(NewCharactersSaveReport {
                    logged_count,
                    entries,
                    log_events,
                    disposition: NewCharactersSaveDisposition::BlockedProjection {
                        entry_index,
                        block,
                    },
                });
            }
        };
        let report = save_new_character_entry(
            Some(&snapshot),
            player_database,
            jjc_database,
            goods_database,
            connection,
        )
        .await;
        let succeeded = matches!(
            &report.disposition,
            NewCharacterSaveDisposition::Commit { .. }
        );
        let event = match &report.disposition {
            NewCharacterSaveDisposition::Commit { .. } => Some(player_success_log(
                b"Create New Charactor SUCCESS : ",
                player,
            )),
            NewCharacterSaveDisposition::Failure(_) => {
                Some(add_log_event(b"--Create New Charactor FAILED.".to_vec()))
            }
            NewCharacterSaveDisposition::NullPlayer => Some(add_log_event(
                b"**Create New Charactor NULL Pointer!!!!!!!!!!!!!!!!".to_vec(),
            )),
            NewCharacterSaveDisposition::BlockedMissingFact(_) => None,
        };
        if matches!(
            &report.disposition,
            NewCharacterSaveDisposition::BlockedMissingFact(_)
        ) {
            let NewCharacterSaveReport {
                begin_error,
                disposition,
            } = report;
            let NewCharacterSaveDisposition::BlockedMissingFact(block) = disposition else {
                unreachable!("variant проверен выше");
            };
            return Ok(NewCharactersSaveReport {
                logged_count,
                entries,
                log_events,
                disposition: NewCharactersSaveDisposition::BlockedCreate {
                    entry_index,
                    begin_error,
                    block,
                },
            });
        }
        let event = event.expect("обычная New Character ветка всегда пишет log");
        log_events.push(event);
        if let Err(block) = publish_save_data_log(
            log_sink,
            log_events.last().expect("event только что добавлен"),
        ) {
            return Ok(NewCharactersSaveReport {
                logged_count,
                entries,
                log_events,
                disposition: NewCharactersSaveDisposition::BlockedLog {
                    entry_index,
                    entry: report,
                    block,
                },
            });
        }
        entries.push(report);
        if succeeded {
            world.remove_creation_player(cursor);
        } else {
            cursor += 1;
        }
        entry_index += 1;
    }

    Ok(NewCharactersSaveReport {
        logged_count,
        entries,
        disposition: NewCharactersSaveDisposition::Complete,
        log_events,
    })
}

pub async fn save_factions_from_world_snapshot<F: RsFactionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<SaveFactionSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Save Faction", world.save_factions_len())?;
    let mut entries = Vec::with_capacity(world.save_factions_len());
    let mut log_events = vec![add_log_event(b"Save Faction Data Start...".to_vec())];

    while world.first_saved_faction_mut().is_some() {
        let entry_index = entries.len();
        let canonical_goods_war_count = world.first_saved_faction_goods_war_count();
        let projection = match world.first_saved_faction_mut() {
            Some(faction) => FactionSaveSnapshot::from_faction(faction, canonical_goods_war_count),
            None => unreachable!("faction front исчез между двумя эксклюзивными borrow"),
        };
        let projection = match projection {
            Ok(projection) => projection,
            Err(block) => {
                return Ok(SaveFactionSaveReport {
                    entries,
                    log_events,
                    disposition: SaveFactionSaveDisposition::BlockedMissingFact {
                        entry_index,
                        begin_error: None,
                        block: SaveFactionPhaseBlock::Projection(block),
                    },
                });
            }
        };

        let entry = {
            let mut projection = projection;
            let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
                .await
                .err();
            let save_outcome = faction_database
                .save_faction(Some(&mut projection), Some(&mut *connection))
                .await;
            let save_returned = match save_outcome {
                FactionSaveOutcome::ReturnedTrue => true,
                FactionSaveOutcome::ReturnedFalse => false,
                FactionSaveOutcome::BlockedMissingFact(block) => {
                    return Ok(SaveFactionSaveReport {
                        entries,
                        log_events,
                        disposition: SaveFactionSaveDisposition::BlockedMissingFact {
                            entry_index,
                            begin_error,
                            block: SaveFactionPhaseBlock::Owner(block),
                        },
                    });
                }
            };
            let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err();
            SaveFactionEntrySaveReport {
                begin_error,
                save_returned,
                commit_error,
                cleanup: SaveFactionEntryCleanup::RemoveNodeThenDestroySnapshot,
            }
        };
        entries.push(entry);
        world.remove_first_saved_faction();
    }

    log_events.push(add_log_event(
        format!("++Save {} Faction Data SUCCESS!", logged_count as i32).into_bytes(),
    ));
    if let Err(block) = publish_save_data_log(
        log_sink,
        log_events.last().expect("final event только что добавлен"),
    ) {
        return Ok(SaveFactionSaveReport {
            entries,
            disposition: SaveFactionSaveDisposition::BlockedLog {
                logged_count,
                block,
            },
            log_events,
        });
    }
    world.clear_saved_faction_nodes();
    Ok(SaveFactionSaveReport {
        entries,
        disposition: SaveFactionSaveDisposition::Complete(SaveFactionFinalClear { logged_count }),
        log_events,
    })
}

pub async fn save_restore_characters_from_world_snapshot<P: RsPlayerOwner<CPlayer>>(
    world: &mut WorldDbDataSaveSession<'_>,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<RestoreCharactersSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Restore Character", world.restore_players_len())?;
    let mut entries = Vec::with_capacity(world.restore_players_len());
    let mut log_events = vec![add_log_event(b"Save Restore Charactor Start...".to_vec())];
    let mut cursor = 0usize;

    while cursor < world.restore_players_len() {
        let player_id = world
            .restore_player_id(cursor)
            .expect("cursor проверен против frozen restore-list length");
        let report = save_restore_character_entry(player_id, player_database, connection).await;
        let succeeded = matches!(
            &report.disposition,
            RestoreCharacterSaveDisposition::Commit { .. }
        );
        let event = match &report.disposition {
            RestoreCharacterSaveDisposition::Commit { .. } => show_save_info_event(
                format!("++Restore Charactor SUCCESS : {}", player_id as i32).into_bytes(),
            ),
            RestoreCharacterSaveDisposition::Failure(_) => {
                add_log_event(b"--Restore Charactor FAILED!".to_vec())
            }
        };
        log_events.push(event);
        if let Err(block) = publish_save_data_log(
            log_sink,
            log_events.last().expect("event только что добавлен"),
        ) {
            let entry_index = entries.len();
            return Ok(RestoreCharactersSaveReport {
                logged_count,
                entries,
                disposition: RestoreCharactersSaveDisposition::BlockedLog {
                    entry_index,
                    entry: Box::new(report),
                    block,
                },
                log_events,
            });
        }
        entries.push(report);
        if succeeded {
            world.remove_restore_player(cursor);
        } else {
            cursor += 1;
        }
    }

    Ok(RestoreCharactersSaveReport {
        logged_count,
        entries,
        disposition: RestoreCharactersSaveDisposition::Complete,
        log_events,
    })
}

pub async fn save_delete_characters_from_world_snapshot<P: RsPlayerOwner<CPlayer>>(
    world: &mut WorldDbDataSaveSession<'_>,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<DeleteCharactersSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Delete Character", world.deletion_players_len())?;
    let mut entries = Vec::with_capacity(world.deletion_players_len());
    let mut log_events = vec![add_log_event(b"Save Delete Charactor ...".to_vec())];
    let mut cursor = 0usize;
    let mut entry_index = 0usize;

    while cursor < world.deletion_players_len() {
        let snapshot = world
            .deletion_player(cursor)
            .expect("cursor проверен против frozen deletion-list length");
        let report = save_delete_character_entry(snapshot, player_database, connection).await;
        let succeeded = matches!(
            &report.disposition,
            DeleteCharacterSaveDisposition::Commit { .. }
        );
        let event = match &report.disposition {
            DeleteCharacterSaveDisposition::Commit { .. } => Some(show_save_info_event(
                format!("++Delete Charactor SUCCESS : {}", snapshot.player_id as i32).into_bytes(),
            )),
            DeleteCharacterSaveDisposition::Failure(_) => {
                Some(add_log_event(b"--Delete Charactor FAILED!".to_vec()))
            }
            DeleteCharacterSaveDisposition::BlockedMissingFact(_) => None,
        };
        if matches!(
            &report.disposition,
            DeleteCharacterSaveDisposition::BlockedMissingFact(_)
        ) {
            let DeleteCharacterSaveReport {
                begin_error,
                disposition,
            } = report;
            let DeleteCharacterSaveDisposition::BlockedMissingFact(block) = disposition else {
                unreachable!("variant проверен выше");
            };
            return Ok(DeleteCharactersSaveReport {
                logged_count,
                entries,
                log_events,
                disposition: DeleteCharactersSaveDisposition::BlockedMissingFact {
                    entry_index,
                    begin_error,
                    block,
                },
            });
        }
        let event = event.expect("обычная Delete Character ветка всегда пишет log");
        log_events.push(event);
        if let Err(block) = publish_save_data_log(
            log_sink,
            log_events.last().expect("event только что добавлен"),
        ) {
            return Ok(DeleteCharactersSaveReport {
                logged_count,
                entries,
                disposition: DeleteCharactersSaveDisposition::BlockedLog {
                    entry_index,
                    entry: report,
                    block,
                },
                log_events,
            });
        }
        entries.push(report);
        if succeeded {
            world.remove_deletion_player(cursor);
        } else {
            cursor += 1;
        }
        entry_index += 1;
    }

    Ok(DeleteCharactersSaveReport {
        logged_count,
        entries,
        disposition: DeleteCharactersSaveDisposition::Complete,
        log_events,
    })
}

/// Выполняет Delete Union DB-вызовы в исходном list-order.
///
/// Bool `DelConfederation` и ошибки транзакционных wrapper-ов только попадают в
/// отчёт. После возврата caller обязан очистить весь live `listDeleteUnions` и
/// написать success-log с `report.clear.logged_count`; ни один entry не
/// сохраняется для retry.
pub async fn save_delete_unions<U: RsUnionOwner>(
    snapshot: DeleteUnionListSnapshot<'_>,
    union_database: &mut U,
    connection: &mut WorldTdsClient,
) -> DeleteUnionSaveReport {
    let mut entries = Vec::with_capacity(snapshot.union_ids.len());
    let log_events = vec![add_log_event(b"Save Delete Union ...".to_vec())];
    for &union_id in snapshot.union_ids {
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let delete_succeeded = union_database
            .del_confederation(union_id, Some(&mut *connection))
            .await;
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(DeleteUnionEntrySaveReport {
            begin_error,
            delete_succeeded,
            commit_error,
        });
    }

    DeleteUnionSaveReport {
        entries,
        clear: DeleteUnionClear {
            logged_count: snapshot.logged_count,
        },
        log_events,
    }
}

pub async fn save_delete_unions_from_world_snapshot<U: RsUnionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    union_database: &mut U,
    connection: &mut WorldTdsClient,
) -> Result<DeleteUnionSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Delete Union", world.delete_union_ids().len())?;
    let mut report = save_delete_unions(
        DeleteUnionListSnapshot {
            union_ids: world.delete_union_ids(),
            logged_count,
        },
        union_database,
        connection,
    )
    .await;
    world.clear_delete_unions();
    report.log_events.push(add_log_event(
        format!(
            "++Delte {} Unions SUCCESS!",
            report.clear.logged_count as i32
        )
        .into_bytes(),
    ));
    Ok(report)
}

/// Выполняет Delete Faction DB-вызовы в исходном list-order.
///
/// Bool `DelFaction` и ошибки транзакционных wrapper-ов только попадают в
/// отчёт. После возврата caller обязан очистить весь live
/// `listDeleteFactions` и написать success-log с
/// `report.clear.logged_count`; ни один entry не сохраняется для retry.
pub async fn save_delete_factions<F: RsFactionOwner>(
    snapshot: DeleteFactionListSnapshot<'_>,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
) -> DeleteFactionSaveReport {
    let mut entries = Vec::with_capacity(snapshot.faction_ids.len());
    let log_events = vec![add_log_event(b"Save Delete Faction Start...".to_vec())];
    for &faction_id in snapshot.faction_ids {
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let delete_succeeded = faction_database
            .del_faction(faction_id, Some(&mut *connection))
            .await;
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(DeleteFactionEntrySaveReport {
            begin_error,
            delete_succeeded,
            commit_error,
        });
    }

    DeleteFactionSaveReport {
        entries,
        clear: DeleteFactionClear {
            logged_count: snapshot.logged_count,
        },
        log_events,
    }
}

pub async fn save_delete_factions_from_world_snapshot<F: RsFactionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
) -> Result<DeleteFactionSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Delete Faction", world.delete_faction_ids().len())?;
    let mut report = save_delete_factions(
        DeleteFactionListSnapshot {
            faction_ids: world.delete_faction_ids(),
            logged_count,
        },
        faction_database,
        connection,
    )
    .await;
    world.clear_delete_factions();
    report.log_events.push(add_log_event(
        format!(
            "++ Delte {} Factions SUCCESS!",
            report.clear.logged_count as i32
        )
        .into_bytes(),
    ));
    Ok(report)
}

/// Выполняет Save Faction Data DB-вызовы в исходном live list-order.
///
/// Для каждого обычного bool-результата commit вызывается безусловно, после чего
/// caller обязан применить `cleanup` до следующего node. `BlockedMissingFact`
/// запрещает продолжать работу с connection и оставляет текущую судьбу открытой.
pub async fn save_factions<F: RsFactionOwner>(
    snapshot: SaveFactionListSnapshot<'_, '_>,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
) -> SaveFactionSaveReport {
    let mut entries = Vec::with_capacity(snapshot.factions.len());
    let mut log_events = vec![add_log_event(b"Save Faction Data Start...".to_vec())];

    for (entry_index, faction_snapshot) in snapshot.factions.iter_mut().enumerate() {
        let cleanup = if faction_snapshot.is_some() {
            SaveFactionEntryCleanup::RemoveNodeThenDestroySnapshot
        } else {
            SaveFactionEntryCleanup::RemoveNode
        };
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let save_outcome = faction_database
            .save_faction(faction_snapshot.as_mut(), Some(&mut *connection))
            .await;
        let save_returned = match save_outcome {
            FactionSaveOutcome::ReturnedTrue => true,
            FactionSaveOutcome::ReturnedFalse => false,
            FactionSaveOutcome::BlockedMissingFact(block) => {
                return SaveFactionSaveReport {
                    entries,
                    log_events,
                    disposition: SaveFactionSaveDisposition::BlockedMissingFact {
                        entry_index,
                        begin_error,
                        block: SaveFactionPhaseBlock::Owner(block),
                    },
                };
            }
        };
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(SaveFactionEntrySaveReport {
            begin_error,
            save_returned,
            commit_error,
            cleanup,
        });
    }

    log_events.push(add_log_event(
        format!(
            "++Save {} Faction Data SUCCESS!",
            snapshot.logged_count as i32
        )
        .into_bytes(),
    ));
    SaveFactionSaveReport {
        entries,
        disposition: SaveFactionSaveDisposition::Complete(SaveFactionFinalClear {
            logged_count: snapshot.logged_count,
        }),
        log_events,
    }
}

/// Выполняет Save Union Data DB-вызовы в исходном live list-order.
///
/// Для каждого обычного bool-результата commit вызывается безусловно, после чего
/// caller обязан применить `cleanup` до следующего node. `BlockedMissingFact`
/// запрещает продолжать работу с connection и оставляет текущую судьбу открытой.
pub async fn save_unions<U: RsUnionOwner>(
    snapshot: SaveUnionListSnapshot<'_, '_>,
    union_database: &mut U,
    connection: &mut WorldTdsClient,
) -> SaveUnionSaveReport {
    let mut entries = Vec::with_capacity(snapshot.unions.len());
    let mut log_events = vec![add_log_event(b"Save Union Data Start...".to_vec())];

    for (entry_index, union_snapshot) in snapshot.unions.iter_mut().enumerate() {
        let cleanup = if union_snapshot.is_some() {
            SaveUnionEntryCleanup::RemoveNodeThenDestroySnapshot
        } else {
            SaveUnionEntryCleanup::RemoveNode
        };
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let save_outcome = union_database
            .save_confederation(union_snapshot.as_ref(), Some(&mut *connection))
            .await;
        let save_returned = match save_outcome {
            UnionSaveOutcome::ReturnedTrue => true,
            UnionSaveOutcome::ReturnedFalse => false,
            UnionSaveOutcome::BlockedMissingFact(block) => {
                return SaveUnionSaveReport {
                    entries,
                    log_events,
                    disposition: SaveUnionSaveDisposition::BlockedMissingFact {
                        entry_index,
                        begin_error,
                        block,
                    },
                };
            }
        };
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(SaveUnionEntrySaveReport {
            begin_error,
            save_returned,
            commit_error,
            cleanup,
        });
    }

    log_events.push(add_log_event(
        format!(
            "++Save {} Union Data SUCCESS!",
            snapshot.logged_count as i32
        )
        .into_bytes(),
    ));
    SaveUnionSaveReport {
        entries,
        disposition: SaveUnionSaveDisposition::Complete(SaveUnionFinalClear {
            logged_count: snapshot.logged_count,
        }),
        log_events,
    }
}

pub async fn save_unions_from_world_snapshot<U: RsUnionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    union_database: &mut U,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> Result<SaveUnionSaveReport, WorldSnapshotSaveBlock> {
    let logged_count = legacy_snapshot_count("Save Union", world.save_unions_len())?;
    let mut entries = Vec::with_capacity(world.save_unions_len());
    let mut log_events = vec![add_log_event(b"Save Union Data Start...".to_vec())];

    while world.first_saved_union().is_some() {
        let entry_index = entries.len();
        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let save_outcome = {
            let union = world
                .first_saved_union()
                .expect("union front проверен в условии цикла");
            let projection = UnionSaveSnapshot::from_union(union);
            union_database
                .save_confederation(Some(&projection), Some(&mut *connection))
                .await
        };
        let save_returned = match save_outcome {
            UnionSaveOutcome::ReturnedTrue => true,
            UnionSaveOutcome::ReturnedFalse => false,
            UnionSaveOutcome::BlockedMissingFact(block) => {
                return Ok(SaveUnionSaveReport {
                    entries,
                    log_events,
                    disposition: SaveUnionSaveDisposition::BlockedMissingFact {
                        entry_index,
                        begin_error,
                        block,
                    },
                });
            }
        };
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(SaveUnionEntrySaveReport {
            begin_error,
            save_returned,
            commit_error,
            cleanup: SaveUnionEntryCleanup::RemoveNodeThenDestroySnapshot,
        });
        world.remove_first_saved_union();
    }

    log_events.push(add_log_event(
        format!("++Save {} Union Data SUCCESS!", logged_count as i32).into_bytes(),
    ));
    if let Err(block) = publish_save_data_log(
        log_sink,
        log_events.last().expect("final event только что добавлен"),
    ) {
        return Ok(SaveUnionSaveReport {
            entries,
            disposition: SaveUnionSaveDisposition::BlockedLog {
                logged_count,
                block,
            },
            log_events,
        });
    }
    world.clear_saved_union_nodes();
    Ok(SaveUnionSaveReport {
        entries,
        disposition: SaveUnionSaveDisposition::Complete(SaveUnionFinalClear { logged_count }),
        log_events,
    })
}

/// Выполняет исходный префикс `DoSaveData` от setup-ID до Save Union Data.
///
/// Аргументы остаются раздельными, потому что исходник использовал независимые
/// DB-owner-ы и snapshot-owner-ы. На первой safe-границе функция возвращается,
/// не передавая connection следующей фазе; уже применённые DB/container эффекты
/// и отчёты сохраняются буквально.
#[allow(
    clippy::too_many_arguments,
    reason = "точные границы DoSaveData требуют отдельных component DB-owner-ов"
)]
pub async fn do_save_data_through_unions<S, O, V, P, J, G, U, F>(
    world: &mut WorldDbDataSaveSession<'_>,
    variables: &S,
    registry: &GoodsBasePropertiesRegistry,
    setup_database: &mut O,
    variable_database: &mut V,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    union_database: &mut U,
    faction_database: &mut F,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> DoSaveDataThroughUnionsReport
where
    S: VariableListSaveSource,
    O: RsSetupOwner,
    V: RsGenVarOwner,
    P: RsPlayerOwner<CPlayer>,
    J: RsJjcSysOwner,
    G: DbGoodsOwner<CPlayer>,
    U: RsUnionOwner,
    F: RsFactionOwner,
{
    let mut phases = DoSaveDataThroughUnionsPhases::default();

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Variables Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SetupIds,
                checkpoint: SaveDataLogCheckpoint::SaveVariablesStart,
                block,
            },
        };
    }

    let setup_ids =
        match save_setup_ids_from_world_snapshot(world, setup_database, connection).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataThroughUnionsReport {
                    phases,
                    disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::SetupIds,
                        block,
                    },
                };
            }
        };
    let setup_log_index = setup_ids.log_events.len() - 1;
    let setup_log_block =
        publish_save_data_log(log_sink, &setup_ids.log_events[setup_log_index]).err();
    phases.setup_ids = Some(setup_ids);
    if let Some(block) = setup_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SetupIds,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(setup_log_index),
                block,
            },
        };
    }

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save VarDate Start...".to_vec()))
    {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GeneralVariables,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let variables_report = save_general_variables(variables, variable_database, connection).await;
    let variables_log_block = variables_report
        .log_events
        .get(1)
        .and_then(|event| publish_save_data_log(log_sink, event).err());
    phases.general_variables = Some(variables_report);
    if let Some(block) = variables_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GeneralVariables,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }
    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save New Charactor Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::NewCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let new_characters = match save_new_characters_from_world_snapshot(
        world,
        registry,
        player_database,
        jjc_database,
        goods_database,
        connection,
        log_sink,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataThroughUnionsReport {
                phases,
                disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::NewCharacters,
                    block,
                },
            };
        }
    };
    let new_characters_log_block = match &new_characters.disposition {
        NewCharactersSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let new_characters_log_index = new_characters.log_events.len().saturating_sub(1);
    let new_characters_blocked = !matches!(
        &new_characters.disposition,
        NewCharactersSaveDisposition::Complete
    );
    phases.new_characters = Some(new_characters);
    if let Some(block) = new_characters_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::NewCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(new_characters_log_index),
                block,
            },
        };
    }
    if new_characters_blocked {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::NewCharacters,
            ),
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Restore Charactor Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::RestoreCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let restore_characters = match save_restore_characters_from_world_snapshot(
        world,
        player_database,
        connection,
        log_sink,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataThroughUnionsReport {
                phases,
                disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::RestoreCharacters,
                    block,
                },
            };
        }
    };
    let restore_log_block = match &restore_characters.disposition {
        RestoreCharactersSaveDisposition::BlockedLog { block, .. } => Some(*block),
        RestoreCharactersSaveDisposition::Complete => None,
    };
    let restore_log_index = restore_characters.log_events.len().saturating_sub(1);
    phases.restore_characters = Some(restore_characters);
    if let Some(block) = restore_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::RestoreCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(restore_log_index),
                block,
            },
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Delete Charactor ...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let delete_characters = match save_delete_characters_from_world_snapshot(
        world,
        player_database,
        connection,
        log_sink,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataThroughUnionsReport {
                phases,
                disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::DeleteCharacters,
                    block,
                },
            };
        }
    };
    let delete_characters_log_block = match &delete_characters.disposition {
        DeleteCharactersSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let delete_characters_log_index = delete_characters.log_events.len().saturating_sub(1);
    let delete_characters_blocked = !matches!(
        &delete_characters.disposition,
        DeleteCharactersSaveDisposition::Complete
    );
    phases.delete_characters = Some(delete_characters);
    if let Some(block) = delete_characters_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(delete_characters_log_index),
                block,
            },
        };
    }
    if delete_characters_blocked {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::DeleteCharacters,
            ),
        };
    }

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save Delete Union ...".to_vec()))
    {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteUnions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let delete_unions =
        match save_delete_unions_from_world_snapshot(world, union_database, connection).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataThroughUnionsReport {
                    phases,
                    disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::DeleteUnions,
                        block,
                    },
                };
            }
        };
    let delete_unions_log_index = delete_unions.log_events.len() - 1;
    let delete_unions_log_block =
        publish_save_data_log(log_sink, &delete_unions.log_events[delete_unions_log_index]).err();
    phases.delete_unions = Some(delete_unions);
    if let Some(block) = delete_unions_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteUnions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(delete_unions_log_index),
                block,
            },
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Delete Faction Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let delete_factions =
        match save_delete_factions_from_world_snapshot(world, faction_database, connection).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataThroughUnionsReport {
                    phases,
                    disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::DeleteFactions,
                        block,
                    },
                };
            }
        };
    let delete_factions_log_index = delete_factions.log_events.len() - 1;
    let delete_factions_log_block = publish_save_data_log(
        log_sink,
        &delete_factions.log_events[delete_factions_log_index],
    )
    .err();
    phases.delete_factions = Some(delete_factions);
    if let Some(block) = delete_factions_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::DeleteFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(delete_factions_log_index),
                block,
            },
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Faction Data Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let save_factions = match save_factions_from_world_snapshot(
        world,
        faction_database,
        connection,
        log_sink,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataThroughUnionsReport {
                phases,
                disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::SaveFactions,
                    block,
                },
            };
        }
    };
    let save_factions_log_block = match &save_factions.disposition {
        SaveFactionSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let save_factions_log_index = save_factions.log_events.len().saturating_sub(1);
    let save_factions_blocked = matches!(
        &save_factions.disposition,
        SaveFactionSaveDisposition::BlockedMissingFact { .. }
    );
    phases.save_factions = Some(save_factions);
    if let Some(block) = save_factions_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(save_factions_log_index),
                block,
            },
        };
    }
    if save_factions_blocked {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::SaveFactions,
            ),
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Union Data Start...".to_vec()),
    ) {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveUnions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let save_unions =
        match save_unions_from_world_snapshot(world, union_database, connection, log_sink).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataThroughUnionsReport {
                    phases,
                    disposition: DoSaveDataThroughUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::SaveUnions,
                        block,
                    },
                };
            }
        };
    let save_unions_log_block = match &save_unions.disposition {
        SaveUnionSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let save_unions_log_index = save_unions.log_events.len().saturating_sub(1);
    let save_unions_blocked = matches!(
        &save_unions.disposition,
        SaveUnionSaveDisposition::BlockedMissingFact { .. }
    );
    phases.save_unions = Some(save_unions);
    if let Some(block) = save_unions_log_block {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveUnions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(save_unions_log_index),
                block,
            },
        };
    }
    if save_unions_blocked {
        return DoSaveDataThroughUnionsReport {
            phases,
            disposition: DoSaveDataThroughUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::SaveUnions,
            ),
        };
    }

    let counters = SaveDataEarlyCounters {
        created: phases
            .new_characters
            .as_ref()
            .expect("New Character phase завершена")
            .logged_count,
        cancel_deleted: phases
            .restore_characters
            .as_ref()
            .expect("Restore Character phase завершена")
            .logged_count,
        sign_deleted: phases
            .delete_characters
            .as_ref()
            .expect("Delete Character phase завершена")
            .logged_count,
    };
    DoSaveDataThroughUnionsReport {
        phases,
        disposition: DoSaveDataThroughUnionsDisposition::ContinueWithRegion(counters),
    }
}

pub async fn save_regions_from_world_snapshot<R: RsRegionOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    region_database: &mut R,
    connection: &mut WorldTdsClient,
) -> Result<SaveRegionSaveReport, WorldSnapshotSaveBlock> {
    let region_count = world.regions_len();
    let logged_count = legacy_snapshot_count("Save Region", region_count)?;
    let mut entries = Vec::with_capacity(region_count);
    let mut log_events = vec![add_log_event(b"Save Region Data...".to_vec())];
    for index in 0..region_count {
        let region = world.region(index).expect("frozen region node не исчезает");
        let entry = save_region_entry(region.as_ref(), region_database, connection).await;
        if matches!(
            entry.cleanup,
            SaveRegionEntryCleanup::DestroySnapshotAndRetainNode
        ) {
            world.destroy_saved_region(index);
        }
        entries.push(entry);
    }
    log_events.push(add_log_event(
        format!("++Save {} Region Data SUCCESS.", logged_count as i32).into_bytes(),
    ));
    Ok(SaveRegionSaveReport {
        entries,
        clear: SaveRegionFinalClear { logged_count },
        log_events,
    })
}

async fn save_region_entry<R: RsRegionOwner>(
    region_snapshot: Option<&RegionSaveSnapshot>,
    region_database: &mut R,
    connection: &mut WorldTdsClient,
) -> SaveRegionEntrySaveReport {
    let cleanup = if region_snapshot.is_some() {
        SaveRegionEntryCleanup::DestroySnapshotAndRetainNode
    } else {
        SaveRegionEntryCleanup::RetainNode
    };
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let save_returned = region_database
        .save(region_snapshot, Some(&mut *connection))
        .await;
    let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
        .await
        .err();
    SaveRegionEntrySaveReport {
        begin_error,
        save_returned,
        commit_error,
        cleanup,
    }
}

/// Выполняет Save HonorRanks на том же уже открытом World DB соединении.
///
/// `InsertHonorRanks == false` пропускает второй owner и входит в тот же
/// abnormal-catch, что и `SaveHonorRanks == false`. После
/// `BlockedMissingFact` caller обязан остановить lifecycle и не передавать
/// connection следующей Save GodsBattle phase.
pub async fn save_honor_ranks<P: RsPlayerOwner<CPlayer>>(
    snapshot: &mut HonorRanksDbDataSnapshot,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
) -> HonorRanksTransactionSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let insert_succeeded = player_database
        .insert_honor_ranks(snapshot, Some(&mut *connection))
        .await;

    let (save_returned, disposition) = if !insert_succeeded {
        (
            None,
            HonorRanksSaveDisposition::Failure(if begin_succeeded {
                FailedTransactionFinish::Rollback {
                    error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                        .await
                        .err(),
                }
            } else {
                FailedTransactionFinish::NoTransaction
            }),
        )
    } else {
        match player_database
            .save_honor_ranks(snapshot, Some(&mut *connection))
            .await
        {
            HonorRanksSaveOutcome::ReturnedTrue => (
                Some(true),
                HonorRanksSaveDisposition::Commit {
                    error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                        .await
                        .err(),
                },
            ),
            HonorRanksSaveOutcome::ReturnedFalse => (
                Some(false),
                HonorRanksSaveDisposition::Failure(if begin_succeeded {
                    FailedTransactionFinish::Rollback {
                        error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                            .await
                            .err(),
                    }
                } else {
                    FailedTransactionFinish::NoTransaction
                }),
            ),
            HonorRanksSaveOutcome::BlockedMissingFact(block) => {
                (None, HonorRanksSaveDisposition::BlockedMissingFact(block))
            }
        }
    };

    let mut log_events = vec![add_log_event(b"Save HonorRanks start...".to_vec())];
    match &disposition {
        HonorRanksSaveDisposition::Commit { .. } => {
            log_events.push(add_log_event(b"+Success to save HonorRanks!!!l".to_vec()))
        }
        HonorRanksSaveDisposition::Failure(_) => {
            log_events.push(add_log_event(b"Save HonorRanks start ABNORMAL".to_vec()))
        }
        HonorRanksSaveDisposition::BlockedMissingFact(_) => {}
    }
    HonorRanksTransactionSaveReport {
        begin_error,
        insert_succeeded,
        save_returned,
        disposition,
        log_events,
    }
}

pub async fn save_honor_ranks_from_world_snapshot<P: RsPlayerOwner<CPlayer>>(
    honor_ranks: &mut CHonorRanks,
    player_database: &mut P,
    connection: &mut WorldTdsClient,
) -> Result<HonorRanksTransactionSaveReport, WorldSnapshotSaveBlock> {
    let snapshot = honor_ranks
        .db_data_mut()
        .ok_or(WorldSnapshotSaveBlock::MissingHonorRanks)?;
    Ok(save_honor_ranks(snapshot, player_database, connection).await)
}

/// Выполняет отдельную Save GodsBattle Belief/XYD transaction phase.
///
/// `false` условно откатывается и не запрещает caller-у перейти к следующей
/// NPC-фазе. `true` означает исходный success-log даже при ошибке commit.
pub async fn save_gods_battle_faction_xyd<G: RsGodsBattleOwner>(
    snapshot: GodsBattleFactionXydSnapshot,
    gods_battle_database: &mut G,
    connection: &mut WorldTdsClient,
) -> GodsBattleTransactionSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let save_returned = gods_battle_database
        .save_faction_xyd(snapshot, Some(&mut *connection))
        .await;
    let disposition = if save_returned {
        GodsBattleSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        }
    } else {
        GodsBattleSaveDisposition::Failure(if begin_succeeded {
            FailedTransactionFinish::Rollback {
                error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                    .await
                    .err(),
            }
        } else {
            FailedTransactionFinish::NoTransaction
        })
    };

    let result_log = if matches!(&disposition, GodsBattleSaveDisposition::Commit { .. }) {
        add_log_event(b"\xA1\xF0Success to save GodsBattle-Belief\xA1\xA3".to_vec())
    } else {
        add_log_event(b"\xA1\xF9Save GodsBattle-Belief ABNORMAL!".to_vec())
    };
    GodsBattleTransactionSaveReport {
        operation: GodsBattleSaveOperation::FactionXyd,
        begin_error,
        save_returned,
        disposition,
        log_events: vec![
            add_log_event(b"Save GodsBattle...start...".to_vec()),
            result_log,
        ],
    }
}

/// Выполняет следующую отдельную Save GodsBattle NPC transaction phase.
///
/// Функция принимает ordered snapshot, но не вызывает no-argument overload и
/// не начинает следующую EnemyFactions phase.
pub async fn save_gods_battle_npc_factions<G: RsGodsBattleOwner>(
    snapshot: &[GodsBattleNpcFactionSnapshot],
    gods_battle_database: &mut G,
    connection: &mut WorldTdsClient,
) -> GodsBattleTransactionSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    let save_returned = gods_battle_database
        .save_npc_faction(snapshot, Some(&mut *connection))
        .await;
    let disposition = if save_returned {
        GodsBattleSaveDisposition::Commit {
            error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
        }
    } else {
        GodsBattleSaveDisposition::Failure(if begin_succeeded {
            FailedTransactionFinish::Rollback {
                error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                    .await
                    .err(),
            }
        } else {
            FailedTransactionFinish::NoTransaction
        })
    };

    let result_log = if matches!(&disposition, GodsBattleSaveDisposition::Commit { .. }) {
        add_log_event(b"\xA1\xF0Success to save GodsBattle-NPC ...start...".to_vec())
    } else {
        add_log_event(b"\xA1\xF9Save GodsBattle-NPC ABNORMAL!".to_vec())
    };
    GodsBattleTransactionSaveReport {
        operation: GodsBattleSaveOperation::NpcFaction,
        begin_error,
        save_returned,
        disposition,
        log_events: vec![result_log],
    }
}

/// Выполняет следующую отдельную EnemyFactions transaction phase.
///
/// Оба обычных bool-результата owner-а безусловно получают commit и одну
/// normal cleanup-обязанность. После `BlockedMissingFact` caller обязан
/// остановить lifecycle и не передавать connection следующей Country-фазе.
pub async fn save_enemy_factions<E: RsEnemyFactionsOwner>(
    snapshot: &[Option<EnemyFactionSaveSnapshot>],
    enemy_factions_database: &mut E,
    connection: &mut WorldTdsClient,
) -> EnemyFactionsTransactionSaveReport {
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();

    let (save_returned, blocked) = match enemy_factions_database
        .save_all_enemy_factions(snapshot, Some(&mut *connection))
        .await
    {
        EnemyFactionsSaveOutcome::ReturnedTrue => (Some(true), None),
        EnemyFactionsSaveOutcome::ReturnedFalse => (Some(false), None),
        EnemyFactionsSaveOutcome::BlockedMissingFact(block) => (None, Some(block)),
    };
    let disposition = if let Some(block) = blocked {
        EnemyFactionsTransactionSaveDisposition::BlockedMissingFact(block)
    } else {
        EnemyFactionsTransactionSaveDisposition::Complete {
            commit_error: run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err(),
            cleanup: EnemyFactionsFinalCleanup::DestroyValuesThenClearNodes,
        }
    };

    EnemyFactionsTransactionSaveReport {
        begin_error,
        save_returned,
        disposition,
        log_events: vec![add_log_event(b"Save EnemyFactions...".to_vec())],
    }
}

pub async fn save_enemy_factions_from_world_snapshot<E: RsEnemyFactionsOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    enemy_factions_database: &mut E,
    connection: &mut WorldTdsClient,
) -> EnemyFactionsTransactionSaveReport {
    let mut report =
        save_enemy_factions(world.enemy_factions(), enemy_factions_database, connection).await;
    if matches!(
        &report.disposition,
        EnemyFactionsTransactionSaveDisposition::Complete { .. }
    ) {
        world.clear_saved_enemy_factions();
        report
            .log_events
            .push(add_log_event(b"++Save EnemyFactions SUCCESS.".to_vec()));
    }
    report
}

pub async fn save_countries_from_world_snapshot<C: DbCountryOwner>(
    world: &mut WorldDbDataSaveSession<'_>,
    country_database: &mut C,
    connection: &mut WorldTdsClient,
) -> Result<SaveCountrySaveReport, WorldSnapshotSaveBlock> {
    let country_count = world.countries_len();
    let logged_count = legacy_snapshot_count("Update Country", country_count)?;
    let mut entries = Vec::with_capacity(country_count);
    let mut log_events = vec![add_log_event(b"Update Country Data...".to_vec())];
    while let Some(country) = world.first_country() {
        let entry = save_country_entry(country, country_database, connection).await;
        world.remove_first_saved_country();
        entries.push(entry);
    }
    log_events.push(add_log_event(
        format!("++Update {} Country Data SUCCESS.", logged_count as i32).into_bytes(),
    ));
    Ok(SaveCountrySaveReport {
        entries,
        clear: SaveCountryFinalClear { logged_count },
        log_events,
    })
}

async fn save_country_entry<C: DbCountryOwner>(
    country_snapshot: Option<&CountrySaveSnapshot>,
    country_database: &mut C,
    connection: &mut WorldTdsClient,
) -> SaveCountryEntrySaveReport {
    let cleanup = if country_snapshot.is_some() {
        SaveCountryEntryCleanup::RemoveNodeThenDestroySnapshot
    } else {
        SaveCountryEntryCleanup::RemoveNode
    };
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let save_returned = country_database
        .save(country_snapshot, Some(&mut *connection))
        .await;
    let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
        .await
        .err();
    SaveCountryEntrySaveReport {
        begin_error,
        save_returned,
        commit_error,
        cleanup,
    }
}

/// Выполняет Save Charactor LoadDetails Data в исходном unsigned map-order.
///
/// Null values только создают исходный log-результат. Old-way вызывает
/// self-opening owner без transaction; new-way всегда вызывает begin и commit
/// вокруг caller-connection owner, игнорируя их ошибки и bool owner-а. После
/// полного обхода caller обязан сохранить `mDBPlayer` неизменной для следующей
/// Save Charactor Data phase.
pub async fn save_load_details<L: LargessOwner>(
    snapshot: LoadDetailsPlayerMapSnapshot<'_, '_>,
    largess: &mut L,
    connection: &mut WorldTdsClient,
) -> LoadDetailsSaveReport {
    let mut entries = Vec::with_capacity(snapshot.players.len());
    let way = if snapshot.use_old_save_largess_way {
        LoadDetailsSaveWay::Old
    } else {
        LoadDetailsSaveWay::New
    };
    let mut log_events = vec![
        add_log_event(b"Save Charactor LoadDetails Data...".to_vec()),
        add_log_event(
            match way {
                LoadDetailsSaveWay::Old => b"** use old save way".as_slice(),
                LoadDetailsSaveWay::New => b"** use new save way".as_slice(),
            }
            .to_vec(),
        ),
    ];

    for (&map_key, player) in snapshot.players {
        let Some(player) = player else {
            log_events.push(add_log_event(
                b"** Save LoadDetails NULL Pointer!!!!!!".to_vec(),
            ));
            entries.push(LoadDetailsEntrySaveReport::NullPlayer { map_key });
            continue;
        };

        if snapshot.use_old_save_largess_way {
            match largess
                .save_load_details(player.cd_key, player.player_id)
                .await
            {
                SaveLoadDetailsOutcome::ReturnedTrue => {
                    entries.push(LoadDetailsEntrySaveReport::OldWay {
                        map_key,
                        save_returned: true,
                    });
                }
                SaveLoadDetailsOutcome::ReturnedFalse => {
                    entries.push(LoadDetailsEntrySaveReport::OldWay {
                        map_key,
                        save_returned: false,
                    });
                }
            }
            continue;
        }

        let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
            .await
            .err();
        let save_returned = match largess
            .save_load_details_with_connection(
                player.cd_key,
                player.player_id,
                Some(&mut *connection),
            )
            .await
        {
            SaveLoadDetailsOutcome::ReturnedTrue => true,
            SaveLoadDetailsOutcome::ReturnedFalse => false,
        };
        let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
            .await
            .err();
        entries.push(LoadDetailsEntrySaveReport::NewWay {
            map_key,
            begin_error,
            save_returned,
            commit_error,
        });
    }

    LoadDetailsSaveReport {
        way,
        entries,
        disposition: LoadDetailsSaveDisposition::CompleteRetainPlayerMap,
        log_events,
    }
}

pub async fn save_load_details_from_world_snapshot<L: LargessOwner>(
    world: &WorldDbDataSaveSession<'_>,
    logged_count: u32,
    use_old_save_largess_way: bool,
    largess: &mut L,
    connection: &mut WorldTdsClient,
) -> LoadDetailsWorldSaveReport {
    let players = world
        .players()
        .iter()
        .map(|(&map_key, player)| {
            (
                map_key,
                Some(LoadDetailsPlayerSnapshot {
                    player_id: player.get_id(),
                    cd_key: player.get_account(),
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let phase = save_load_details(
        LoadDetailsPlayerMapSnapshot {
            players: &players,
            use_old_save_largess_way,
        },
        largess,
        connection,
    )
    .await;
    LoadDetailsWorldSaveReport {
        logged_count,
        phase,
    }
}

enum SaveCharacterEntryOutcome {
    Finished(SaveCharacterEntrySaveReport),
    Blocked {
        map_key: u32,
        begin_error: Option<tiberius::error::Error>,
        block: PlayerSaveBlock,
    },
}

async fn save_character_entry<P, J, G>(
    map_key: u32,
    player: &CPlayer,
    save: &PlayerSaveSnapshot<'_, '_, '_>,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
) -> SaveCharacterEntryOutcome
where
    P: RsPlayerOwner<CPlayer>,
    J: RsJjcSysOwner,
    G: DbGoodsOwner<CPlayer>,
{
    let begin_error = run_transaction_command(connection, BEGIN_TRANSACTION_SQL)
        .await
        .err();
    let begin_succeeded = begin_error.is_none();
    match player
        .save_data(
            save,
            Some(&mut *connection),
            player_database,
            jjc_database,
            goods_database,
        )
        .await
    {
        PlayerSaveOutcome::ReturnedTrue => {
            let commit_error = run_transaction_command(connection, COMMIT_TRANSACTION_SQL)
                .await
                .err();
            SaveCharacterEntryOutcome::Finished(SaveCharacterEntrySaveReport::Saved {
                map_key,
                begin_error,
                commit_error,
                cleanup: SaveCharacterSuccessCleanup::DestroyPlayerThenEraseEntryUnderLock,
            })
        }
        PlayerSaveOutcome::ReturnedFalse => {
            let finish = if begin_succeeded {
                FailedTransactionFinish::Rollback {
                    error: run_transaction_command(connection, ROLLBACK_TRANSACTION_SQL)
                        .await
                        .err(),
                }
            } else {
                FailedTransactionFinish::NoTransaction
            };
            SaveCharacterEntryOutcome::Finished(SaveCharacterEntrySaveReport::Failed {
                map_key,
                begin_error,
                finish,
            })
        }
        PlayerSaveOutcome::BlockedMissingFact(block) => SaveCharacterEntryOutcome::Blocked {
            map_key,
            begin_error,
            block,
        },
    }
}

/// Выполняет Save Charactor Data в исходном unsigned map-order.
///
/// `Saved` требует сразу после success-log применить указанный cleanup под
/// player-list lock. `Failed` и `NullPlayer` сохраняют entry. После
/// `BlockedMissingFact` caller обязан остановить lifecycle и не использовать
/// connection либо map до разрешения локальной границы.
pub async fn save_characters<P, J, G>(
    snapshot: SaveCharacterPlayerMapSnapshot<'_, '_, '_>,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
) -> SaveCharacterSaveReport
where
    P: RsPlayerOwner<CPlayer>,
    J: RsJjcSysOwner,
    G: DbGoodsOwner<CPlayer>,
{
    let mut entries = Vec::with_capacity(snapshot.players.len());
    let mut log_events = vec![add_log_event(b"Save Charactor Data...".to_vec())];

    for (&map_key, player) in snapshot.players {
        let Some(player) = player else {
            log_events.push(add_log_event(
                b"** Save Charactor NULL Pointer!!!!!!".to_vec(),
            ));
            entries.push(SaveCharacterEntrySaveReport::NullPlayer { map_key });
            continue;
        };

        match save_character_entry(
            map_key,
            player.player,
            &player.save,
            player_database,
            jjc_database,
            goods_database,
            connection,
        )
        .await
        {
            SaveCharacterEntryOutcome::Finished(report) => {
                match &report {
                    SaveCharacterEntrySaveReport::Saved { .. } => log_events.push(
                        player_success_log(b"++ Save Charactor SUCCESS : ", player.player),
                    ),
                    SaveCharacterEntrySaveReport::Failed { .. } => {
                        log_events.push(add_log_event(b"-- Save Charactor FAILED!".to_vec()))
                    }
                    SaveCharacterEntrySaveReport::NullPlayer { .. } => {
                        unreachable!("non-null snapshot не создаёт null-result")
                    }
                }
                entries.push(report);
            }
            SaveCharacterEntryOutcome::Blocked {
                map_key,
                begin_error,
                block,
            } => {
                return SaveCharacterSaveReport {
                    entries,
                    log_events,
                    disposition: SaveCharacterSaveDisposition::BlockedMissingFact {
                        map_key,
                        begin_error,
                        block: SaveCharacterPhaseBlock::Save(block),
                    },
                };
            }
        }
    }

    SaveCharacterSaveReport {
        entries,
        disposition: SaveCharacterSaveDisposition::Complete {
            logged_count: snapshot.logged_count,
        },
        log_events,
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Save Character сохраняет раздельные DB-owner-ы и log-owner"
)]
pub async fn save_characters_from_world_snapshot<P, J, G>(
    world: &mut WorldDbDataSaveSession<'_>,
    logged_count: u32,
    registry: &GoodsBasePropertiesRegistry,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> SaveCharacterSaveReport
where
    P: RsPlayerOwner<CPlayer>,
    J: RsJjcSysOwner,
    G: DbGoodsOwner<CPlayer>,
{
    let keys = world.players().keys().copied().collect::<Vec<_>>();
    let mut entries = Vec::with_capacity(keys.len());
    let mut log_events = vec![add_log_event(b"Save Charactor Data...".to_vec())];

    for map_key in keys {
        let outcome = {
            let player = world
                .players()
                .get(&map_key)
                .expect("frozen player-map не меняется вне success cleanup");
            let projection = match player.db_projection(registry) {
                Ok(projection) => projection,
                Err(block) => {
                    return SaveCharacterSaveReport {
                        entries,
                        log_events,
                        disposition: SaveCharacterSaveDisposition::BlockedMissingFact {
                            map_key,
                            begin_error: None,
                            block: SaveCharacterPhaseBlock::Projection(block),
                        },
                    };
                }
            };
            let snapshot = match projection.save_snapshot() {
                Ok(snapshot) => snapshot,
                Err(block) => {
                    return SaveCharacterSaveReport {
                        entries,
                        log_events,
                        disposition: SaveCharacterSaveDisposition::BlockedMissingFact {
                            map_key,
                            begin_error: None,
                            block: SaveCharacterPhaseBlock::Projection(block),
                        },
                    };
                }
            };
            save_character_entry(
                map_key,
                player,
                &snapshot,
                player_database,
                jjc_database,
                goods_database,
                connection,
            )
            .await
        };

        match outcome {
            SaveCharacterEntryOutcome::Finished(report) => {
                let succeeded = matches!(&report, SaveCharacterEntrySaveReport::Saved { .. });
                if succeeded {
                    let player = world
                        .players()
                        .get(&map_key)
                        .expect("success cleanup ещё не удалил player");
                    log_events.push(player_success_log(b"++ Save Charactor SUCCESS : ", player));
                } else {
                    log_events.push(add_log_event(b"-- Save Charactor FAILED!".to_vec()));
                }
                if let Err(block) = publish_save_data_log(
                    log_sink,
                    log_events.last().expect("entry event только что добавлен"),
                ) {
                    return SaveCharacterSaveReport {
                        entries,
                        log_events,
                        disposition: SaveCharacterSaveDisposition::BlockedLog {
                            map_key,
                            entry: Box::new(report),
                            block,
                        },
                    };
                }
                entries.push(report);
                if succeeded {
                    world.remove_player(map_key);
                }
            }
            SaveCharacterEntryOutcome::Blocked {
                map_key,
                begin_error,
                block,
            } => {
                return SaveCharacterSaveReport {
                    entries,
                    log_events,
                    disposition: SaveCharacterSaveDisposition::BlockedMissingFact {
                        map_key,
                        begin_error,
                        block: SaveCharacterPhaseBlock::Save(block),
                    },
                };
            }
        }
    }

    SaveCharacterSaveReport {
        entries,
        disposition: SaveCharacterSaveDisposition::Complete { logged_count },
        log_events,
    }
}

/// Выполняет исходный suffix `DoSaveData` после Save Union Data.
///
/// Region, обе GodsBattle-фазы и Country продолжаются после обычного `false`
/// ровно как исходник. Только typed snapshot/owner block прекращает передачу
/// connection следующему владельцу. Ранний player-map count из LoadDetails
/// становится четвёртым итоговым `SAVED` и повторно используется Save Character.
#[allow(
    clippy::too_many_arguments,
    reason = "точный suffix DoSaveData сохраняет независимые component owner-ы"
)]
pub async fn do_save_data_after_unions<P, J, G, R, B, E, C, L>(
    world: &mut WorldDbDataSaveSession<'_>,
    honor_ranks: &mut CHonorRanks,
    early_counters: SaveDataEarlyCounters,
    gods_battle_faction_xyd: GodsBattleFactionXydSnapshot,
    gods_battle_npc_factions: &[GodsBattleNpcFactionSnapshot],
    use_old_save_largess_way: bool,
    registry: &GoodsBasePropertiesRegistry,
    player_database: &mut P,
    jjc_database: &mut J,
    goods_database: &mut G,
    region_database: &mut R,
    gods_battle_database: &mut B,
    enemy_factions_database: &mut E,
    country_database: &mut C,
    largess: &mut L,
    connection: &mut WorldTdsClient,
    log_sink: &mut impl SaveDataLogSink,
) -> DoSaveDataAfterUnionsReport
where
    P: RsPlayerOwner<CPlayer>,
    J: RsJjcSysOwner,
    G: DbGoodsOwner<CPlayer>,
    R: RsRegionOwner,
    B: RsGodsBattleOwner,
    E: RsEnemyFactionsOwner,
    C: DbCountryOwner,
    L: LargessOwner,
{
    let mut phases = DoSaveDataAfterUnionsPhases::default();

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save Region Data...".to_vec()))
    {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveRegions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let regions = match save_regions_from_world_snapshot(world, region_database, connection).await {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataAfterUnionsReport {
                phases,
                disposition: DoSaveDataAfterUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::SaveRegions,
                    block,
                },
            };
        }
    };
    let regions_final_index = regions.log_events.len() - 1;
    let regions_log_block =
        publish_save_data_log(log_sink, &regions.log_events[regions_final_index]).err();
    phases.regions = Some(regions);
    if let Some(block) = regions_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveRegions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(regions_final_index),
                block,
            },
        };
    }
    world.clear_saved_region_nodes();

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save HonorRanks start...".to_vec()),
    ) {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::HonorRanks,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let honor_report = match save_honor_ranks_from_world_snapshot(
        honor_ranks,
        player_database,
        connection,
    )
    .await
    {
        Ok(report) => report,
        Err(block) => {
            return DoSaveDataAfterUnionsReport {
                phases,
                disposition: DoSaveDataAfterUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::HonorRanks,
                    block,
                },
            };
        }
    };
    let honor_blocked = matches!(
        &honor_report.disposition,
        HonorRanksSaveDisposition::BlockedMissingFact(_)
    );
    let honor_log_block = honor_report
        .log_events
        .get(1)
        .and_then(|event| publish_save_data_log(log_sink, event).err());
    phases.honor_ranks = Some(honor_report);
    if let Some(block) = honor_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::HonorRanks,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }
    if honor_blocked {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::HonorRanks,
            ),
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save GodsBattle...start...".to_vec()),
    ) {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GodsBattleFactionXyd,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let gods_battle_faction_xyd_report =
        save_gods_battle_faction_xyd(gods_battle_faction_xyd, gods_battle_database, connection)
            .await;
    let gods_battle_faction_xyd_log_block =
        publish_save_data_log(log_sink, &gods_battle_faction_xyd_report.log_events[1]).err();
    phases.gods_battle_faction_xyd = Some(gods_battle_faction_xyd_report);
    if let Some(block) = gods_battle_faction_xyd_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GodsBattleFactionXyd,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }

    let gods_battle_npc_report =
        save_gods_battle_npc_factions(gods_battle_npc_factions, gods_battle_database, connection)
            .await;
    let gods_battle_npc_log_block =
        publish_save_data_log(log_sink, &gods_battle_npc_report.log_events[0]).err();
    phases.gods_battle_npc_factions = Some(gods_battle_npc_report);
    if let Some(block) = gods_battle_npc_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::GodsBattleNpcFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(0),
                block,
            },
        };
    }

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save EnemyFactions...".to_vec()))
    {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::EnemyFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let enemy_report =
        save_enemy_factions_from_world_snapshot(world, enemy_factions_database, connection).await;
    let enemy_blocked = matches!(
        &enemy_report.disposition,
        EnemyFactionsTransactionSaveDisposition::BlockedMissingFact(_)
    );
    let enemy_log_block = enemy_report
        .log_events
        .get(1)
        .and_then(|event| publish_save_data_log(log_sink, event).err());
    phases.enemy_factions = Some(enemy_report);
    if let Some(block) = enemy_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::EnemyFactions,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }
    if enemy_blocked {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::EnemyFactions,
            ),
        };
    }

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Update Country Data...".to_vec()))
    {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::Countries,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let countries =
        match save_countries_from_world_snapshot(world, country_database, connection).await {
            Ok(report) => report,
            Err(block) => {
                return DoSaveDataAfterUnionsReport {
                    phases,
                    disposition: DoSaveDataAfterUnionsDisposition::BlockedSnapshot {
                        phase: DoSaveDataPhase::Countries,
                        block,
                    },
                };
            }
        };
    let countries_final_index = countries.log_events.len() - 1;
    let countries_log_block =
        publish_save_data_log(log_sink, &countries.log_events[countries_final_index]).err();
    phases.countries = Some(countries);
    if let Some(block) = countries_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::Countries,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(countries_final_index),
                block,
            },
        };
    }

    if let Err(block) = publish_save_data_log(
        log_sink,
        &add_log_event(b"Save Charactor LoadDetails Data...".to_vec()),
    ) {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::LoadDetails,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }
    let saved = match legacy_snapshot_count("Save Character", world.players().len()) {
        Ok(count) => count,
        Err(block) => {
            return DoSaveDataAfterUnionsReport {
                phases,
                disposition: DoSaveDataAfterUnionsDisposition::BlockedSnapshot {
                    phase: DoSaveDataPhase::LoadDetails,
                    block,
                },
            };
        }
    };
    let load_details_way_event = add_log_event(
        if use_old_save_largess_way {
            b"** use old save way".as_slice()
        } else {
            b"** use new save way".as_slice()
        }
        .to_vec(),
    );
    if let Err(block) = publish_save_data_log(log_sink, &load_details_way_event) {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::LoadDetails,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(1),
                block,
            },
        };
    }

    let load_details = save_load_details_from_world_snapshot(
        world,
        saved,
        use_old_save_largess_way,
        largess,
        connection,
    )
    .await;
    phases.load_details = Some(load_details);

    if let Err(block) =
        publish_save_data_log(log_sink, &add_log_event(b"Save Charactor Data...".to_vec()))
    {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseStart,
                block,
            },
        };
    }

    let save_characters = save_characters_from_world_snapshot(
        world,
        saved,
        registry,
        player_database,
        jjc_database,
        goods_database,
        connection,
        log_sink,
    )
    .await;
    let save_characters_log_block = match &save_characters.disposition {
        SaveCharacterSaveDisposition::BlockedLog { block, .. } => Some(*block),
        _ => None,
    };
    let save_characters_log_index = save_characters.log_events.len().saturating_sub(1);
    let save_characters_blocked = matches!(
        &save_characters.disposition,
        SaveCharacterSaveDisposition::BlockedMissingFact { .. }
    );
    phases.save_characters = Some(save_characters);
    if let Some(block) = save_characters_log_block {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedLog {
                phase: DoSaveDataPhase::SaveCharacters,
                checkpoint: SaveDataLogCheckpoint::PhaseEvent(save_characters_log_index),
                block,
            },
        };
    }
    if save_characters_blocked {
        return DoSaveDataAfterUnionsReport {
            phases,
            disposition: DoSaveDataAfterUnionsDisposition::BlockedPhase(
                DoSaveDataPhase::SaveCharacters,
            ),
        };
    }

    DoSaveDataAfterUnionsReport {
        phases,
        disposition: DoSaveDataAfterUnionsDisposition::Complete(SaveDataCounters {
            created: early_counters.created,
            cancel_deleted: early_counters.cancel_deleted,
            sign_deleted: early_counters.sign_deleted,
            saved,
        }),
    }
}

/// Выполняет все исходные DB/container фазы на уже открытом World connection.
///
/// Функция не создаёт и не закрывает connection и не запускает save-thread.
/// Вся phase-цепь публикует logs через переданный owner в порядке. Первый
/// block либо четыре counters передаются существующему finalizer-у.
#[allow(
    clippy::too_many_arguments,
    reason = "полный DoSaveData сохраняет все независимые component owner-ы"
)]
pub async fn do_save_data_phases<S, O, V, P, J, G, U, F, R, B, E, C, L, Log>(
    world: &mut WorldDbDataSaveSession<'_>,
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
    connection: &mut WorldTdsClient,
    log_sink: &mut Log,
) -> DoSaveDataPhasesReport
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
{
    let through_unions = do_save_data_through_unions(
        world,
        variables,
        registry,
        setup_database,
        variable_database,
        player_database,
        jjc_database,
        goods_database,
        union_database,
        faction_database,
        connection,
        log_sink,
    )
    .await;
    let early_counters = match through_unions.disposition {
        DoSaveDataThroughUnionsDisposition::ContinueWithRegion(counters) => counters,
        DoSaveDataThroughUnionsDisposition::BlockedSnapshot { .. }
        | DoSaveDataThroughUnionsDisposition::BlockedPhase(_)
        | DoSaveDataThroughUnionsDisposition::BlockedLog { .. } => {
            return DoSaveDataPhasesReport {
                through_unions,
                after_unions: None,
                disposition: DoSaveDataPhasesDisposition::BlockedThroughUnions,
            };
        }
    };

    let after_unions = do_save_data_after_unions(
        world,
        honor_ranks,
        early_counters,
        gods_battle_faction_xyd,
        gods_battle_npc_factions,
        use_old_save_largess_way,
        registry,
        player_database,
        jjc_database,
        goods_database,
        region_database,
        gods_battle_database,
        enemy_factions_database,
        country_database,
        largess,
        connection,
        log_sink,
    )
    .await;
    let disposition = match after_unions.disposition {
        DoSaveDataAfterUnionsDisposition::Complete(counters) => {
            DoSaveDataPhasesDisposition::Complete(counters)
        }
        DoSaveDataAfterUnionsDisposition::BlockedSnapshot { .. }
        | DoSaveDataAfterUnionsDisposition::BlockedPhase(_)
        | DoSaveDataAfterUnionsDisposition::BlockedLog { .. } => {
            DoSaveDataPhasesDisposition::BlockedAfterUnions
        }
    };

    DoSaveDataPhasesReport {
        through_unions,
        after_unions: Some(after_unions),
        disposition,
    }
}

/// Начинает общий отчёт после уже выполненного component-specific cleanup.
///
/// Caller заранее получает cleanup через `SaveDataFinalSnapshot::connection_finish`,
/// выполняет его и только затем снимает `end_tick_ms`. Возвращённый summary-log
/// должен быть опубликован до `capture_save_data_local_time`.
fn begin_finish_save_data(
    snapshot: SaveDataFinalSnapshot,
    end_tick_ms: u32,
) -> SaveDataFinalStart {
    let connection_finish = snapshot.connection_finish();
    let counters = match snapshot.path {
        SaveDataFinalPath::Completed(counters) => counters,
        SaveDataFinalPath::ConnectionOpenFailed => SaveDataCounters {
            created: 0,
            cancel_deleted: 0,
            sign_deleted: 0,
            saved: 0,
        },
    };
    let elapsed_ms = end_tick_ms.wrapping_sub(snapshot.started_at_tick_ms);
    let summary_log = legacy_save_summary(counters, elapsed_ms);

    SaveDataFinalStart {
        connection_finish,
        summary_log,
        elapsed_ms,
        end_tick_ms,
    }
}

/// Завершает хвост после summary-log и следующего `GetLocalTime`.
///
/// `get_monitoring` читает queue-size, server name/ID и `dwNumber` после трёх
/// time-global мутаций, а `send_monitoring` вызывается после успешного
/// `_sprintf`. Его результат исходно отсутствовал, поэтому сразу после
/// возврата closure save-флаг сбрасывается.
fn finish_save_data<GetMonitoring, SendMonitoring>(
    start: SaveDataFinalStart,
    state: &mut SaveDataLifecycleState,
    local_time: SaveDataLocalTime,
    get_monitoring: GetMonitoring,
    send_monitoring: SendMonitoring,
) -> SaveDataFinalReport
where
    GetMonitoring: FnOnce() -> SaveDataMonitoringSnapshot,
    SendMonitoring: FnOnce(&SaveDataMonitoringReport),
{
    state.last_save_time = local_time;
    state.last_save_tick_ms = start.end_tick_ms;
    state.this_save_start_tick_ms = 0;

    let monitoring_snapshot = get_monitoring();

    let text = legacy_save_monitoring_text(
        &monitoring_snapshot.server_name,
        local_time,
        start.elapsed_ms,
        monitoring_snapshot.write_log_count,
    );
    let monitoring = SaveDataMonitoringReport {
        message_type: -2,
        server_id: monitoring_snapshot.server_id,
        world_number_bits: monitoring_snapshot.world_number_bits,
        text,
    };
 // Исходный SendErrLog возвращал void: попытка send всегда ведёт к
 // сбросу флага, независимо от результата внутреннего CMessage::Send.
    send_monitoring(&monitoring);
    state.is_saving_data = false;
    let disposition = SaveDataFinalDisposition::Complete(monitoring);

    SaveDataFinalReport {
        connection_finish: start.connection_finish,
        summary_log: start.summary_log,
        elapsed_ms: start.elapsed_ms,
        disposition,
    }
}

/// Выполняет один полный typed lifecycle `DoSaveData` без runtime thread/entry.
///
/// Phase block возвращает ещё открытый connection и не выбирает cleanup. На
/// normal/open-failure путях функция сама сохраняет исходный порядок
/// connection-log, конечного tick, summary-log, local time и monitoring send.
#[allow(
    clippy::too_many_arguments,
    reason = "DoSaveData сохраняет независимые snapshot-, DB-, log- и monitoring-owner-ы"
)]
pub async fn do_save_data_lifecycle<
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
    GetMonitoring,
    SendMonitoring,
    PublishState,
>(
    settings: &WorldDatabaseSettings,
    state: &mut SaveDataLifecycleState,
    world: &mut WorldDbDataSaveSession<'_>,
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
    mut publish_state: PublishState,
    get_monitoring: GetMonitoring,
    send_monitoring: SendMonitoring,
) -> DoSaveDataLifecycleReport
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
    GetMonitoring: FnOnce() -> SaveDataMonitoringSnapshot,
    SendMonitoring: FnOnce(&SaveDataMonitoringReport),
    PublishState: FnMut(SaveDataLifecycleState),
{
    let (evidence, final_snapshot) =
        match begin_do_save_data(settings, state, &mut publish_state).await {
        DoSaveDataStart::Opened {
            mut connection,
            started_at_tick_ms,
        } => {
            let phases = do_save_data_phases(
                world,
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
                &mut connection,
                log_sink,
            )
            .await;
            let counters = match phases.disposition {
                DoSaveDataPhasesDisposition::Complete(counters) => counters,
                DoSaveDataPhasesDisposition::BlockedThroughUnions
                | DoSaveDataPhasesDisposition::BlockedAfterUnions => {
                    return DoSaveDataLifecycleReport::BlockedPhases {
                        phases,
                        connection,
                        started_at_tick_ms,
                    };
                }
            };
            let final_snapshot = SaveDataFinalSnapshot {
                path: SaveDataFinalPath::Completed(counters),
                started_at_tick_ms,
            };

 // `Client::close(self)` одновременно завершает TDS transport и
 // потребляет Rust-owner. Это совместимая замена первого CloseCn;
 // исходный повторный CloseCn внутри ReleaseCn был idempotent.
            let close_error = connection.close().await.err();
            let evidence = SaveDataLifecycleEvidence {
                phases: Some(phases),
                open_error: None,
                close_error,
            };
            if let Err(block) =
                publish_save_data_log(log_sink, &add_log_event(b"Save Data end ...".to_vec()))
            {
                return DoSaveDataLifecycleReport::BlockedConnectionLog {
                    evidence,
                    final_snapshot,
                    checkpoint: SaveDataConnectionLogCheckpoint::SaveDataEnd,
                    block,
                };
            }
 // Здесь находится исходная ReleaseCn-граница; Tiberius owner уже
 // потреблён первым close, поэтому второго observable вызова нет.
            (evidence, final_snapshot)
        }
        DoSaveDataStart::ConnectionOpenFailed {
            error,
            final_snapshot,
        } => {
            let evidence = SaveDataLifecycleEvidence {
                phases: None,
                open_error: Some(error),
                close_error: None,
            };
            if let Err(block) =
                publish_save_data_log(log_sink, &add_log_event(b"Connect To DB FAILED!".to_vec()))
            {
                return DoSaveDataLifecycleReport::BlockedConnectionLog {
                    evidence,
                    final_snapshot,
                    checkpoint: SaveDataConnectionLogCheckpoint::ConnectToDatabaseFailed,
                    block,
                };
            }
 // Неуспешный Tiberius connect уже освободил transport-owner; эта
 // точка сохраняет исходную логическую ReleaseCn-границу после log.
            (evidence, final_snapshot)
        }
    };

    let end_tick_ms = capture_save_data_tick_ms();
    let final_start = begin_finish_save_data(final_snapshot, end_tick_ms);
    let summary_event = add_log_event(final_start.summary_log.clone());
    if let Err(block) = publish_save_data_log(log_sink, &summary_event) {
        return DoSaveDataLifecycleReport::BlockedSummaryLog {
            evidence,
            final_start,
            block,
        };
    }

    let local_time = capture_save_data_local_time();
    let report = finish_save_data(
        final_start,
        state,
        local_time,
        get_monitoring,
        send_monitoring,
    );
    publish_state(*state);
    DoSaveDataLifecycleReport::Final { evidence, report }
}

fn legacy_save_summary(counters: SaveDataCounters, elapsed_ms: u32) -> Vec<u8> {
    format!(
        "\r\n{} Charactor CREATED,\r\n{} Charactor CANCEL DELETE,\r\n{} Charactor SIGN DELETE,\r\n{} Charactor SAVED\r\nUSED TIME:{}ms",
        counters.created as i32,
        counters.cancel_deleted as i32,
        counters.sign_deleted as i32,
        counters.saved as i32,
        elapsed_ms as i32,
    )
    .into_bytes()
}

fn legacy_save_monitoring_text(
    server_name: &[u8],
    local_time: SaveDataLocalTime,
    elapsed_ms: u32,
    write_log_count: u32,
) -> Vec<u8> {
    let server_name = server_name
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default();
    let suffix = format!(
        "|SAVE TIME~{:4}-{:2}-{:2} {:2}:{:2}:{:2}|USED TIME~{}|LOG NUM~{}",
        local_time.year,
        local_time.month,
        local_time.day,
        local_time.hour,
        local_time.minute,
        local_time.second,
        elapsed_ms as i32,
        write_log_count as i32,
    );
    let mut text = Vec::with_capacity(4 + server_name.len() + suffix.len());
    text.extend_from_slice(b"|ws~");
    text.extend_from_slice(server_name);
    text.extend_from_slice(suffix.as_bytes());

    text
}

async fn run_transaction_command(
    connection: &mut WorldTdsClient,
    sql: &'static str,
) -> Result<(), tiberius::error::Error> {
    connection.simple_query(sql).await?.into_results().await?;
    Ok(())
}
