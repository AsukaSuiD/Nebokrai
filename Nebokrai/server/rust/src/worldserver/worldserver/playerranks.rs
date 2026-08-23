//! Общий рейтинг игроков исторического WorldServer.
//!
//! `UpdateRanksToGameServer`, `StatPlayerRanks`
//! `AddRank`, `OnStatRanks`
//! и `Initialize` — часть контракта owner-а. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! World serializer и Game decoder подтверждают wire: signed count и
//! insertion-order records `i32 player_id + C-string name + u16 occupation +
//! u16 level + C-string faction_name`. `Vec` заменяет `std::list`, owned bytes
//! — `std::string`; порядок и little-endian поля не меняются. Внутренний NUL
//! штатно завершает исходный C-string и поэтому обрезает только wire-поле.
//! Невозможный 32-битный count блокирует весь append до изменения destination.
//!
//! Публикация использует готовые `CMessage` и `ServerCommandHandle`:
//! `0x7F801 + subtype 0x17 + serialized ranks` сохраняется, а самописные
//! буфер, CRC-envelope и fan-out не дублируются. Исходный `SendAll` игнорировал
//! transport-result; Rust оставляет его в отчёте, не меняя порядок вызовов.
//! `StatPlayerRanks` связан в `CGame` с реальным `CRsPlayer`: список очищается
//! до start-log, tick снимается после него, DB добавляет live prefix, затем
//! всегда идут end-log и публикация даже после DB `false`. `AddRank` получает
//! organizing явно вместо singleton-а и сохраняет empty-name ветви отсутствия
//! faction; исходный null внутри map остаётся typed-блоком.
//!
//! `Initialize` копирует время и signed maximum из `COrganizingParam`, заменяет
//! в календарной копии только текущие year/month/day и переносит событие на
//! сутки лишь при strict `< now`. `OnStatRanks` после stat/publication снова
//! берёт исходное время, подставляет текущую дату и уже безусловно добавляет
//! сутки. Эти три присваивания исправляют неоднозначность исходного listing-а и подтверждены
//! инструкциями и.
//! `Option<TimerId>` заменяет неинициализированный constructor-ом event ID;
//! сам `CTimer` остаётся библиотечным ordered owner-ом.
//! `Default` повторяет подтверждённую inline-инициализацию singleton-а, а
//! явная передача единственного `CPlayerRanks` заменяет process-global
//! `getInstance`/`GetPlayerRanks`. `Vec` освобождается обычным Rust `Drop`;
//! уничтожает owner и обнуляет singleton после
//! `Release`, что Rust выражает обычным владением. Сам `Release` сначала
//! снимает назначенное calendar-событие, затем очищает ranks. Constructor-ная
//! дыра event ID не
//! превращается в случайный `KillTimeEvent`: `None` исправляет внутреннее
//! чтение неинициализированного DWORD, которое не является контрактом игры.
//! Два входных поля теперь приходят из действующего `COrganizingParam`;
//! PlayerRanks по-прежнему принимает узкую typed-проекцию и не дублирует его
//! позиционный parser.

use std::error::Error;
use std::fmt;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::servers::ServerCommandHandle;
use crate::public::date::{TagTime, TagTimeArithmeticBlock};
use crate::public::timer::{CTimer, TimerId};
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    COrganizingCtrl, FreePlayerLookup,
};

