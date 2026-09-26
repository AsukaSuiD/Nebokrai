//! Арена `CJJcSystem` из WorldServer.
//!
//! Signed maps сохраняют числовой порядок; available regions выбираются по
//! возрастанию. `Run` снимает время до gate, обновляет rank timestamp до DB
//! load, затем выполняет reset и ordered fight pass.
//!
//! Timeout отсутствующего первого player удаляет fight и возвращает region;
//! обычный timeout лишь обнуляет start times, рассылает `0x80505` и сохраняет
//! запись. Исторический double-increment после erase пропускает следующую fight/
//! queue запись и намеренно сохранён.
//!
//! Weekly reset сравнивает настроенный день, но очищает флаг по жёсткому
//! `tm_wday != 0`, поэтому может повторяться в ту же минуту. Config loader
//! сохраняет частичную публикацию двух обязательных файлов и Win32 zero-defaults
//! необязательного `JJcConfig.ini`.
//!
//! Прямые ссылки на `CGame`/`CPlayer` заменены двумя узкими typed-швами
//! `JjcGameView` и `JjcPlayerView`; реализации живут у старых владельцев и
//! делегируют их inherent-методам. Состояние game server-а проходит owned
//! снимком `JjcGameServerSnapshot`.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::app::world_message::{CMessage, SendMessageError};
use nebokrai_shared::network::{CBaseMessage, ServerCommandHandle};

/// Узкий view чтения игрока для JJC уровней; реализация у владельца игрока
/// (старый `CPlayer`) делегирует inherent-методам, имена совпадают
/// специально (inherent priority исключает рекурсию).
pub trait JjcPlayerView {
    fn get_level(&self) -> u8;

    fn get_jjc_level(&self) -> u32;
}

/// Снимок состояния game server-а для JJC: связь и индекс.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JjcGameServerSnapshot {
    pub connected: bool,
    pub index: u32,
}

/// Узкий game-view JJC: игроки, привязка region-а и player-а к game server-у
/// и sender текущего GameServer. Реализация у владельца игры (старый `CGame`)
/// делегирует inherent-методам.
pub trait JjcGameView {
    fn jjc_online_player(&self, player_id: u32) -> Option<&dyn JjcPlayerView>;

    fn jjc_map_player(&self, map_key: u32) -> Option<&dyn JjcPlayerView>;

    fn jjc_region_exists(&self, region_id: i32) -> bool;

    fn jjc_region_game_server(&self, region_id: i32) -> Option<JjcGameServerSnapshot>;

    fn jjc_player_game_server(&self, player_id: i32) -> Option<JjcGameServerSnapshot>;

    fn jjc_game_server_sender(&self) -> Option<ServerCommandHandle>;
}

