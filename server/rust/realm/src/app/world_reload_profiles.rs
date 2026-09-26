//! Refresh/reload-контракт и state-отчёты стадий MainLoop WorldServer
//! (hub-data уровень): snapshot-gate RefreshInfo, atomic reload-flags с
//! таблицей профилей `ReloadConf`, resource snapshot стадии, maintenance
//! player-ranks/honor, report-снимки профилирования и profile-init helpers.
//! Источник контракта — та же точная пара, что у [`crate::app::world_runtime`]
//! (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`,
//! RSDS совпадает).
//!
//! `current_country_save_limits` (`tagSaveCountry`) и snapshot-gate
//! `legacy_refresh_count` опираются на типы
//! `WorldMainLoop*`/`WorldRefreshSnapshotBlock`/`WorldProcessMessageError`
//! этого файла. Reload-диспетчер (`reload_profiles`, `reload_conf_log`)
//! остаётся у process-owner-а: он типизирован живым `CGame`.

use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use nebokrai_shared::resources::{CLogSystem, CPlayerList, GlobeSetupSnapshot, RegionRouter};

use crate::app::world_hub_data::{
    WorldMainLoopClockState, WorldMainLoopInitializationState, WorldMainLoopProfileState,
    WorldMainLoopTailClockInitialization, WorldMainLoopTailClockState, WorldProcessMessageOutcome,
};
use crate::app::world_message::SendMessageError;
use crate::app::world_runtime::{PlayerRanksStatRunBlock, PlayerRanksStatRunReport};
use crate::app::world_save_reports::WorldRunSaveTriggerState;
use crate::app::worldserver::{
    AddLogTextDisposition, WorldRefreshInfoReport, WorldReloadBlock, WorldReloadContext,
};
use crate::app::servermessage::WorldServerMessageError;
use crate::auction::auctionlog::AuctionBangUpdateOutcome;
use crate::characters::honorranks::{HonorRanksNewDayBlock, HonorRanksNewDayReport};
use crate::characters::player::PlayerPropertyCoefficients;
use crate::characters::playerranks::{PlayerRanksGameServerUpdate, PlayerRanksSerializationBlock};
use crate::content::cgoodsfactory::GoodsOriginalNameIndex;
use crate::content::countryparam::CCountryParam;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::ScriptLoadContext;
use crate::organizations::country::CountryKingSaveLimits;
use crate::regions::worldregion::WorldRegionSetupSerializationBlock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRefreshSnapshotBlock {
    pub field: &'static str,
    pub count: usize,
}