/// Owned запись исторического `CPlayerRanks::tagRank`.
///
/// PDB приписывает copy-constructor и destructor этой записи
/// `savedb.cpp`, но оба тела не содержат DB-эффектов: copy последовательно
/// переносит ID, два C++ string и два `u16`, а destructor освобождает только
/// string storage. `Clone` и обычный Rust `Drop` сохраняют все пять данных и
/// устраняют только MSVC/heap lifetime plumbing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerRankEntry {
    pub(crate) player_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) occupation: u16,
    pub(crate) level: u16,
    pub(crate) faction_name: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CPlayerRanks {
    stat: bool,
    stat_time: TagTime,
    stat_event_id: Option<TimerId>,
    maximum_count: Option<i32>,
    ranks: Vec<PlayerRankEntry>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PlayerRanksInitializationConfig {
    pub(crate) stat_time: TagTime,
    pub(crate) maximum_count: i32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PlayerRanksInitializationReport {
    pub(crate) scheduled_time: TagTime,
    pub(crate) event_id: TimerId,
    pub(crate) legacy_result: bool,
}

/// Наблюдаемый prefix `Release` до уничтожения Rust owner-а контекстом.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerRanksReleaseReport {
    pub(crate) previous_event_id: Option<TimerId>,
    pub(crate) timer_event_removed: Option<bool>,
    pub(crate) cleared_ranks: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerRanksScheduleBlock {
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PlayerRanksGameServerUpdate {
    pub(crate) rank_count: usize,
    pub(crate) payload_length: usize,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

impl CPlayerRanks {
 /// Копирует параметры и ставит первое календарное событие owner-а.
    pub(crate) fn initialize<Callback: Copy>(
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

 /// Готовит следующее событие после синхронного `OnStatRanks` callback-а.
    pub(crate) fn next_stat_time(
        &self,
        current_time: TagTime,
    ) -> Result<TagTime, PlayerRanksScheduleBlock> {
        let mut scheduled_time = self.stat_time_for_current_date(current_time);
        scheduled_time
            .add_day(1)
            .map_err(PlayerRanksScheduleBlock::DateArithmetic)?;
        Ok(scheduled_time)
    }

 /// Фиксирует side effects после `SetTimeEvent` следующего дня.
    pub(crate) fn finish_stat_schedule(&mut self, event_id: TimerId) {
        self.stat_event_id = Some(event_id);
        self.stat = true;
    }

    pub(crate) const fn stat_enabled(&self) -> bool {
        self.stat
    }

    pub(crate) const fn stat_time(&self) -> TagTime {
        self.stat_time
    }

    pub(crate) const fn stat_event_id(&self) -> Option<TimerId> {
        self.stat_event_id
    }

    pub(crate) const fn maximum_count(&self) -> Option<i32> {
        self.maximum_count
    }

    pub(crate) fn set_maximum_count(&mut self, maximum_count: i32) {
        self.maximum_count = Some(maximum_count);
    }

    pub(crate) fn clear(&mut self) {
        self.ranks.clear();
    }

 /// Снимает calendar callback до очистки list-owner-а, как `Release`.
    pub(crate) fn release<Callback>(
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

    pub(crate) fn push(&mut self, rank: PlayerRankEntry) {
        self.ranks.push(rank);
    }

 /// Добавляет строку DB и вычисляет faction-name по live organizing map.
    pub(crate) fn add_rank(
        &mut self,
        organizing: &COrganizingCtrl,
        player_id: i32,
        name: Vec<u8>,
        occupation: u16,
        level: u16,
    ) -> Result<(), PlayerRankAddBlock> {
        let faction_name = match organizing.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => Vec::new(),
            FreePlayerLookup::Faction(faction_id) => organizing
                .faction_by_id(faction_id)
                .map_or_else(Vec::new, |faction| faction.name().to_vec()),
            FreePlayerLookup::BlockedNullFaction { map_key } => {
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

    pub(crate) fn ranks(&self) -> &[PlayerRankEntry] {
        &self.ranks
    }

    fn stat_time_for_current_date(&self, current_time: TagTime) -> TagTime {
        let mut scheduled_time = self.stat_time;
        scheduled_time.year = current_time.year;
        scheduled_time.month = current_time.month;
        scheduled_time.day = current_time.day;
        scheduled_time
    }

    pub(crate) fn add_to_byte_array(
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

 /// Публикует `0x7F801/0x17` всем подключённым GameServer-ам.
    pub(crate) fn update_ranks_to_game_server(
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
pub(crate) enum PlayerRanksSerializationBlock {
    CountOutOfRange {
        count: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerRankAddBlock {
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
