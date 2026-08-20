//! Владелец периодического `CJJcSystem::Run` исторического WorldServer.
//!
//! Статус constructor/singleton lifetime, `WeekUpdate` RVA `0x00082190`,
//! `SeasonUpdate` RVA `0x00082210`, `ResetJJc` RVA `0x000841F0`,
//! `RecycleJJcRegion` RVA `0x00085460`, `GetRegionServerID` RVA `0x000854C0`,
//! `JJcPKTimeout` RVA `0x00085840` и `Run` RVA `0x00085A40` —
//! `IMPLEMENTED`; остальные функции ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная
//! пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
//! EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp`.
//!
//! PDB подтверждает signed `long` keys обоих `std::map`, полный layout
//! `tagJJcInfo` `0x1C`, `tagJJcRank` `0x30` и поля singleton-а до
//! `m_bIsThisWeekUpdated +0x9C`. `BTreeMap<i32, _>` сохраняет signed order;
//! два старых `hash_set<long>` заменены `HashSet<i32>`, поскольку достигнутые
//! функции наблюдают только membership/erase/insert, но не bucket-order.
//! Process-static last-rank time по `0x006BF0E4` перенесён в единственный
//! owned `CJJcSystem`; его точное начальное значение в PE равно нулю.
//!
//! `Run` снимает Unix time до `bUseJJc`, обновляет last-rank time до сырого
//! `CRsJJcSys::LoadJJcRank`, затем выполняет reset и signed ordered fight-pass.
//! DB rank/week/season, INI, clock и logging owners остаются явным
//! `JjcRunContext`: callback rank получает живой vector и может изменить его
//! даже при `false`. Сообщения `0x80509/0x8050A/0x80505` строит сам владелец и
//! отправляет через готовую World transport-границу в исходном порядке.
//!
//! В `JJcPKTimeout` отсутствие первого player удаляет fight и возвращает
//! регион, а настоящий timeout только обнуляет оба `lStartTime`, рассылает
//! `0x80505` и сохраняет fight-map. Второй player создаётся default-значением
//! через прежний `map::operator[]`. Recycle принимает только inclusive
//! `[lJJcRegionIDMin, lJJcRegionIDMax]`, независимо удаляет in-use membership
//! и вставляет available membership. Ещё одна исходная странность имеет
//! `VERIFIED_DISASSEMBLY`: exact `0x00485B16..0x00485B7D` сначала увеличивает
//! сохранённый iterator перед erase, присваивает его current, а затем выполняет
//! общий increment ещё раз. Поэтому после orphan-removal один следующий fight
//! пропускается до следующего `Run`; Rust сохраняет этот double-increment.
//!
//! `ResetJJc` намеренно не читает настроенную секунду. Более странный факт
//! подтверждён exact EXE `0x00484202..0x00484249` и
//! `0x004843B7..0x004843C5`: скопированный `tm_wday` сравнивается с
//! `m_nWeekToClearDay`, но после прохода флаг очищается по жёсткому
//! `tm_wday != 0`. Поэтому update на настроенном ненулевом дне может повторяться
//! каждый вызов в ту же минуту; эта наблюдаемая странность сохранена.
//! `_localtime`, `GetLocalTime`, `GetTickCount`, `WritePrivateProfileStringA`,
//! logging и DB остаются Linux-совместимыми callbacks без Windows FFI. STL,
//! SEH, allocator, singleton destructor и unwind noise удалены у реализованных
//! блоков; `nullptr` transport заменён готовым `Option` внутри `CGame`.

use std::collections::{BTreeMap, HashSet};

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;

/// Точный PDB-layout `tagJJcRank` без обещания NUL в 32-байтном имени.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct JjcRank {
    pub(crate) rank: i32,
    pub(crate) player_id: i32,
    pub(crate) jjc_level: u32,
    pub(crate) level: i32,
    pub(crate) name: [u8; 0x20],
}

/// Точная owned-проекция семи полей `tagJJcInfo`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct JjcInfo {
    pub(crate) jjc_level: u32,
    pub(crate) old_region_id: i32,
    pub(crate) position_x: i32,
    pub(crate) position_y: i32,
    pub(crate) opponent_id: i32,
    pub(crate) jjc_region_id: i32,
    pub(crate) start_time: i32,
}

/// Достигнутые поля `CGlobeSetup::m_stSetup`, читаемые Run-chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JjcRunConfig {
    pub(crate) use_jjc: i32,
    pub(crate) rank_interval_seconds: i32,
    pub(crate) pk_timeout_seconds: i32,
    pub(crate) region_id_min: i32,
    pub(crate) region_id_max: i32,
}