/// Мировой владелец копии прежнего помодульного дубликата count-gate: одна
/// формула не размножается по потребителям snapshot-стадии.
pub fn legacy_refresh_count(
    field: &'static str,
    count: usize,
) -> Result<u32, WorldRefreshSnapshotBlock> {
    u32::try_from(count).map_err(|_| WorldRefreshSnapshotBlock { field, count })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldMainLoopRefreshDisposition {
    NotDue,
    MissingNetworkOwner,
    Refreshed(WorldRefreshInfoReport),
}

#[allow(
    clippy::large_enum_variant,
    reason = "полный отчёт возвращается по значению, чтобы не добавлять heap allocation в каждый MainLoop turn"
)]
#[derive(Debug, Eq, PartialEq)]
pub enum WorldMainLoopRefreshStageReport {
    BlockedMissingFact {
        elapsed_since_refresh_ms: u32,
        assigned_last_refresh_tick_ms: u32,
        block: WorldRefreshSnapshotBlock,
    },
    Complete {
        elapsed_since_refresh_ms: u32,
        refresh: WorldMainLoopRefreshDisposition,
        finished_at_ms: u32,
        elapsed_stage_ms: u32,
        accumulated_refresh_time_ms: u32,
        profile: Option<WorldMainLoopProfileReport>,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorldReloadProfileFlagsSnapshot {
    pub low: u32,
    pub high: u32,
}

#[derive(Debug, Default)]
pub struct WorldReloadProfileFlags {
    low: AtomicU32,
    high: AtomicU32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldRegionLoadSpec {
    pub region_id: i32,
    pub resource_id: u32,
    pub exp_scale: f32,
    pub region_type: i32,
    pub no_pk: bool,
    pub no_contribute: bool,
    pub name: Vec<u8>,
    pub game_server_index: u32,
    pub country: u8,
    pub notify: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldReloadOneScriptBlock {
    pub requested_path: Vec<u8>,
    pub normalized_map_key: Vec<u8>,
}

pub type WorldReloadOneScriptResult = Result<bool, WorldReloadOneScriptBlock>;

/// Позиционный `ScriptLoadContext` поверх живого reload-владельца; точечная
/// форма missing-resource повторяет исходный операторный текст.
pub struct WorldScriptLoadContext<'a, C: ?Sized>(pub &'a mut C);

impl<C: WorldReloadContext + ?Sized> ScriptLoadContext for WorldScriptLoadContext<'_, C> {
    fn read_resource(&mut self, path: &[u8]) -> Option<Vec<u8>> {
        self.0.read_resource(path)
    }

    fn indexed_files(&mut self, root: &[u8], extension: &[u8]) -> Option<Vec<Vec<u8>>> {
        self.0
            .default_client_resource()
            .find_file_list(root, extension)
    }

    fn loose_files(&mut self, pattern: &[u8], extension: &[u8]) -> Vec<Vec<u8>> {
        self.0.script_files(pattern, extension)
    }

    fn missing_resource(&mut self, path: &[u8]) {
        let mut message = b"Can't found ".to_vec();
        message.extend_from_slice(legacy_c_string_prefix(path));
        message.push(b'!');
        self.0.notify_reload_operator(b"Message", &message);
    }
}

/// Опубликованная неизменяемая проекция setup-владельцев одного runtime turn.
///
/// Она снимается после верхнего reload-gate и заново перед каждым FIFO
/// сообщением. Поэтому reload из GM/server-владельца виден следующему сообщению
/// того же turn, а живой `WorldReloadContext` не alias-ится с выданными ему же
/// Rust-ссылками.
#[derive(Clone)]
pub struct WorldMainLoopResourceSnapshot {
    pub registry: Arc<GoodsBasePropertiesRegistry>,
    pub original_name_index: Arc<GoodsOriginalNameIndex>,
    pub coefficients: PlayerPropertyCoefficients,
    pub player_list: CPlayerList,
    pub globe_setup: GlobeSetupSnapshot,
    pub region_router: RegionRouter,
    pub log_system: CLogSystem,
    pub gold_coin_index: u32,
}

pub trait WorldMainLoopResourceContext: WorldReloadContext {
    fn main_loop_resource_snapshot(&self) -> WorldMainLoopResourceSnapshot;
}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldReloadProfile {
    PlayerList,
    GoodsList,
    MonsterList,
    TradeList,
    SkillList,
    NewSkillMonsterList,
    GlobeSetup,
    GameSetup,
    StringTable,
    LogSystem,
    GmList,
    ScriptFile,
    RegionList,
    RegionLevelSetup,
    HitLevelSetup,
    Broadcast,
    AttackCity,
    InvalidStrings,
    GeneralVariableList,
    FactionParameters,
    VillageWar,
    FourNationWar,
    CityWar,
    FactionWar,
    Quest,
    CountryParameters,
    IncrementShop,
    Contribute,
    Prison,
    TimeToReturn,
    PreciousBox,
    FairyExp,
    ChangeBody,
    CountryWar,
    BattleFairyExp,
    BattleFairyCombine,
    Synthesis,
    DaKongXiangQian,
    EquipmentCompose,
    GoodsDestroy,
    HonorEliminate,
    TaoZhuang,
    CiQing,
    Jjc,
    AllThing,
    GodsBattle,
}

impl WorldReloadProfile {
    pub fn parse(value: &[u8]) -> Option<Self> {
        const NAMES: &[(&[u8], WorldReloadProfile)] = &[
            (b"PlayerList", WorldReloadProfile::PlayerList),
            (b"GoodsList", WorldReloadProfile::GoodsList),
            (b"MonsterList", WorldReloadProfile::MonsterList),
            (b"TradeList", WorldReloadProfile::TradeList),
            (b"SkillList", WorldReloadProfile::SkillList),
            (
                b"NewSkillMonsterList",
                WorldReloadProfile::NewSkillMonsterList,
            ),
            (b"GlobeSetup", WorldReloadProfile::GlobeSetup),
            (b"GameSetup", WorldReloadProfile::GameSetup),
            (b"StringTable", WorldReloadProfile::StringTable),
            (b"LogSystem", WorldReloadProfile::LogSystem),
            (b"GMList", WorldReloadProfile::GmList),
            (b"ScriptFile", WorldReloadProfile::ScriptFile),
            (b"RegionList", WorldReloadProfile::RegionList),
            (b"RegionLevelSetup", WorldReloadProfile::RegionLevelSetup),
            (b"HitLevelSetup", WorldReloadProfile::HitLevelSetup),
            (b"Broadcast", WorldReloadProfile::Broadcast),
            (b"AttackCitySys", WorldReloadProfile::AttackCity),
            (b"InvalidStr", WorldReloadProfile::InvalidStrings),
            (
                b"GeneralVariableList",
                WorldReloadProfile::GeneralVariableList,
            ),
            (b"FactionPara", WorldReloadProfile::FactionParameters),
            (b"VilWarPara", WorldReloadProfile::VillageWar),
            (b"FourNationWar", WorldReloadProfile::FourNationWar),
            (b"CityWarPara", WorldReloadProfile::CityWar),
            (b"FactionWarPara", WorldReloadProfile::FactionWar),
            (b"QuestData", WorldReloadProfile::Quest),
            (b"CountryParam", WorldReloadProfile::CountryParameters),
            (b"CountryPara", WorldReloadProfile::CountryParameters),
            (b"IncrementShopList", WorldReloadProfile::IncrementShop),
            (b"ContributeSetup", WorldReloadProfile::Contribute),
            (b"PrisonConf", WorldReloadProfile::Prison),
            (b"TimeToReturn", WorldReloadProfile::TimeToReturn),
            (b"PreciousBoxConf", WorldReloadProfile::PreciousBox),
            (b"FairyExpConf", WorldReloadProfile::FairyExp),
            (b"ChangeBodyConf", WorldReloadProfile::ChangeBody),
            (b"CountryWar", WorldReloadProfile::CountryWar),
            (b"BattleFairyExpConfig", WorldReloadProfile::BattleFairyExp),
            (
                b"BattleFairyCombineConfig",
                WorldReloadProfile::BattleFairyCombine,
            ),
            (b"SynthesisList", WorldReloadProfile::Synthesis),
            (b"DaKongXiangQian", WorldReloadProfile::DaKongXiangQian),
            (b"EquipmentCompose", WorldReloadProfile::EquipmentCompose),
            (b"GoodsDestroyConf", WorldReloadProfile::GoodsDestroy),
            (b"HonorElimilate", WorldReloadProfile::HonorEliminate),
            (b"taozhuang", WorldReloadProfile::TaoZhuang),
            (b"ciqing", WorldReloadProfile::CiQing),
            (b"JJcConfig", WorldReloadProfile::Jjc),
            (b"Allthing", WorldReloadProfile::AllThing),
            (b"godsBattle", WorldReloadProfile::GodsBattle),
        ];
        let value = legacy_c_string_prefix(value);
        NAMES
            .iter()
            .find(|(name, _)| value.eq_ignore_ascii_case(name))
            .map(|(_, profile)| *profile)
    }
}

impl WorldReloadProfileFlags {
    pub const fn new(low: u32, high: u32) -> Self {
        Self {
            low: AtomicU32::new(low),
            high: AtomicU32::new(high),
        }
    }

    pub fn snapshot(&self) -> WorldReloadProfileFlagsSnapshot {
        WorldReloadProfileFlagsSnapshot {
            low: self.low.load(Ordering::Relaxed),
            high: self.high.load(Ordering::Relaxed),
        }
    }

    pub fn set_low_bits(&self, mask: u32) -> u32 {
        let updated = self.low.load(Ordering::Relaxed) | mask;
        self.low.store(updated, Ordering::Relaxed);
        updated
    }

    pub fn set_high_bits(&self, mask: u32) -> u32 {
        let updated = self.high.load(Ordering::Relaxed) | mask;
        self.high.store(updated, Ordering::Relaxed);
        updated
    }

    pub fn has_pending(&self) -> bool {
        self.low.load(Ordering::Relaxed) != 0 || self.high.load(Ordering::Relaxed) != 0
    }

    pub fn contains(&self, half: WorldReloadFlagHalf, mask: u32) -> bool {
        match half {
            WorldReloadFlagHalf::Low => self.low.load(Ordering::Relaxed) & mask != 0,
            WorldReloadFlagHalf::High => self.high.load(Ordering::Relaxed) & mask != 0,
        }
    }

 /// DIFF-3 (машинная досверка по точной паре `Nworldserver.exe` +
 /// `WorldServer.pdb`, RSDS `289F1FB3-…` age 1; дамп
 /// `.local/verify-c5c/dis_reload_profiles.txt`): при обработке любого
 /// low-бита оригинал выполняет `low &= ~mask; high = 0`, поэтому первый
 /// обработанный low-профиль гасит все pending high-профили. High-бит
 /// снимает только свою маску.
    pub fn consume(&self, action: WorldReloadAction) {
        match action.half {
            WorldReloadFlagHalf::Low => {
                let remaining = self.low.load(Ordering::Relaxed) & !action.mask;
                self.low.store(remaining, Ordering::Relaxed);
                self.high.store(0, Ordering::Relaxed);
            }
            WorldReloadFlagHalf::High => {
                let remaining = self.high.load(Ordering::Relaxed) & !action.mask;
                self.high.store(remaining, Ordering::Relaxed);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldReloadFlagHalf {
    Low,
    High,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldReloadConfLogBlock {
    MissingNetworkServerOwner,
    MissingWorldNumber,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldReloadConfLogDisposition {
    SuppressedEmptyProfile,
 /// DIFF-5: Broadcast-профиль при отсутствии `setup/sysboardcast.ini`
 /// машинно завершается ранним `return 0` до записи conf-log.
    SuppressedBroadcastMissingFile,
    Published {
        text: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldReloadProfileEvent {
    pub half: WorldReloadFlagHalf,
    pub mask: u32,
    pub reload_profile: &'static [u8],
    pub log_profile: &'static [u8],
    pub flags_after_clear: WorldReloadProfileFlagsSnapshot,
    pub reload_result: i32,
    pub log: WorldReloadConfLogDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldReloadRegionSetupBlock {
    pub region_id: i32,
    pub source: WorldRegionSetupSerializationBlock,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldReloadProfilesReport {
    Complete {
        events: Vec<WorldReloadProfileEvent>,
        remaining_flags: WorldReloadProfileFlagsSnapshot,
    },
    BlockedMissingFact {
        completed_events: Vec<WorldReloadProfileEvent>,
        half: WorldReloadFlagHalf,
        mask: u32,
        reload_profile: &'static [u8],
        log_profile: &'static [u8],
        flags_after_clear: WorldReloadProfileFlagsSnapshot,
        reload_result: i32,
        block: WorldReloadConfLogBlock,
    },
    BlockedRegionSetup {
        completed_events: Vec<WorldReloadProfileEvent>,
        half: WorldReloadFlagHalf,
        mask: u32,
        flags_after_clear: WorldReloadProfileFlagsSnapshot,
        block: WorldReloadRegionSetupBlock,
    },
    BlockedReloadOwner {
        completed_events: Vec<WorldReloadProfileEvent>,
        half: WorldReloadFlagHalf,
        mask: u32,
        reload_profile: &'static [u8],
        log_profile: &'static [u8],
        flags_after_clear: WorldReloadProfileFlagsSnapshot,
        block: WorldReloadBlock,
    },
}

#[derive(Clone, Copy)]
pub enum WorldReloadActionKind {
    Reload,
    ReloadAllRegions,
}

/// Один шаг диспетчера reload-профилей.
///
/// `first_option`/`second_option` — буквальные аргументы машинного
/// `ReLoad(profile, send_to_game_servers, reload_server_resources)`.
/// Обнуление pending high-флагов при обработке любого low-бита —
/// безусловная машинная форма (DIFF-3, см.
/// [`WorldReloadProfileFlags::consume`]), отдельного per-row ключа здесь
/// нет.
#[derive(Clone, Copy)]
pub struct WorldReloadAction {
    pub half: WorldReloadFlagHalf,
    pub mask: u32,
    pub reload_profile: &'static [u8],
    pub log_profile: &'static [u8],
    pub first_option: bool,
    pub second_option: bool,
    pub kind: WorldReloadActionKind,
}

impl WorldReloadAction {
    const fn reload_low(
        mask: u32,
        profile: &'static [u8],
        first_option: bool,
        second_option: bool,
    ) -> Self {
        Self {
            half: WorldReloadFlagHalf::Low,
            mask,
            reload_profile: profile,
            log_profile: profile,
            first_option,
            second_option,
            kind: WorldReloadActionKind::Reload,
        }
    }

    const fn reload_high(mask: u32, profile: &'static [u8]) -> Self {
        Self {
            half: WorldReloadFlagHalf::High,
            mask,
            reload_profile: profile,
            log_profile: profile,
            first_option: true,
            second_option: true,
            kind: WorldReloadActionKind::Reload,
        }
    }

    const fn reload_high_with_log(
        mask: u32,
        reload_profile: &'static [u8],
        log_profile: &'static [u8],
    ) -> Self {
        Self {
            half: WorldReloadFlagHalf::High,
            mask,
            reload_profile,
            log_profile,
            first_option: true,
            second_option: true,
            kind: WorldReloadActionKind::Reload,
        }
    }

    const fn reload_all_regions(mask: u32) -> Self {
        Self {
            half: WorldReloadFlagHalf::Low,
            mask,
            reload_profile: b"AllRegion",
            log_profile: b"AllRegion",
            first_option: false,
            second_option: false,
            kind: WorldReloadActionKind::ReloadAllRegions,
        }
    }
}

/// Таблица профилей `ReloadConf` в машинном порядке обхода.
///
/// DIFF-4 (машинная досверка): ChangeBodyConf (lo 0x80000000),
/// SynthesisList (lo 0x50000000) и Allthing (lo 0x100) оригинал вызывает
/// `ReLoad` с `(send=1, resources=1)`, поэтому второй bool этих строк —
/// `true`. DIFF-6: conf-log godsBattle записывается машинным написанием
/// `godsBattle`.
pub const WORLD_RELOAD_ACTIONS: &[WorldReloadAction] = &[
    WorldReloadAction::reload_low(0x4000_0000, b"StringTable", true, true),
    WorldReloadAction::reload_low(0x0000_0001, b"LogSystem", true, true),
    WorldReloadAction::reload_low(0x0000_0002, b"GMList", true, true),
    WorldReloadAction::reload_low(0x0000_0004, b"Broadcast", false, true),
    WorldReloadAction::reload_low(0x0000_0008, b"VilWarPara", false, true),
    WorldReloadAction::reload_low(0x0000_0010, b"CityWarPara", false, true),
    WorldReloadAction::reload_low(0x0000_0020, b"IncrementShopList", true, true),
    WorldReloadAction::reload_low(0x0000_0040, b"GameSetup", true, true),
    WorldReloadAction::reload_low(0x0000_0080, b"InvalidStr", true, true),
    WorldReloadAction::reload_low(0x0000_0100, b"PlayerList", true, true),
    WorldReloadAction::reload_low(0x0000_0200, b"GoodsList", true, true),
    WorldReloadAction::reload_low(0x0000_0400, b"MonsterList", true, true),
    WorldReloadAction::reload_low(0x0000_0800, b"TradeList", true, true),
    WorldReloadAction::reload_low(0x0000_1000, b"SkillList", true, true),
    WorldReloadAction::reload_low(0x0000_2000, b"GlobeSetup", true, true),
    WorldReloadAction::reload_low(0x0000_4000, b"ScriptFile", true, true),
    WorldReloadAction::reload_high(0x0000_0001, b"NewSkillMonsterList"),
    WorldReloadAction::reload_low(0x0001_0000, b"GeneralVariableList", false, true),
    WorldReloadAction::reload_low(0x0002_0000, b"RegionList", true, true),
    WorldReloadAction::reload_low(0x0004_0000, b"RegionLevelSetup", true, true),
    WorldReloadAction::reload_all_regions(0x0010_0000),
    WorldReloadAction::reload_low(0x0020_0000, b"FactionPara", false, true),
    WorldReloadAction::reload_low(0x0040_0000, b"FactionWarPara", false, true),
    WorldReloadAction::reload_low(0x0080_0000, b"QuestData", false, true),
    WorldReloadAction::reload_low(0x0100_0000, b"ContributeSetup", true, true),
    WorldReloadAction::reload_low(0x0200_0000, b"PrisonConf", true, true),
    WorldReloadAction::reload_low(0x0400_0000, b"TimeToReturn", false, true),
    WorldReloadAction::reload_low(0x0800_0000, b"PreciousBoxConf", true, true),
    WorldReloadAction::reload_low(0x2000_0000, b"FairyExpConf", true, true),
    WorldReloadAction::reload_low(0x8000_0000, b"ChangeBodyConf", true, true),
    WorldReloadAction::reload_low(0x1000_0000, b"CountryWar", false, true),
    WorldReloadAction::reload_high(0x0000_0020, b"FourNationWar"),
    WorldReloadAction::reload_high(0x0000_0400, b"BattleFairyExpConfig"),
    WorldReloadAction::reload_high(0x0000_0800, b"BattleFairyCombineConfig"),
    WorldReloadAction::reload_low(0x5000_0000, b"SynthesisList", true, true),
    WorldReloadAction::reload_high(0x0000_0080, b"EquipmentCompose"),
    WorldReloadAction::reload_high(0x0000_1000, b"HonorElimilate"),
    WorldReloadAction::reload_high(0x0000_2000, b"ciqing"),
    WorldReloadAction::reload_high_with_log(0x0001_0000, b"godsBattle", b"godsBattle"),
    WorldReloadAction::reload_high(0x0000_8000, b"taozhuang"),
    WorldReloadAction::reload_high(0x0000_4000, b"JJcConfig"),
    WorldReloadAction::reload_low(0x0000_0100, b"Allthing", true, true),
];

#[derive(Debug, Default)]
pub struct WorldPlayerRanksRequestState {
    requested: AtomicBool,
}

impl WorldPlayerRanksRequestState {
    pub fn request(&self) {
        self.requested.store(true, Ordering::Relaxed);
    }

    pub fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Relaxed)
    }

    pub fn take_if_requested(&self) -> bool {
        if !self.requested.load(Ordering::Relaxed) {
            return false;
        }
        self.requested.store(false, Ordering::Relaxed);
        true
    }
}

#[derive(Debug)]
pub enum WorldPlayerRanksMaintenanceDisposition {
    NotRequested,
    Updated {
        stat: PlayerRanksStatRunReport,
        publication: PlayerRanksGameServerUpdate,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldHonorRanksMaintenanceDisposition {
    Disabled,
    AlreadyCurrent {
        current_day: u32,
        sort_day: u32,
    },
    Updated {
        current_day: u32,
        previous_sort_day: u32,
        started_at_ms: u32,
        finished_at_ms: u32,
        elapsed_ms: u32,
        start_log: AddLogTextDisposition,
        complete_log: AddLogTextDisposition,
        rollover: HonorRanksNewDayReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldHonorRanksMaintenanceBlock {
    pub current_day: u32,
    pub previous_sort_day: u32,
    pub started_at_ms: u32,
    pub start_log: AddLogTextDisposition,
    pub source: HonorRanksNewDayBlock,
}

#[derive(Debug)]
pub enum WorldAuctionBangMaintenanceDisposition {
    AlreadyCurrent {
        current_month_day: i32,
        old_month_day: i32,
    },
    Updated {
        current_month_day: i32,
        previous_old_month_day: i32,
        update_succeeded: bool,
        outcome: AuctionBangUpdateOutcome,
        start_log: AddLogTextDisposition,
        result_log: AddLogTextDisposition,
    },
}

#[derive(Debug)]
pub enum WorldMainLoopMaintenanceBlock {
    PlayerRanksStat(PlayerRanksStatRunBlock),
    PlayerRanksSerialization(PlayerRanksSerializationBlock),
    HonorRanks(WorldHonorRanksMaintenanceBlock),
}

#[derive(Debug)]
pub struct WorldMainLoopMaintenanceReport {
    pub player_ranks: WorldPlayerRanksMaintenanceDisposition,
    pub honor_ranks: WorldHonorRanksMaintenanceDisposition,
    pub auction_bang: WorldAuctionBangMaintenanceDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldMainLoopLargessGateReport {
    BlockedMissingFact {
        field: &'static str,
    },
    Disabled {
        load_interval_ms: u32,
        pass_count: u32,
    },
    Waiting {
        load_interval_ms: u32,
        pass_count: u32,
        elapsed_ms: u32,
    },
    StartWorkerRequested {
        load_interval_ms: u32,
        pass_count: u32,
        elapsed_ms: u32,
        requested_at_ms: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMainLoopProfileSnapshot {
    pub ai_calls: u32,
    pub ai_time_ms: u32,
    pub refresh_text_time_ms: u32,
    pub process_message_time_ms: u32,
    pub login_server_message_time_ms: u32,
    pub game_server_message_time_ms: u32,
    pub net_session_time_ms: u32,
    pub faction_war_time_ms: u32,
    pub timer_time_ms: u32,
    pub process_player_data_queue_time_ms: u32,
    pub session_factory_time_ms: u32,
    pub save_point_time_ms: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldMainLoopProfileReport {
    pub elapsed_since_last_publish_ms: u32,
    pub snapshot: WorldMainLoopProfileSnapshot,
    pub log: AddLogTextDisposition,
}

pub fn initialize_main_loop_profile_if_needed<GetTick>(
    initialization: &mut WorldMainLoopInitializationState,
    profile: &mut WorldMainLoopProfileState,
    mut get_tick: GetTick,
) -> Option<WorldMainLoopProfileInitialization>
where
    GetTick: FnMut() -> u32,
{
    if initialization.mask & 1 != 0 {
        return None;
    }

    let previous_mask = initialization.mask;
    initialization.mask |= 1;
    let initial_report_tick_ms = get_tick();
    profile.last_published_at_ms = initial_report_tick_ms;
    Some(WorldMainLoopProfileInitialization {
        previous_mask,
        initialized_mask: initialization.mask,
        initial_report_tick_ms,
    })
}

pub fn initialize_main_loop_save_if_needed<GetTick>(
    initialization: &mut WorldMainLoopInitializationState,
    save: &mut WorldRunSaveTriggerState,
    mut get_tick: GetTick,
) -> Option<WorldMainLoopSaveInitialization>
where
    GetTick: FnMut() -> u32,
{
    if initialization.mask & 2 != 0 {
        return None;
    }

    let previous_mask = initialization.mask;
    initialization.mask |= 2;
    let initial_save_tick_ms = get_tick();
    save.last_save_point_time_ms = initial_save_tick_ms;
    Some(WorldMainLoopSaveInitialization {
        previous_mask,
        initialized_mask: initialization.mask,
        initial_save_tick_ms,
    })
}

pub fn initialize_main_loop_refresh_if_needed(
    initialization: &mut WorldMainLoopInitializationState,
    clocks: &mut WorldMainLoopClockState,
) -> Option<WorldMainLoopRefreshInitialization> {
    if initialization.mask & 4 != 0 {
        return None;
    }

    let previous_mask = initialization.mask;
    initialization.mask |= 4;
    let copied_current_tick_ms = clocks.current_tick_ms;
    clocks.last_refresh_tick_ms = copied_current_tick_ms;
    Some(WorldMainLoopRefreshInitialization {
        previous_mask,
        initialized_mask: initialization.mask,
        copied_current_tick_ms,
    })
}

pub fn update_main_loop_current_tick<GetTick>(
    clocks: &mut WorldMainLoopClockState,
    mut get_tick: GetTick,
) -> u32
where
    GetTick: FnMut() -> u32,
{
    let current_tick_ms = get_tick();
    clocks.current_tick_ms = current_tick_ms;
    current_tick_ms
}

pub fn start_main_loop_profile_stage<GetTick>(
    clocks: &mut WorldMainLoopClockState,
    mut get_tick: GetTick,
) -> u32
where
    GetTick: FnMut() -> u32,
{
    let started_at_ms = get_tick();
    clocks.stage_started_at_ms = started_at_ms;
    started_at_ms
}

pub fn initialize_main_loop_tail_clocks<GetTick>(
    initialization: &mut WorldMainLoopInitializationState,
    clocks: &mut WorldMainLoopTailClockState,
    mut get_tick: GetTick,
) -> WorldMainLoopTailClockInitialization
where
    GetTick: FnMut() -> u32,
{
    let previous_mask = initialization.mask;
    let initial_current_tick_ms = if initialization.mask & 0x10 == 0 {
        initialization.mask |= 0x10;
        let tick = get_tick();
        clocks.current_tick_ms = tick;
        Some(tick)
    } else {
        None
    };
    let initial_pacing_deadline_ms = if initialization.mask & 0x20 == 0 {
        initialization.mask |= 0x20;
        clocks.pacing_deadline_ms = clocks.current_tick_ms;
        Some(clocks.pacing_deadline_ms)
    } else {
        None
    };
    let initial_minute_started_at_ms = if initialization.mask & 0x40 == 0 {
        initialization.mask |= 0x40;
        let tick = get_tick();
        clocks.minute_started_at_ms = tick;
        Some(tick)
    } else {
        None
    };
    WorldMainLoopTailClockInitialization {
        previous_mask,
        initialized_mask: initialization.mask,
        initial_current_tick_ms,
        initial_pacing_deadline_ms,
        initial_minute_started_at_ms,
    }
}

#[derive(Debug)]
pub enum WorldProcessMessageStageReport {
    Blocked {
        started_at_ms: u32,
        error: WorldProcessMessageError,
    },
    Complete {
        started_at_ms: u32,
        outcome: WorldProcessMessageOutcome,
        finished_at_ms: u32,
        elapsed_ms: u32,
        accumulated_time_ms: u32,
        next_stage_started_at_ms: u32,
    },
}

#[derive(Debug)]
pub enum WorldProcessMessageError {
    MissingNetworkServerOwner,
    MissingCountryLimit(&'static str),
    ServerMessage(WorldServerMessageError),
}

impl fmt::Display for WorldProcessMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNetworkServerOwner => formatter
                .write_str("World ProcessMessage не может прочитать обязательный server-owner"),
            Self::MissingCountryLimit(parameter) => write!(
                formatter,
                "World ProcessMessage не получил country-параметр {parameter}"
            ),
            Self::ServerMessage(error) => error.fmt(formatter),
        }
    }
}

impl Error for WorldProcessMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingNetworkServerOwner | Self::MissingCountryLimit(_) => None,
            Self::ServerMessage(error) => Some(error),
        }
    }
}

pub fn current_country_save_limits(
    parameters: &CCountryParam,
) -> Result<CountryKingSaveLimits, WorldProcessMessageError> {
    Ok(CountryKingSaveLimits {
        control_point: parameters
            .max_king_control_point()
            .ok_or(WorldProcessMessageError::MissingCountryLimit(
                "_max_king_control_point",
            ))?,
        material_point: parameters
            .max_king_material_point()
            .ok_or(WorldProcessMessageError::MissingCountryLimit(
                "_max_king_material_point",
            ))?,
        war_point: parameters
            .max_king_war_point()
            .ok_or(WorldProcessMessageError::MissingCountryLimit(
                "_max_king_war_point",
            ))?,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMainLoopProfileInitialization {
    pub previous_mask: u32,
    pub initialized_mask: u32,
    pub initial_report_tick_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMainLoopSaveInitialization {
    pub previous_mask: u32,
    pub initialized_mask: u32,
    pub initial_save_tick_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldMainLoopRefreshInitialization {
    pub previous_mask: u32,
    pub initialized_mask: u32,
    pub copied_current_tick_ms: u32,
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}
