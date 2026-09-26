//! Save-state и отчёты save-триггера хода WorldServer, перенесённые из
//! `src/worldserver/worldserver/game.rs` волной C5-A (hub-data уровень):
//! запросы broadcast player-data, trigger-флаги ручного сохранения, pre-gate,
//! launch/notify отчёты и терминальная форма trigger-итога. Источник
//! контракта — та же точная пара, что у [`crate::app::world_runtime`]
//! (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`,
//! RSDS совпадает).
//!
//! Guard-обёртка [`WorldRunSaveGuard`] остаётся generic на типе игры: старый
//! пакет закрепляет её `CGame` собственным alias-ом до волны самого `CGame`.

use crate::app::world_message::SendMessageError;
use crate::app::worldserver::{
    AddLogTextDisposition, WorldGenerateDbDataBlock, WorldGenerateDbDataReport,
    WorldSaveThreadHandleState, WorldSaveThreadLaunchRequest,
};
use crate::organizations::organizingctrl::{OrganizingSaveDataBlock, OrganizingSaveDataReport};
use crate::persistence::saveworker::WorldRunSaveGuard;

#[derive(Debug)]
pub struct WorldRunSaveLaunchReport {
    pub snapshot: WorldGenerateDbDataReport,
    pub launch: WorldSaveThreadLaunchRequest,
    pub resulting_handle: WorldSaveThreadHandleState,
}

#[derive(Debug)]
pub struct WorldSaveAllOrganizationsLaunchReport {
    pub organizing: OrganizingSaveDataReport,
    pub launch: WorldSaveThreadLaunchRequest,
    pub resulting_handle: WorldSaveThreadHandleState,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldCollectPlayerDataRequestState {
    pub send_now: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCollectPlayerDataBroadcast {
    pub message_type: i32,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRunSaveTriggerState {
    pub send_save_message_now: bool,
    pub save_all_organizations: bool,
    pub save_now_data: bool,
    pub last_save_point_time_ms: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldManualSaveRequestReport {
    pub log: AddLogTextDisposition,
    pub save_point_time_ms: u32,
}

pub enum WorldRunSavePreGateReport<'game, Game> {
    IntervalNotElapsed {
        manual_request: Option<WorldManualSaveRequestReport>,
        profile_started_at_ms: u32,
        elapsed_ms: u32,
        save_point_time_ms: u32,
    },
    SaveLockBusy {
        manual_request: Option<WorldManualSaveRequestReport>,
        profile_started_at_ms: u32,
        elapsed_ms: u32,
        save_point_time_ms: u32,
        adjusted_last_save_point_time_ms: u32,
    },
    AfterLock {
        manual_request: Option<WorldManualSaveRequestReport>,
        profile_started_at_ms: u32,
        elapsed_ms: u32,
        save_point_time_ms: u32,
        trigger: WorldRunSaveTriggerReport<'game, Game>,
    },
}

#[derive(Debug)]
pub struct WorldRunImmediateSaveReport {
    pub log: AddLogTextDisposition,
    pub save: WorldRunSaveLaunchReport,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldSaveNotifyDelivery {
    pub game_server_index: u32,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldSaveNotifyReport {
    pub log: AddLogTextDisposition,
    pub previous_db_responses: i32,
    pub message_type: i32,
    pub deliveries: Vec<WorldSaveNotifyDelivery>,
}

#[derive(Debug)]
pub enum WorldRunSaveTriggerDisposition {
    SaveAllOrganizations(WorldSaveAllOrganizationsLaunchReport),
    PlayerData {
        immediate: Option<WorldRunImmediateSaveReport>,
        notify: Option<WorldSaveNotifyReport>,
    },
}

pub enum WorldRunSaveTriggerReport<'game, Game> {
    BlockedSaveAllOrganizations {
        guard: WorldRunSaveGuard<'game, Game>,
        block: OrganizingSaveDataBlock,
    },
    BlockedImmediateSave {
        guard: WorldRunSaveGuard<'game, Game>,
        log: AddLogTextDisposition,
        block: WorldGenerateDbDataBlock,
    },
    Complete(WorldRunSaveTriggerDisposition),
}