/// Девять signed полей 32-bit MSVC `tm` после немедленного copy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JjcLocalTime {
    pub(crate) second: i32,
    pub(crate) minute: i32,
    pub(crate) hour: i32,
    pub(crate) month_day: i32,
    pub(crate) month: i32,
    pub(crate) year_since_1900: i32,
    pub(crate) week_day: i32,
    pub(crate) year_day: i32,
    pub(crate) daylight_saving: i32,
}

/// Поля отдельного `GetLocalTime`, публикуемые start-log недельного update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JjcSystemTime {
    pub(crate) year: u16,
    pub(crate) month: u16,
    pub(crate) week_day: u16,
    pub(crate) day: u16,
    pub(crate) hour: u16,
    pub(crate) minute: u16,
    pub(crate) second: u16,
    pub(crate) milliseconds: u16,
}

/// Typed logging-события Run-chain в точном порядке вызовов.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum JjcLogEvent {
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
}

/// Ещё сырые DB/INI/time/log owners, достигнутые полным `CJJcSystem::Run`.
pub(crate) trait JjcRunContext {
    /// Повторяет начальный `_time(nullptr)` уже в 32-bit signed форме.
    fn current_time_seconds(&mut self) -> i32;

    /// Повторяет `_localtime` и немедленную копию всех девяти полей.
    fn local_time(&mut self, timestamp: i32) -> JjcLocalTime;

    fn system_time(&mut self) -> JjcSystemTime;
    fn tick_count_ms(&mut self) -> u32;

    /// Сырой `CRsJJcSys::LoadJJcRank`; vector остаётся живым и при `false`.
    fn load_jjc_rank(&mut self, ranks: &mut Vec<JjcRank>) -> bool;

    /// Сырой `CRsJJcSys::JJcWeekClear`, возвращающий результат thread-start.
    fn start_jjc_week_clear(&mut self) -> bool;

    /// Сырой `CRsJJcSys::JJcSeasonClear`; caller исторически игнорировал bool.
    fn clear_jjc_season(&mut self) -> bool;

    /// Повторяет одну запись в исходный `szIniFile`; return caller не читал.
    fn write_private_profile_string(&mut self, section: &[u8], key: &[u8], value: &[u8]) -> bool;

    fn log(&mut self, event: JjcLogEvent);
}

/// Safe-граница размера, невозможного в 32-bit vector исходного процесса.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JjcRunBlock {
    pub(crate) rank_count: usize,
}

/// Один выполненный broadcast и его ещё сырой DB-side effect.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct JjcUpdateBroadcastReport {
    pub(crate) message_type: u32,
    pub(crate) delivery: Result<i32, SendMessageError>,
    pub(crate) database_result: bool,
}

/// Результат двух исходных INI writes после недельного clear.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct JjcIniWriteReport {
    pub(crate) passed_weeks: bool,
    pub(crate) last_clear_time: bool,
}

/// Полная trigger-ветвь `ResetJJc`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct JjcWeeklyResetUpdateReport {
    pub(crate) local_clock: JjcSystemTime,
    pub(crate) started_at_ms: u32,
    pub(crate) week: JjcUpdateBroadcastReport,
    pub(crate) passed_weeks: i32,
    pub(crate) last_clear_time: i32,
    pub(crate) ini_writes: JjcIniWriteReport,
    pub(crate) season: Option<JjcUpdateBroadcastReport>,
    pub(crate) finished_at_ms: u32,
    pub(crate) elapsed_ms: u32,
    pub(crate) warning_checked_at_ms: u32,
    pub(crate) warning_elapsed_ms: Option<u32>,
}

/// Один вызов `ResetJJc`, включая исходный hard-coded Sunday reset.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct JjcResetReport {
    pub(crate) local_time: JjcLocalTime,
    pub(crate) update: Option<JjcWeeklyResetUpdateReport>,
    pub(crate) flag_reset_for_non_sunday: bool,
    pub(crate) is_this_week_updated_after: bool,
}

/// Результат strict rank-refresh gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JjcRankRefreshReport {
    Waiting {
        elapsed_seconds: i32,
    },
    Loaded {
        elapsed_seconds: i32,
        success: bool,
        item_count: i32,
    },
}

/// Две различимые failure-ветви `GetRegionServerID` и его успех.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JjcRegionServerLookup {
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

/// Наблюдаемая membership-мутация `RecycleJJcRegion`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JjcRegionRecycleReport {
    Recycled {
        region_id: i32,
        removed_from_in_use: bool,
        inserted_into_available: bool,
    },
    Invalid {
        region_id: i32,
    },
}

/// Один элемент signed ordered fight-pass.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum JjcFightRunReport {
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

/// Полный наблюдаемый результат одного `CJJcSystem::Run`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum JjcRunReport {
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

