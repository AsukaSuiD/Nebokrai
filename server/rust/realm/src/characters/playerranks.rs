//! Рейтинг `CPlayerRanks` из WorldServer, перенесённый в Realm `characters/`,
//! подтверждённый `worldserver.exe` и `worldserver.pdb`.
//!
//! Wire subtype `0x17` содержит signed count и insertion-order player records.
//! Статистика очищает список до start-log; DB добавляет доступный префикс, после
//! чего end-log и публикация выполняются даже при DB `false`. Transport-result
//! исходный `SendAll` игнорировал.
//!
//! Calendar event подставляет текущую дату и переносится на сутки только при
//! strict `< now`; callback после публикации добавляет сутки безусловно.
//! Неназначенный timer ID хранится как `None`, поэтому `Release` снимает только
//! реально зарегистрированное событие.
//!
//! Lookup фракции для `add_rank` сужен до двухметодного трейта
//! `PlayerRankOrganizingLookup`; impl у владельца организаций (`COrganizingCtrl`).

use std::error::Error;
use std::fmt;

use crate::app::world_message::{CMessage, SendMessageError};
use nebokrai_shared::network::ServerCommandHandle;
use nebokrai_shared::runtime::{CTimer, TimerId};
use nebokrai_shared::values::{TagTime, TagTimeArithmeticBlock};

/// Lookup фракции для одной записи rank-листа; точный снимок ответа
/// `COrganizingCtrl::is_free_player`, включая заблокированный null-pointer
/// вариант с исходным map key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayerRankFactionLookup {
    NoFaction,
    Faction(i32),
    NullFaction { map_key: i32 },
}

/// Узкая точка чтения фракции, нужная `CPlayerRanks::add_rank`.
/// Реализация живёт на realm `COrganizingCtrl` (`organizations/organizingctrl`)
/// и делегирует его inherent-методам.
pub trait PlayerRankOrganizingLookup {
    fn faction_lookup_of_player(&self, player_id: i32) -> PlayerRankFactionLookup;

    fn faction_name_of_id(&self, faction_id: i32) -> Option<Vec<u8>>;
}