pub const JJC_REGION_LIST_PATH: &[u8] = b"data/JJCRegionlist.ini";
pub const JJC_LEVEL_LIST_PATH: &[u8] = b"data/JJcLevel.ini";
pub const JJC_CONFIG_PATH: &[u8] = b"setup/JJcConfig.ini";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjcRank {
    pub rank: i32,
    pub player_id: i32,
    pub jjc_level: u32,
    pub level: i32,
    pub name: [u8; 0x20],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JjcInfo {
    pub jjc_level: u32,
    pub old_region_id: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub opponent_id: i32,
    pub jjc_region_id: i32,
    pub start_time: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JjcRunConfig {
    pub use_jjc: i32,
    pub rank_interval_seconds: i32,
    pub pk_timeout_seconds: i32,
    pub region_id_min: i32,
    pub region_id_max: i32,
    pub max_regions_in_use: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjcConfigurationLoadReport {
    MissingRegionList,
    MissingLevelList {
        available_regions: usize,
    },
    Loaded {
        available_regions: usize,
        level_steps: usize,
        week_day: i32,
        hour: i32,
        minute: i32,
        second: i32,
        passed_weeks: i32,
        last_clear_time: i32,
        config_file_present: bool,
    },
}

impl JjcConfigurationLoadReport {
    pub const fn legacy_result(self) -> bool {
        matches!(self, Self::Loaded { .. })
    }

    pub const fn missing_path(self) -> Option<&'static [u8]> {
        match self {
            Self::MissingRegionList => Some(JJC_REGION_LIST_PATH),
            Self::MissingLevelList { .. } => Some(JJC_LEVEL_LIST_PATH),
            Self::Loaded { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjcCanApplyDisposition {
    Allowed,
    PlayerOrGameServerUnavailable,
    AlreadyQueuedOrFighting,
    CycleClosed,
}

impl JjcCanApplyDisposition {
    const fn legacy_result(self) -> i32 {
        match self {
            Self::Allowed => 0,
            Self::PlayerOrGameServerUnavailable => 1,
            Self::AlreadyQueuedOrFighting => 2,
            Self::CycleClosed => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjcRegionAcquireDisposition {
    Acquired {
        region_id: i32,
        removed_from_available: bool,
        inserted_into_in_use: bool,
    },
    AvailableEmpty,
    InUseExactlyAtLimit,
    GameServerUnavailable { region_id: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JjcRegionAcquireReport {
    pub available_before: usize,
    pub in_use_before: usize,
    pub disposition: JjcRegionAcquireDisposition,
}

impl JjcRegionAcquireReport {
    const fn region_id(&self) -> i32 {
        match self.disposition {
            JjcRegionAcquireDisposition::Acquired { region_id, .. } => region_id,
            JjcRegionAcquireDisposition::AvailableEmpty
            | JjcRegionAcquireDisposition::InUseExactlyAtLimit
            | JjcRegionAcquireDisposition::GameServerUnavailable { .. } => 0,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct JjcStartFightReport {
    pub region_id: i32,
    pub first_player_id: i32,
    pub second_player_id: i32,
    pub start_time: i32,
    pub region_delivery: Result<i32, SendMessageError>,
    pub first_player_delivery: Result<i32, SendMessageError>,
    pub second_player_delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum JjcApplyDisposition {
    Rejected(JjcCanApplyDisposition),
    RegionUnavailable(JjcRegionAcquireReport),
    WaitingForOpponent {
        region: JjcRegionAcquireReport,
        recycle: JjcRegionRecycleReport,
    },
    Matched {
        region: JjcRegionAcquireReport,
        opponent_id: i32,
        removed_unroutable_players: Vec<i32>,
        fight: JjcStartFightReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct JjcApplyReport {
    pub player_id: i32,
    pub legacy_result: i32,
    pub disposition: JjcApplyDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjcEndPkDisposition {
    RemovedIncompletePlayer {
        player_id: i32,
        opponent_id: i32,
        removed: bool,
    },
    Applied {
        player_id: i32,
        opponent_id: i32,
        removed_fight: bool,
        removed_player: bool,
        removed_opponent: bool,
        recycle: JjcRegionRecycleReport,
    },
    CorrelationMismatch {
        player_id: i32,
        opponent_id: i32,
        opponent_points_to: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JjcEndPkReport {
    pub requested_region_id: i32,
    pub disposition: JjcEndPkDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub struct JjcQuitNotification {
    pub player_id: i32,
    pub requested_region_id: i32,
    pub map_id: i32,
    pub delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum JjcQuitDisposition {
    PlayerMissing,
    EndedFight(JjcEndPkReport),
    RemovedApplication {
        opponent_id: i32,
        internal_region_id: i32,
        removed_player: bool,
        removed_opponent: bool,
        notifications: [JjcQuitNotification; 2],
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct JjcQuitReport {
    pub player_id: i32,
    pub requested_region_id: i32,
    pub disposition: JjcQuitDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JjcLocalTime {
    pub second: i32,
    pub minute: i32,
    pub hour: i32,
    pub month_day: i32,
    pub month: i32,
    pub year_since_1900: i32,
    pub week_day: i32,
    pub year_day: i32,
    pub daylight_saving: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JjcSystemTime {
    pub year: u16,
    pub month: u16,
    pub week_day: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjcLogEvent {
    Closed,
    RankLoaded {
        success: bool,
        item_count: i32,
    },
    WeekUpdateStarted {
        local_time: JjcSystemTime,
    },
    WeekUpdateThreadFailed,
    SeasonUpdateStarted,
    WeekOrSeasonUpdateFinished {
        elapsed_ms: u32,
    },
    WeekClearDatabaseSlow {
        elapsed_ms: u32,
    },
    RegionNotConfigured {
        region_id: i32,
    },
    RegionNotConnected {
        region_id: i32,
    },
    InvalidRegion {
        region_id: i32,
    },
    FightTimedOut {
        region_id: i32,
        first_player_id: i32,
        second_player_id: i32,
    },
    ApplyStarted { player_id: i32 },
    ApplyFinished { player_id: i32 },
    ApplyPlayerMissing { player_id: i32 },
    PlayerGameServerUnavailable { player_id: i32 },
    PlayerServerIdZero { player_id: i32 },
    AvailableRegionsEmpty,
    UsedRegionsFull,
    PvpRegionGameServerUnavailable { region_id: i32 },
    RegionPoolSnapshot { in_use: usize, available: usize },
    ApplyMatched {
        player_id: i32,
        region_id: i32,
        opponent_id: i32,
    },
    PkEnded {
        region_id: i32,
        player_id: i32,
    },
    PkCorrelationMismatch {
        region_id: i32,
        player_id: i32,
        opponent_id: i32,
        opponent_points_to: i32,
    },
    QuitWhileFighting {
        player_id: i32,
        opponent_id: i32,
        region_id: i32,
    },
    QuitAfterApplication {
        player_id: i32,
        opponent_id: i32,
        region_id: i32,
    },
}

pub trait JjcRunContext {
    fn current_time_seconds(&mut self) -> i32;

    fn local_time(&mut self, timestamp: i32) -> JjcLocalTime;

    fn system_time(&mut self) -> JjcSystemTime;
    fn tick_count_ms(&mut self) -> u32;

    fn load_jjc_rank(&mut self, ranks: &mut Vec<JjcRank>) -> bool;

    fn start_jjc_week_clear(&mut self) -> bool;

    fn clear_jjc_season(&mut self) -> bool;

    fn write_private_profile_string(&mut self, section: &[u8], key: &[u8], value: &[u8]) -> bool;

    fn log(&mut self, event: JjcLogEvent);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JjcRunBlock {
    pub rank_count: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct JjcUpdateBroadcastReport {
    pub message_type: u32,
    pub delivery: Result<i32, SendMessageError>,
    pub database_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JjcIniWriteReport {
    pub passed_weeks: bool,
    pub last_clear_time: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct JjcWeeklyResetUpdateReport {
    pub local_clock: JjcSystemTime,
    pub started_at_ms: u32,
    pub week: JjcUpdateBroadcastReport,
    pub passed_weeks: i32,
    pub last_clear_time: i32,
    pub ini_writes: JjcIniWriteReport,
    pub season: Option<JjcUpdateBroadcastReport>,
    pub finished_at_ms: u32,
    pub elapsed_ms: u32,
    pub warning_checked_at_ms: u32,
    pub warning_elapsed_ms: Option<u32>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct JjcResetReport {
    pub local_time: JjcLocalTime,
    pub update: Option<JjcWeeklyResetUpdateReport>,
    pub flag_reset_for_non_sunday: bool,
    pub is_this_week_updated_after: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjcRankRefreshReport {
    Waiting {
        elapsed_seconds: i32,
    },
    Loaded {
        elapsed_seconds: i32,
        success: bool,
        item_count: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjcRegionServerLookup {
    Found { game_server_id: i32 },
    RegionNotConfigured,
    GameServerNotConnected,
}

impl JjcRegionServerLookup {
    const fn map_id(self) -> i32 {
        match self {
            Self::Found { game_server_id } => game_server_id,
            Self::RegionNotConfigured | Self::GameServerNotConnected => -1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjcRegionRecycleReport {
    Recycled {
        region_id: i32,
        removed_from_in_use: bool,
        inserted_into_available: bool,
    },
    Invalid {
        region_id: i32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum JjcFightRunReport {
    NonPositiveRegion {
        region_id: i32,
        first_player_id: i32,
        second_player_id: i32,
    },
    Waiting {
        region_id: i32,
        first_player_id: i32,
        second_player_id: i32,
        start_time: i32,
        elapsed_seconds: i32,
    },
    TimedOut {
        region_id: i32,
        first_player_id: i32,
        second_player_id: i32,
        lookup: JjcRegionServerLookup,
        delivery: Result<i32, SendMessageError>,
    },
    RemovedMissingFirstPlayer {
        region_id: i32,
        first_player_id: i32,
        second_player_id: i32,
        recycle: JjcRegionRecycleReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum JjcRunReport {
    Closed {
        current_time: i32,
    },
    Complete {
        current_time: i32,
        rank_refresh: JjcRankRefreshReport,
        reset: JjcResetReport,
        fights: Vec<JjcFightRunReport>,
        skipped_after_removal: Vec<i32>,
    },
}

pub struct CJJcSystem {
    queue: BTreeMap<i32, JjcInfo>,
    fighting_list: BTreeMap<i32, (i32, i32)>,
    regions_left: BTreeSet<i32>,
    regions_in_use: HashSet<i32>,
    level_list: BTreeMap<(i32, i32), i32>,
    ranks: Vec<JjcRank>,
    week_to_clear_day: i32,
    week_clear_hour: i32,
    week_clear_minute: i32,
    week_clear_second: i32,
    passed_weeks: i32,
    last_clear_time: i32,
    is_this_week_updated: bool,
    last_rank_refresh_time: i32,
}

impl CJJcSystem {
    pub fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
            fighting_list: BTreeMap::new(),
            regions_left: BTreeSet::new(),
            regions_in_use: HashSet::new(),
            level_list: BTreeMap::new(),
            ranks: Vec::new(),
            week_to_clear_day: 0,
            week_clear_hour: 0,
            week_clear_minute: 0,
            week_clear_second: 0,
            passed_weeks: 0,
            last_clear_time: 0,
            is_this_week_updated: false,
            last_rank_refresh_time: 0,
        }
    }

    pub fn ranks(&self) -> &[JjcRank] {
        &self.ranks
    }

    pub fn set_week_clear_schedule(
        &mut self,
        week_day: i32,
        hour: i32,
        minute: i32,
        second: i32,
    ) {
        self.week_to_clear_day = week_day;
        self.week_clear_hour = hour;
        self.week_clear_minute = minute;
        self.week_clear_second = second;
    }

    pub fn set_clear_state(&mut self, passed_weeks: i32, last_clear_time: i32) {
        self.passed_weeks = passed_weeks;
        self.last_clear_time = last_clear_time;
    }

 /// Загружает три resource-файла `LoadJJcConfig` без Win32/iostream.
 ///
 /// Наличие двух marker-файлов определяет старый bool-result. Отсутствующий
 /// `JJcConfig.ini` эквивалентен шести defaults `GetPrivateProfileIntA(0)`.
 /// Невалидная marker-строка безопасно пропускается вместо переноса
 /// внутреннего failbit/stale-value дефекта `operator>>`.
    pub fn load_configuration(
        &mut self,
        region_source: Option<&[u8]>,
        level_source: Option<&[u8]>,
        config_source: Option<&[u8]>,
    ) -> JjcConfigurationLoadReport {
        let Some(region_source) = region_source else {
            return JjcConfigurationLoadReport::MissingRegionList;
        };
        let regions = parse_jjc_regions(region_source);
        self.replace_available_regions(regions);
        let available_regions = self.regions_left.len();

        let Some(level_source) = level_source else {
            return JjcConfigurationLoadReport::MissingLevelList {
                available_regions,
            };
        };
        let levels = parse_jjc_level_steps(level_source);
        self.replace_level_steps(levels);
        let level_steps = self.level_list.len();

        let ini = parse_jjc_ini(config_source.unwrap_or_default());
        let week_day = if (0..=6).contains(&ini.week_day) {
            ini.week_day
        } else {
            0
        };
        self.set_week_clear_schedule(week_day, ini.hour, ini.minute, ini.second);
        self.set_clear_state(ini.passed_weeks, ini.last_clear_time);
        JjcConfigurationLoadReport::Loaded {
            available_regions,
            level_steps,
            week_day,
            hour: ini.hour,
            minute: ini.minute,
            second: ini.second,
            passed_weeks: ini.passed_weeks,
            last_clear_time: ini.last_clear_time,
            config_file_present: config_source.is_some(),
        }
    }

    pub fn replace_available_regions(
        &mut self,
        regions: impl IntoIterator<Item = i32>,
    ) {
        self.regions_left = regions.into_iter().collect();
    }

    pub fn replace_level_steps(
        &mut self,
        levels: impl IntoIterator<Item = ((i32, i32), i32)>,
    ) {
        self.level_list = levels.into_iter().collect();
    }

    fn level_step(&self, level: i32) -> i32 {
        self.level_list
            .iter()
            .find_map(|(&(minimum, maximum), &step)| {
                (minimum <= level && level <= maximum).then_some(step)
            })
            .unwrap_or(0)
    }

    fn is_player_in_pk(&self, player_id: i32, config: JjcRunConfig) -> bool {
        let Some(info) = self.queue.get(&player_id) else {
            return false;
        };
        config.region_id_min <= info.jjc_region_id
            && info.jjc_region_id <= config.region_id_max
            && self.fighting_list.contains_key(&info.jjc_region_id)
    }

    fn can_apply_jjc_now<Game: JjcGameView + ?Sized>(
        &self,
        game: &Game,
        player_id: i32,
        config: JjcRunConfig,
    ) -> JjcCanApplyDisposition {
        let Some(_player) = game.jjc_online_player(player_id as u32) else {
            return JjcCanApplyDisposition::PlayerOrGameServerUnavailable;
        };
        let Some(server) = game.jjc_player_game_server(player_id) else {
            return JjcCanApplyDisposition::PlayerOrGameServerUnavailable;
        };
        if !server.connected {
            return JjcCanApplyDisposition::PlayerOrGameServerUnavailable;
        }
        if self.queue.contains_key(&player_id) || self.is_player_in_pk(player_id, config) {
            return JjcCanApplyDisposition::AlreadyQueuedOrFighting;
        }
        if self.passed_weeks > 0 && self.passed_weeks % 8 == 0 {
            return JjcCanApplyDisposition::CycleClosed;
        }
        JjcCanApplyDisposition::Allowed
    }

    fn acquire_region<Game: JjcGameView + ?Sized, Context: JjcRunContext + ?Sized>(
        &mut self,
        game: &Game,
        config: JjcRunConfig,
        context: &mut Context,
    ) -> JjcRegionAcquireReport {
        let available_before = self.regions_left.len();
        let in_use_before = self.regions_in_use.len();
        let Some(&region_id) = self.regions_left.first() else {
            context.log(JjcLogEvent::AvailableRegionsEmpty);
            return JjcRegionAcquireReport {
                available_before,
                in_use_before,
                disposition: JjcRegionAcquireDisposition::AvailableEmpty,
            };
        };
        if in_use_before as i32 == config.max_regions_in_use {
            context.log(JjcLogEvent::UsedRegionsFull);
            return JjcRegionAcquireReport {
                available_before,
                in_use_before,
                disposition: JjcRegionAcquireDisposition::InUseExactlyAtLimit,
            };
        }
        if Self::get_region_server_id(game, context, region_id).map_id() < 0 {
            context.log(JjcLogEvent::PvpRegionGameServerUnavailable { region_id });
            return JjcRegionAcquireReport {
                available_before,
                in_use_before,
                disposition: JjcRegionAcquireDisposition::GameServerUnavailable { region_id },
            };
        }
        context.log(JjcLogEvent::RegionPoolSnapshot {
            in_use: in_use_before,
            available: available_before,
        });
        let removed_from_available = self.regions_left.remove(&region_id);
        let inserted_into_in_use = self.regions_in_use.insert(region_id);
        JjcRegionAcquireReport {
            available_before,
            in_use_before,
            disposition: JjcRegionAcquireDisposition::Acquired {
                region_id,
                removed_from_available,
                inserted_into_in_use,
            },
        }
    }

    fn get_one_opponent<Game: JjcGameView + ?Sized, Context: JjcRunContext + ?Sized>(
        &mut self,
        game: &Game,
        player_id: i32,
        config: JjcRunConfig,
        context: &mut Context,
    ) -> (i32, Vec<i32>) {
        let Some(player) = game.jjc_map_player(player_id as u32) else {
            return (0, Vec::new());
        };
        let player_level = player.get_level() as i32;
        let player_jjc_level = player.get_jjc_level();
        let mut best_distance = player_jjc_level;
        let mut opponent_id = 0;
        let mut removed_unroutable_players = Vec::new();
        let candidates = self.queue.keys().copied().collect::<Vec<_>>();

        let mut candidate_index = 0;
        while candidate_index < candidates.len() {
            let candidate_id = candidates[candidate_index];
            if candidate_id == player_id || self.is_player_in_pk(candidate_id, config) {
                candidate_index += 1;
                continue;
            }
            let Some(candidate) = game.jjc_map_player(candidate_id as u32) else {
                candidate_index += 1;
                continue;
            };
            let Some(server) = game.jjc_player_game_server(candidate_id) else {
                context.log(JjcLogEvent::PlayerGameServerUnavailable {
                    player_id: candidate_id,
                });
                candidate_index += 1;
                continue;
            };
            if !server.connected {
                context.log(JjcLogEvent::PlayerGameServerUnavailable {
                    player_id: candidate_id,
                });
                candidate_index += 1;
                continue;
            }
            if server.index == 0 {
                context.log(JjcLogEvent::PlayerServerIdZero {
                    player_id: candidate_id,
                });
                if self
                    .queue
                    .get(&candidate_id)
                    .is_some_and(|info| info.old_region_id == 0)
                {
                    self.queue.remove(&candidate_id);
                    removed_unroutable_players.push(candidate_id);
 // `GetOneOpponent` увеличивал iterator до erase, затем
 // запускал общий increment loop-а. Сохраняем наблюдаемый
 // пропуск successor без alias на удалённую BTreeMap entry.
                    candidate_index += 2;
                    continue;
                }
                candidate_index += 1;
                continue;
            }

            let player_step = self.level_step(player_level);
            let candidate_step = self.level_step(candidate.get_level() as i32);
            let delta = candidate
                .get_jjc_level()
                .wrapping_sub(player_jjc_level) as i32;
            let distance = delta.wrapping_abs();
            if candidate_step == player_step
                && player_step > 0
                && distance <= best_distance as i32
            {
                best_distance = distance as u32;
                opponent_id = candidate_id;
            }
            candidate_index += 1;
        }
        (opponent_id, removed_unroutable_players)
    }

    fn player_server_map_id<Game: JjcGameView + ?Sized, Context: JjcRunContext + ?Sized>(
        game: &Game,
        context: &mut Context,
        player_id: i32,
    ) -> i32 {
        let Some(server) = game.jjc_player_game_server(player_id) else {
            context.log(JjcLogEvent::PlayerGameServerUnavailable { player_id });
            return 0;
        };
        if !server.connected {
            context.log(JjcLogEvent::PlayerGameServerUnavailable { player_id });
            return 0;
        }
        server.index as i32
    }

    fn start_fight<Game: JjcGameView + ?Sized, Context: JjcRunContext + ?Sized>(
        &mut self,
        game: &Game,
        context: &mut Context,
        region_id: i32,
        first_player_id: i32,
        second_player_id: i32,
    ) -> JjcStartFightReport {
        self.fighting_list
            .insert(region_id, (first_player_id, second_player_id));
        let start_time = context.current_time_seconds();
        self.queue.entry(first_player_id).or_default().start_time = start_time;
        self.queue.entry(second_player_id).or_default().start_time = start_time;
        let first = *self.queue.entry(first_player_id).or_default();
        let second = *self.queue.entry(second_player_id).or_default();

        let mut region_message = CMessage::new(0x0008_0503);
        add_jjc_info(region_message.base_mut(), first);
        add_jjc_info(region_message.base_mut(), second);
        let region_map_id = Self::get_region_server_id(game, context, region_id).map_id();
        let region_delivery =
            region_message.send_to_map_id(game.jjc_game_server_sender().as_ref(), region_map_id);

        let mut first_message = CMessage::new(0x0008_0504);
        first_message.base_mut().add_long(first_player_id);
        first_message.base_mut().add_long(region_id);
        let first_map_id = Self::player_server_map_id(game, context, first_player_id);
        let first_player_delivery = first_message
            .send_to_map_id(game.jjc_game_server_sender().as_ref(), first_map_id);

        let mut second_message = CMessage::new(0x0008_0504);
        second_message.base_mut().add_long(second_player_id);
        second_message.base_mut().add_long(region_id);
        let second_map_id = Self::player_server_map_id(game, context, second_player_id);
        let second_player_delivery = second_message
            .send_to_map_id(game.jjc_game_server_sender().as_ref(), second_map_id);

        JjcStartFightReport {
            region_id,
            first_player_id,
            second_player_id,
            start_time,
            region_delivery,
            first_player_delivery,
            second_player_delivery,
        }
    }

    pub fn apply_player<Game: JjcGameView + ?Sized, Context: JjcRunContext + ?Sized>(
        &mut self,
        game: &Game,
        context: &mut Context,
        info: JjcInfo,
        player_id: i32,
        config: JjcRunConfig,
    ) -> JjcApplyReport {
        let gate = self.can_apply_jjc_now(game, player_id, config);
        if gate != JjcCanApplyDisposition::Allowed {
            return JjcApplyReport {
                player_id,
                legacy_result: gate.legacy_result(),
                disposition: JjcApplyDisposition::Rejected(gate),
            };
        }
        let Some(server) = game.jjc_player_game_server(player_id) else {
            context.log(JjcLogEvent::PlayerGameServerUnavailable { player_id });
            return JjcApplyReport {
                player_id,
                legacy_result: 1,
                disposition: JjcApplyDisposition::Rejected(
                    JjcCanApplyDisposition::PlayerOrGameServerUnavailable,
                ),
            };
        };
        if !server.connected || server.index == 0 {
            if !server.connected {
                context.log(JjcLogEvent::PlayerGameServerUnavailable { player_id });
            } else {
                context.log(JjcLogEvent::PlayerServerIdZero { player_id });
            }
            return JjcApplyReport {
                player_id,
                legacy_result: 1,
                disposition: JjcApplyDisposition::Rejected(
                    JjcCanApplyDisposition::PlayerOrGameServerUnavailable,
                ),
            };
        }

        let region = self.acquire_region(game, config, context);
        let region_id = region.region_id();
        if region_id == 0 {
            return JjcApplyReport {
                player_id,
                legacy_result: 4,
                disposition: JjcApplyDisposition::RegionUnavailable(region),
            };
        }
        self.queue.insert(player_id, info);
        let (opponent_id, removed_unroutable_players) =
            self.get_one_opponent(game, player_id, config, context);
        if opponent_id == 0 {
            let recycle = self.recycle_jjc_region(config, context, region_id);
            return JjcApplyReport {
                player_id,
                legacy_result: 3,
                disposition: JjcApplyDisposition::WaitingForOpponent { region, recycle },
            };
        }

        self.queue.entry(opponent_id).or_default().opponent_id = player_id;
        self.queue.entry(opponent_id).or_default().jjc_region_id = region_id;
        self.queue.entry(player_id).or_default().opponent_id = opponent_id;
        self.queue.entry(player_id).or_default().jjc_region_id = region_id;
        let fight = self.start_fight(
            game,
            context,
            region_id,
            player_id,
            opponent_id,
        );
        context.log(JjcLogEvent::ApplyMatched {
            player_id,
            region_id,
            opponent_id,
        });
        JjcApplyReport {
            player_id,
            legacy_result: 0,
            disposition: JjcApplyDisposition::Matched {
                region,
                opponent_id,
                removed_unroutable_players,
                fight,
            },
        }
    }

    pub fn end_pk<Context: JjcRunContext + ?Sized>(
        &mut self,
        config: JjcRunConfig,
        context: &mut Context,
        region_id: i32,
        player_id: i32,
    ) -> JjcEndPkReport {
        let player = *self.queue.entry(player_id).or_default();
        let opponent_id = player.opponent_id;
        if player_id == 0 || opponent_id == 0 {
            let removed = self.queue.remove(&player_id).is_some();
            return JjcEndPkReport {
                requested_region_id: region_id,
                disposition: JjcEndPkDisposition::RemovedIncompletePlayer {
                    player_id,
                    opponent_id,
                    removed,
                },
            };
        }
        let opponent = *self.queue.entry(opponent_id).or_default();
        if opponent.opponent_id == player_id
            && player.start_time == opponent.start_time
            && player.jjc_region_id == opponent.jjc_region_id
        {
            let removed_fight = self.fighting_list.remove(&region_id).is_some();
            let removed_player = self.queue.remove(&player_id).is_some();
            let removed_opponent = self.queue.remove(&opponent_id).is_some();
            let recycle = self.recycle_jjc_region(config, context, region_id);
            context.log(JjcLogEvent::PkEnded {
                region_id,
                player_id,
            });
            return JjcEndPkReport {
                requested_region_id: region_id,
                disposition: JjcEndPkDisposition::Applied {
                    player_id,
                    opponent_id,
                    removed_fight,
                    removed_player,
                    removed_opponent,
                    recycle,
                },
            };
        }
        context.log(JjcLogEvent::PkCorrelationMismatch {
            region_id,
            player_id,
            opponent_id,
            opponent_points_to: opponent.opponent_id,
        });
        JjcEndPkReport {
            requested_region_id: region_id,
            disposition: JjcEndPkDisposition::CorrelationMismatch {
                player_id,
                opponent_id,
                opponent_points_to: opponent.opponent_id,
            },
        }
    }

    pub fn quit_player<Game: JjcGameView + ?Sized, Context: JjcRunContext + ?Sized>(
        &mut self,
        game: &Game,
        config: JjcRunConfig,
        context: &mut Context,
        player_id: i32,
        requested_region_id: i32,
    ) -> JjcQuitReport {
        let Some(info) = self.queue.get(&player_id).copied() else {
            return JjcQuitReport {
                player_id,
                requested_region_id,
                disposition: JjcQuitDisposition::PlayerMissing,
            };
        };
        let opponent_id = info.opponent_id;
        let internal_region_id = info.jjc_region_id;
        if self.fighting_list.contains_key(&internal_region_id)
            && config.region_id_min <= requested_region_id
            && requested_region_id <= config.region_id_max
        {
            let end = self.end_pk(config, context, internal_region_id, player_id);
            context.log(JjcLogEvent::QuitWhileFighting {
                player_id,
                opponent_id,
                region_id: internal_region_id,
            });
            return JjcQuitReport {
                player_id,
                requested_region_id,
                disposition: JjcQuitDisposition::EndedFight(end),
            };
        }

        let removed_player = self.queue.remove(&player_id).is_some();
        let removed_opponent = self.queue.remove(&opponent_id).is_some();
        context.log(JjcLogEvent::QuitAfterApplication {
            player_id,
            opponent_id,
            region_id: internal_region_id,
        });
        let notifications = [player_id, opponent_id].map(|recipient_id| {
            let map_id = Self::player_server_map_id(game, context, recipient_id);
            let delivery = (map_id > 0).then(|| {
                let mut message = CMessage::new(0x0008_0506);
                message.base_mut().add_long(recipient_id);
                message.base_mut().add_long(requested_region_id);
                message.send_to_map_id(game.jjc_game_server_sender().as_ref(), map_id)
            });
            JjcQuitNotification {
                player_id: recipient_id,
                requested_region_id,
                map_id,
                delivery,
            }
        });
        JjcQuitReport {
            player_id,
            requested_region_id,
            disposition: JjcQuitDisposition::RemovedApplication {
                opponent_id,
                internal_region_id,
                removed_player,
                removed_opponent,
                notifications,
            },
        }
    }

    pub fn run<Game: JjcGameView + ?Sized, Context: JjcRunContext>(
        &mut self,
        game: &Game,
        config: JjcRunConfig,
        context: &mut Context,
    ) -> Result<JjcRunReport, JjcRunBlock> {
        let current_time = context.current_time_seconds();
        if config.use_jjc == 0 {
            context.log(JjcLogEvent::Closed);
            return Ok(JjcRunReport::Closed { current_time });
        }

        let rank_elapsed = current_time.wrapping_sub(self.last_rank_refresh_time);
        let rank_refresh = if config.rank_interval_seconds < rank_elapsed {
            self.last_rank_refresh_time = current_time;
            let success = context.load_jjc_rank(&mut self.ranks);
            let item_count = i32::try_from(self.ranks.len()).map_err(|_| JjcRunBlock {
                rank_count: self.ranks.len(),
            })?;
            context.log(JjcLogEvent::RankLoaded {
                success,
                item_count,
            });
            JjcRankRefreshReport::Loaded {
                elapsed_seconds: rank_elapsed,
                success,
                item_count,
            }
        } else {
            JjcRankRefreshReport::Waiting {
                elapsed_seconds: rank_elapsed,
            }
        };

        let reset = self.reset_jjc(game, current_time, context);
        let fight_keys: Vec<i32> = self.fighting_list.keys().copied().collect();
        let mut fights = Vec::with_capacity(fight_keys.len());
        let mut skipped_after_removal = Vec::new();
        let mut fight_index = 0;
        while fight_index < fight_keys.len() {
            let region_id = fight_keys[fight_index];
            let Some(&(first_player_id, second_player_id)) = self.fighting_list.get(&region_id)
            else {
                fight_index += 1;
                continue;
            };
            let timeout = self.jjc_pk_timeout(
                game,
                config,
                context,
                JjcFightSnapshot {
                    region_id,
                    first_player_id,
                    second_player_id,
                },
                current_time,
            );
            match timeout {
                JjcPkTimeoutResult::Keep(report) => {
                    fights.push(report);
                    fight_index += 1;
                }
                JjcPkTimeoutResult::RemoveFight => {
                    self.fighting_list.remove(&region_id);
                    let recycle = self.recycle_jjc_region(config, context, region_id);
                    fights.push(JjcFightRunReport::RemovedMissingFirstPlayer {
                        region_id,
                        first_player_id,
                        second_player_id,
                        recycle,
                    });
                    if let Some(&skipped_region_id) = fight_keys.get(fight_index + 1) {
                        skipped_after_removal.push(skipped_region_id);
                    }
                    fight_index += 2;
                }
            }
        }

        Ok(JjcRunReport::Complete {
            current_time,
            rank_refresh,
            reset,
            fights,
            skipped_after_removal,
        })
    }

    fn reset_jjc<Game: JjcGameView + ?Sized, Context: JjcRunContext>(
        &mut self,
        game: &Game,
        current_time: i32,
        context: &mut Context,
    ) -> JjcResetReport {
        let local_time = context.local_time(current_time);
        let should_update = local_time.week_day == self.week_to_clear_day
            && local_time.hour == self.week_clear_hour
            && local_time.minute == self.week_clear_minute
            && !self.is_this_week_updated;

        let update = if should_update {
            self.is_this_week_updated = true;
            let local_clock = context.system_time();
            context.log(JjcLogEvent::WeekUpdateStarted {
                local_time: local_clock,
            });
            let started_at_ms = context.tick_count_ms();
            let week = self.week_update(game, current_time, context);
            if !week.database_result {
                context.log(JjcLogEvent::WeekUpdateThreadFailed);
            }

            self.passed_weeks = self.passed_weeks.wrapping_add(1);
            self.last_clear_time = current_time;
            let passed_weeks = self.passed_weeks.to_string();
            let last_clear_time = self.last_clear_time.to_string();
            let ini_writes = JjcIniWriteReport {
                passed_weeks: context.write_private_profile_string(
                    b"write",
                    b"PassedWeeks",
                    passed_weeks.as_bytes(),
                ),
                last_clear_time: context.write_private_profile_string(
                    b"write",
                    b"lastClearTime",
                    last_clear_time.as_bytes(),
                ),
            };

            let season = if self.passed_weeks > 8 && self.passed_weeks % 8 == 1 {
                context.log(JjcLogEvent::SeasonUpdateStarted);
                Some(self.season_update(game, current_time, context))
            } else {
                None
            };

            let finished_at_ms = context.tick_count_ms();
            let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
            context.log(JjcLogEvent::WeekOrSeasonUpdateFinished { elapsed_ms });
            let warning_checked_at_ms = context.tick_count_ms();
            let warning_elapsed_ms = if 10_000 < warning_checked_at_ms.wrapping_sub(started_at_ms) {
                let warning_at_ms = context.tick_count_ms();
                let elapsed_ms = warning_at_ms.wrapping_sub(started_at_ms);
                context.log(JjcLogEvent::WeekClearDatabaseSlow { elapsed_ms });
                Some(elapsed_ms)
            } else {
                None
            };

            Some(JjcWeeklyResetUpdateReport {
                local_clock,
                started_at_ms,
                week,
                passed_weeks: self.passed_weeks,
                last_clear_time: self.last_clear_time,
                ini_writes,
                season,
                finished_at_ms,
                elapsed_ms,
                warning_checked_at_ms,
                warning_elapsed_ms,
            })
        } else {
            None
        };

        let flag_reset_for_non_sunday = local_time.week_day != 0;
        if flag_reset_for_non_sunday {
            self.is_this_week_updated = false;
        }
        JjcResetReport {
            local_time,
            update,
            flag_reset_for_non_sunday,
            is_this_week_updated_after: self.is_this_week_updated,
        }
    }

    pub fn week_update<Game: JjcGameView + ?Sized, Context: JjcRunContext + ?Sized>(
        &self,
        game: &Game,
        current_time: i32,
        context: &mut Context,
    ) -> JjcUpdateBroadcastReport {
        let mut message = CMessage::new(0x0008_0509);
        message.base_mut().add_long(current_time);
        let sender = game.jjc_game_server_sender();
        let delivery = message.send_all(sender.as_ref());
        let database_result = context.start_jjc_week_clear();
        JjcUpdateBroadcastReport {
            message_type: 0x0008_0509,
            delivery,
            database_result,
        }
    }

    pub fn season_update<Game: JjcGameView + ?Sized, Context: JjcRunContext + ?Sized>(
        &self,
        game: &Game,
        current_time: i32,
        context: &mut Context,
    ) -> JjcUpdateBroadcastReport {
        let mut message = CMessage::new(0x0008_050A);
        message.base_mut().add_long(current_time);
        let sender = game.jjc_game_server_sender();
        let delivery = message.send_all(sender.as_ref());
        let database_result = context.clear_jjc_season();
        JjcUpdateBroadcastReport {
            message_type: 0x0008_050A,
            delivery,
            database_result,
        }
    }

    fn jjc_pk_timeout<Game: JjcGameView + ?Sized, Context: JjcRunContext>(
        &mut self,
        game: &Game,
        config: JjcRunConfig,
        context: &mut Context,
        fight: JjcFightSnapshot,
        current_time: i32,
    ) -> JjcPkTimeoutResult {
        let JjcFightSnapshot {
            region_id,
            first_player_id,
            second_player_id,
        } = fight;
        if region_id < 1 {
            return JjcPkTimeoutResult::Keep(JjcFightRunReport::NonPositiveRegion {
                region_id,
                first_player_id,
                second_player_id,
            });
        }

        let Some(start_time) = self.queue.get(&first_player_id).map(|info| info.start_time) else {
            return JjcPkTimeoutResult::RemoveFight;
        };
        let elapsed_seconds = current_time.wrapping_sub(start_time);
        if start_time <= 0 || config.pk_timeout_seconds >= elapsed_seconds {
            return JjcPkTimeoutResult::Keep(JjcFightRunReport::Waiting {
                region_id,
                first_player_id,
                second_player_id,
                start_time,
                elapsed_seconds,
            });
        }

        self.queue.entry(first_player_id).or_default().start_time = 0;
        self.queue.entry(second_player_id).or_default().start_time = 0;
        let mut message = CMessage::new(0x0008_0505);
        message.base_mut().add_long(region_id);
        message.base_mut().add_long(first_player_id);
        message.base_mut().add_long(second_player_id);
        let lookup = Self::get_region_server_id(game, context, region_id);
        let sender = game.jjc_game_server_sender();
        let delivery = message.send_to_map_id(sender.as_ref(), lookup.map_id());
        context.log(JjcLogEvent::FightTimedOut {
            region_id,
            first_player_id,
            second_player_id,
        });
        JjcPkTimeoutResult::Keep(JjcFightRunReport::TimedOut {
            region_id,
            first_player_id,
            second_player_id,
            lookup,
            delivery,
        })
    }

    fn get_region_server_id<Game: JjcGameView + ?Sized, Context: JjcRunContext + ?Sized>(
        game: &Game,
        context: &mut Context,
        region_id: i32,
    ) -> JjcRegionServerLookup {
        if !game.jjc_region_exists(region_id) {
            context.log(JjcLogEvent::RegionNotConfigured { region_id });
            return JjcRegionServerLookup::RegionNotConfigured;
        }
        let Some(game_server) = game.jjc_region_game_server(region_id) else {
            context.log(JjcLogEvent::RegionNotConnected { region_id });
            return JjcRegionServerLookup::GameServerNotConnected;
        };
        if !game_server.connected {
            context.log(JjcLogEvent::RegionNotConnected { region_id });
            return JjcRegionServerLookup::GameServerNotConnected;
        }
        JjcRegionServerLookup::Found {
            game_server_id: game_server.index as i32,
        }
    }

    fn recycle_jjc_region<Context: JjcRunContext + ?Sized>(
        &mut self,
        config: JjcRunConfig,
        context: &mut Context,
        region_id: i32,
    ) -> JjcRegionRecycleReport {
        if region_id < config.region_id_min || config.region_id_max < region_id {
            context.log(JjcLogEvent::InvalidRegion { region_id });
            return JjcRegionRecycleReport::Invalid { region_id };
        }
        let removed_from_in_use = self.regions_in_use.remove(&region_id);
        let inserted_into_available = self.regions_left.insert(region_id);
        JjcRegionRecycleReport::Recycled {
            region_id,
            removed_from_in_use,
            inserted_into_available,
        }
    }
}

fn add_jjc_info(message: &mut CBaseMessage, info: JjcInfo) {
    message.add_ulong(info.jjc_level);
    message.add_long(info.old_region_id);
    message.add_long(info.position_x);
    message.add_long(info.position_y);
    message.add_long(info.opponent_id);
    message.add_long(info.jjc_region_id);
    message.add_long(info.start_time);
}

fn parse_jjc_regions(source: &[u8]) -> BTreeSet<i32> {
    source
        .split(|&byte| byte == b'#')
        .skip(1)
        .filter_map(|record| ascii_fields(record).next().and_then(parse_legacy_config_int))
        .collect()
}

fn parse_jjc_level_steps(source: &[u8]) -> BTreeMap<(i32, i32), i32> {
    let mut levels = BTreeMap::new();
    for record in source.split(|&byte| byte == b'#').skip(1) {
        let mut fields = ascii_fields(record);
        let Some(step) = fields.next().and_then(parse_legacy_config_int) else {
            continue;
        };
        let Some(minimum) = fields.next().and_then(parse_legacy_config_int) else {
            continue;
        };
        let Some(maximum) = fields.next().and_then(parse_legacy_config_int) else {
            continue;
        };
        levels.insert((minimum, maximum), step);
    }
    levels
}

#[derive(Default)]
struct ParsedJjcIni {
    week_day: i32,
    hour: i32,
    minute: i32,
    second: i32,
    passed_weeks: i32,
    last_clear_time: i32,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum JjcIniSection {
    None,
    Config,
    Write,
}

fn parse_jjc_ini(source: &[u8]) -> ParsedJjcIni {
    let mut values = ParsedJjcIni::default();
    let mut section = JjcIniSection::None;
    for raw_line in source.split(|&byte| byte == b'\n') {
        let line = trim_ascii(raw_line);
        if line.len() >= 2 && line[0] == b'[' && line[line.len() - 1] == b']' {
            let name = trim_ascii(&line[1..line.len() - 1]);
            section = if name.eq_ignore_ascii_case(b"config") {
                JjcIniSection::Config
            } else if name.eq_ignore_ascii_case(b"write") {
                JjcIniSection::Write
            } else {
                JjcIniSection::None
            };
            continue;
        }
        if line.first().is_none_or(|byte| matches!(byte, b';' | b'#')) {
            continue;
        }
        let Some(delimiter) = line.iter().position(|&byte| byte == b'=') else {
            continue;
        };
        let key = trim_ascii(&line[..delimiter]);
        let Some(value) = ascii_fields(trim_ascii(&line[delimiter + 1..]))
            .next()
            .and_then(parse_legacy_config_int)
        else {
            continue;
        };
        match section {
            JjcIniSection::Config if key.eq_ignore_ascii_case(b"Day") => {
                values.week_day = value;
            }
            JjcIniSection::Config if key.eq_ignore_ascii_case(b"Hour") => {
                values.hour = value;
            }
            JjcIniSection::Config if key.eq_ignore_ascii_case(b"Min") => {
                values.minute = value;
            }
            JjcIniSection::Config if key.eq_ignore_ascii_case(b"Sec") => {
                values.second = value;
            }
            JjcIniSection::Write if key.eq_ignore_ascii_case(b"PassedWeeks") => {
                values.passed_weeks = value;
            }
            JjcIniSection::Write if key.eq_ignore_ascii_case(b"lastClearTime") => {
                values.last_clear_time = value;
            }
            JjcIniSection::None | JjcIniSection::Config | JjcIniSection::Write => {}
        }
    }
    values
}

fn parse_legacy_config_int(token: &[u8]) -> Option<i32> {
    let text = std::str::from_utf8(token).ok()?;
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        return u32::from_str_radix(hex, 16).ok().map(|value| value as i32);
    }
    text.parse::<i32>()
        .ok()
        .or_else(|| text.parse::<u32>().ok().map(|value| value as i32))
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

fn ascii_fields(value: &[u8]) -> impl Iterator<Item = &[u8]> {
    value
        .split(u8::is_ascii_whitespace)
        .filter(|field| !field.is_empty())
}

enum JjcPkTimeoutResult {
    Keep(JjcFightRunReport),
    RemoveFight,
}

#[derive(Clone, Copy)]
struct JjcFightSnapshot {
    region_id: i32,
    first_player_id: i32,
    second_player_id: i32,
}

/// World-адаптер JJC-конфигурации поверх `GlobeSetupSnapshot`: собирает
/// исходный кортеж `jjc_run_config_fields` в [`JjcRunConfig`]; единственные
/// потребители — `world_main_loop` и `world_dispatch` Realm.
pub trait GlobeSetupJjcWorldConfig {
    fn jjc_run_config_world(&self) -> JjcRunConfig;
}

impl GlobeSetupJjcWorldConfig for nebokrai_shared::resources::GlobeSetupSnapshot {
    fn jjc_run_config_world(&self) -> JjcRunConfig {
        let (
            use_jjc,
            rank_interval_seconds,
            pk_timeout_seconds,
            region_id_min,
            region_id_max,
            max_regions_in_use,
        ) = self.jjc_run_config_fields();
        JjcRunConfig {
            use_jjc,
            rank_interval_seconds,
            pk_timeout_seconds,
            region_id_min,
            region_id_max,
            max_regions_in_use,
        }
    }
}