/// Owned замена единственного process-static `CJJcSystem`.
pub(crate) struct CJJcSystem {
    queue: BTreeMap<i32, JjcInfo>,
    fighting_list: BTreeMap<i32, (i32, i32)>,
    regions_left: HashSet<i32>,
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
    /// Создаёт точное zero-initialized состояние статического singleton-а.
    pub(crate) fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
            fighting_list: BTreeMap::new(),
            regions_left: HashSet::new(),
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

    /// Возвращает живой rank-vector, заменяющий исходные pointer/size поля.
    pub(crate) fn ranks(&self) -> &[JjcRank] {
        &self.ranks
    }

    /// Настраивает четыре INI-поля; `second` сохраняется, но Reset его не читает.
    pub(crate) fn set_week_clear_schedule(
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

    /// Восстанавливает два значения секции `write` после сырого config load.
    pub(crate) fn set_clear_state(&mut self, passed_weeks: i32, last_clear_time: i32) {
        self.passed_weeks = passed_weeks;
        self.last_clear_time = last_clear_time;
    }

    /// Выполняет полный периодический JJC owner и всегда сохраняет legacy `0`.
    pub(crate) fn run<Context: JjcRunContext>(
        &mut self,
        game: &CGame,
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

    fn reset_jjc<Context: JjcRunContext>(
        &mut self,
        game: &CGame,
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

    fn week_update<Context: JjcRunContext>(
        &self,
        game: &CGame,
        current_time: i32,
        context: &mut Context,
    ) -> JjcUpdateBroadcastReport {
        let mut message = CMessage::new(0x0008_0509);
        message.base_mut().add_long(current_time);
        let sender = game.current_game_server_sender();
        let delivery = message.send_all(sender.as_ref());
        let database_result = context.start_jjc_week_clear();
        JjcUpdateBroadcastReport {
            message_type: 0x0008_0509,
            delivery,
            database_result,
        }
    }

    fn season_update<Context: JjcRunContext>(
        &self,
        game: &CGame,
        current_time: i32,
        context: &mut Context,
    ) -> JjcUpdateBroadcastReport {
        let mut message = CMessage::new(0x0008_050A);
        message.base_mut().add_long(current_time);
        let sender = game.current_game_server_sender();
        let delivery = message.send_all(sender.as_ref());
        let database_result = context.clear_jjc_season();
        JjcUpdateBroadcastReport {
            message_type: 0x0008_050A,
            delivery,
            database_result,
        }
    }

    fn jjc_pk_timeout<Context: JjcRunContext>(
        &mut self,
        game: &CGame,
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
        let sender = game.current_game_server_sender();
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

    fn get_region_server_id<Context: JjcRunContext>(
        game: &CGame,
        context: &mut Context,
        region_id: i32,
    ) -> JjcRegionServerLookup {
        if game.region(region_id).is_none() {
            context.log(JjcLogEvent::RegionNotConfigured { region_id });
            return JjcRegionServerLookup::RegionNotConfigured;
        }
        let Some(game_server) = game.get_region_game_server(region_id) else {
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

    fn recycle_jjc_region<Context: JjcRunContext>(
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp

// ============================================================================
// FUNCTION: CJJcSystem::GetJJcLevelStep
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:390
// RVA: 0x00082620
// ADDRESS: 00482620
// PROTOTYPE: long __thiscall GetJJcLevelStep(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::IsPlayerInPK
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:404
// RVA: 0x000829E0
// ADDRESS: 004829e0
// PROTOTYPE: bool __thiscall IsPlayerInPK(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::CanApplyJJcNow
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:359
// RVA: 0x00082A30
// ADDRESS: 00482a30
// PROTOTYPE: long __thiscall CanApplyJJcNow(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::GetOneOpponent
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:308
// RVA: 0x00083760
// ADDRESS: 00483760
// PROTOTYPE: long __thiscall GetOneOpponent(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::LoadJJcConfig
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:593
// RVA: 0x00085020
// ADDRESS: 00485020
// PROTOTYPE: bool __thiscall LoadJJcConfig(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::StartFight
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:189
// RVA: 0x00085590
// ADDRESS: 00485590
// PROTOTYPE: bool __thiscall StartFight(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::EndPK
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:223
// RVA: 0x00085770
// ADDRESS: 00485770
// PROTOTYPE: void __thiscall EndPK(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::GetOneJJcRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:428
// RVA: 0x00085990
// ADDRESS: 00485990
// PROTOTYPE: long __thiscall GetOneJJcRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::ApplyPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:83
// RVA: 0x00085B90
// ADDRESS: 00485b90
// PROTOTYPE: long __thiscall ApplyPlayer(tagJJcInfo * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJJcSystem::QuitPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\jjcsystem.cpp:142
// RVA: 0x00085CA0
// ADDRESS: 00485ca0
// PROTOTYPE: void __thiscall QuitPlayer(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