/// Owned запись исторического `CPlayerRanks::tagRank`.
///
/// PDB приписывает copy-constructor и destructor этой записи
/// `savedb.cpp`, но оба тела не содержат DB-эффектов: copy последовательно
/// переносит ID, два C++ string и два `u16`, а destructor освобождает только
/// string storage. `Clone` и обычный Rust `Drop` сохраняют все пять данных и
/// устраняют только MSVC/heap lifetime plumbing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlayerRankEntry {
    pub player_id: i32,
    pub name: Vec<u8>,
    pub occupation: u16,
    pub level: u16,
    pub faction_name: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct CPlayerRanks {
    stat: bool,
    stat_time: TagTime,
    stat_event_id: Option<TimerId>,
    maximum_count: Option<i32>,
    ranks: Vec<PlayerRankEntry>,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerRanksInitializationConfig {
    pub stat_time: TagTime,
    pub maximum_count: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerRanksInitializationReport {
    pub scheduled_time: TagTime,
    pub event_id: TimerId,
    pub legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerRanksReleaseReport {
    pub previous_event_id: Option<TimerId>,
    pub timer_event_removed: Option<bool>,
    pub cleared_ranks: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerRanksScheduleBlock {
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub struct PlayerRanksGameServerUpdate {
    pub rank_count: usize,
    pub payload_length: usize,
    pub delivery: Result<i32, SendMessageError>,
}

impl CPlayerRanks {
    pub fn initialize<Callback: Copy>(
        &mut self,
        configuration: PlayerRanksInitializationConfig,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        callback: Callback,
    ) -> Result<PlayerRanksInitializationReport, PlayerRanksScheduleBlock> {
        self.maximum_count = Some(configuration.maximum_count);
        self.stat_time = configuration.stat_time;

        let mut scheduled_time = self.stat_time_for_current_date(current_time);
        if scheduled_time.legacy_lt(current_time) {
            scheduled_time
                .add_day(1)
                .map_err(PlayerRanksScheduleBlock::DateArithmetic)?;
        }

        self.stat = false;
        let event_id = timer.set_time_event(scheduled_time, callback, 0);
        self.stat_event_id = Some(event_id);
        Ok(PlayerRanksInitializationReport {
            scheduled_time,
            event_id,
            legacy_result: true,
        })
    }

    pub fn next_stat_time(
        &self,
        current_time: TagTime,
    ) -> Result<TagTime, PlayerRanksScheduleBlock> {
        let mut scheduled_time = self.stat_time_for_current_date(current_time);
        scheduled_time
            .add_day(1)
            .map_err(PlayerRanksScheduleBlock::DateArithmetic)?;
        Ok(scheduled_time)
    }

    pub fn finish_stat_schedule(&mut self, event_id: TimerId) {
        self.stat_event_id = Some(event_id);
        self.stat = true;
    }

    pub const fn stat_enabled(&self) -> bool {
        self.stat
    }

    pub const fn stat_time(&self) -> TagTime {
        self.stat_time
    }

    pub const fn stat_event_id(&self) -> Option<TimerId> {
        self.stat_event_id
    }

    pub const fn maximum_count(&self) -> Option<i32> {
        self.maximum_count
    }

    pub fn set_maximum_count(&mut self, maximum_count: i32) {
        self.maximum_count = Some(maximum_count);
    }

    pub fn clear(&mut self) {
        self.ranks.clear();
    }

    pub fn release<Callback>(
        &mut self,
        timer: &mut CTimer<Callback>,
    ) -> PlayerRanksReleaseReport {
        let previous_event_id = self.stat_event_id.take();
        let timer_event_removed = previous_event_id.map(|id| timer.kill_time_event(id));
        let cleared_ranks = self.ranks.len();
        self.ranks.clear();
        PlayerRanksReleaseReport {
            previous_event_id,
            timer_event_removed,
            cleared_ranks,
        }
    }

    pub fn push(&mut self, rank: PlayerRankEntry) {
        self.ranks.push(rank);
    }

    pub fn add_rank(
        &mut self,
        organizing: &impl PlayerRankOrganizingLookup,
        player_id: i32,
        name: Vec<u8>,
        occupation: u16,
        level: u16,
    ) -> Result<(), PlayerRankAddBlock> {
        let faction_name = match organizing.faction_lookup_of_player(player_id) {
            PlayerRankFactionLookup::NoFaction => Vec::new(),
            PlayerRankFactionLookup::Faction(faction_id) => organizing
                .faction_name_of_id(faction_id)
                .unwrap_or_default(),
            PlayerRankFactionLookup::NullFaction { map_key } => {
                return Err(PlayerRankAddBlock::NullFaction { map_key });
            }
        };
        self.ranks.push(PlayerRankEntry {
            player_id,
            name,
            occupation,
            level,
            faction_name,
        });
        Ok(())
    }

    pub fn ranks(&self) -> &[PlayerRankEntry] {
        &self.ranks
    }

    fn stat_time_for_current_date(&self, current_time: TagTime) -> TagTime {
        let mut scheduled_time = self.stat_time;
        scheduled_time.year = current_time.year;
        scheduled_time.month = current_time.month;
        scheduled_time.day = current_time.day;
        scheduled_time
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PlayerRanksSerializationBlock> {
        let count = i32::try_from(self.ranks.len()).map_err(|_| {
            PlayerRanksSerializationBlock::CountOutOfRange {
                count: self.ranks.len(),
            }
        })?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&count.to_le_bytes());
        for rank in &self.ranks {
            payload.extend_from_slice(&rank.player_id.to_le_bytes());
            write_player_rank_string(&mut payload, &rank.name);
            payload.extend_from_slice(&rank.occupation.to_le_bytes());
            payload.extend_from_slice(&rank.level.to_le_bytes());
            write_player_rank_string(&mut payload, &rank.faction_name);
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }

    pub fn update_ranks_to_game_server(
        &self,
        sender: Option<&ServerCommandHandle>,
    ) -> Result<PlayerRanksGameServerUpdate, PlayerRanksSerializationBlock> {
        let mut payload = Vec::new();
        self.add_to_byte_array(&mut payload)?;

        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(0x17);
        message.base_mut().add(&payload);
        Ok(PlayerRanksGameServerUpdate {
            rank_count: self.ranks.len(),
            payload_length: payload.len(),
            delivery: message.send_all(sender),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayerRanksSerializationBlock {
    CountOutOfRange {
        count: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerRankAddBlock {
    NullFaction { map_key: i32 },
}

impl fmt::Display for PlayerRanksSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { count } => write!(
                formatter,
                "PlayerRanks содержит {count} записей вне signed 32-битного диапазона"
            ),
        }
    }
}

impl Error for PlayerRanksSerializationBlock {}

fn write_player_rank_string(destination: &mut Vec<u8>, value: &[u8]) {
    let prefix = value.split(|byte| *byte == 0).next().unwrap_or_default();
    destination.extend_from_slice(prefix);
    destination.push(0);
}
