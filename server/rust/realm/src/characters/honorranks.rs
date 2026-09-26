//! Почётные ранги `CHonorRanks` из WorldServer,
//! подтверждённые `worldserver.exe` и `worldserver.pdb`.
//!
//! Live и DB snapshots содержат history/current для day, week, month и total по
//! четырём странам. Generator полностью заменяет каждую DB-копию, не меняя live
//! списки, и сохраняет полный local `tagTime`.
//!
//! Initial-config и updates пишут ordered records; total subtype дополнительно
//! несёт исходную mask. Rollover копирует current в history, очищает day/week/
//! month, но сохраняет накопительный total, затем ставит локальное событие и
//! рассылает GameServer в исходном порядке.
//!
//! `PushToRanks` обновляет без смены имени либо добавляет запись, stable-сортирует
//! по eliminate и level и обрезает список до десяти. Padding новых записей
//! обнулён. Один lifecycle-owned `CHonorRanks` заменяет process singleton.
//!
//! Загрузка из World DB, игровой hub и данные игрока сужены до трёх узких
//! трейтов-швов `HonorRanksDbOwner`, `HonorRanksGameView` и
//! `HonorRankPlayerView`; реализации живут у `TiberiusRsPlayer`
//! (`persistence/rsplayer`), `CGame` (`app/world_game`) и `CPlayer` и
//! делегируют их inherent-методам.

use std::error::Error;
use std::fmt;

use chrono::{Datelike, Local, Timelike};

use crate::app::world_message::{CMessage, SendMessageError};
use crate::characters::honordb::{
    HonorRankDbEntry, HonorRankDbLists, HonorRanksCopyTimeSnapshot, HonorRanksDbDataSnapshot,
    HonorRanksLoadOutcome, HonorRanksLoadSink, HonorRanksSavePeriod, HonorRanksType,
};
use crate::persistence::rssetup::WorldTdsClient;
use nebokrai_shared::network::ServerCommandHandle;

#[derive(Debug, Eq, PartialEq)]
pub struct HonorRanksGameServerUpdate {
    pub rank_type: HonorRanksType,
    pub subtype: i32,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HonorRanksNewDayReport {
    pub sort_day: u32,
    pub rank_mask: u32,
    pub reset_message_queued: bool,
    pub game_server_updates: Vec<HonorRanksGameServerUpdate>,
    pub legacy_result: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum HonorRanksNewDayBlock {
    LocalQueue {
        sort_day: u32,
        rank_mask: u32,
        source: HonorRanksLocalQueueBlock,
    },
    Serialization {
        sort_day: u32,
        rank_mask: u32,
        source: HonorRanksSerializationBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct HonorRankPushReport {
    pub rank_type: HonorRanksType,
    pub country: u8,
    pub player_id: i32,
    pub replaced_existing: bool,
    pub retained_in_top_ten: bool,
    pub removed_player_ids: Vec<i32>,
    pub legacy_result: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HonorRanksKilledPlayerReport {
    pub country: u8,
    pub updates: Vec<HonorRankPushReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HonorRankPushBlock {
    InvalidCountry {
        country: u8,
    },
    NameTooLong {
        player_id: i32,
        visible_name_length: usize,
    },
}

/// Информация об отклонённой постановке локального world-сообщения,
/// эквивалент `WorldLocalMessageQueueBlock` владельца игры: тип сообщения.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HonorRanksLocalQueueBlock {
    pub message_type: i32,
}

/// Узкая точка доступа к игровому hub-у, нужная rollover honor ranks:
/// локальная FIFO world-сообщений и sender текущего GameServer.
/// Реализация живёт у владельца игры (старый `CGame`).
pub trait HonorRanksGameView {
    fn queue_honor_ranks_world_message(
        &self,
        message: CMessage,
    ) -> Result<(), HonorRanksLocalQueueBlock>;

    fn honor_ranks_game_server_sender(&self) -> Option<ServerCommandHandle>;
}

/// Узкая точка чтения данных игрока, нужная `push_to_ranks`. Имена совпадают
/// с inherent-методами владельца: impl на старой стороне резолвит inherent по
/// приоритету, рекурсии нет.
pub trait HonorRankPlayerView {
    fn get_id(&self) -> i32;
    fn get_level(&self) -> u8;
    fn get_occupation(&self) -> u8;
    fn get_appellation_id(&self) -> u32;
    fn get_name(&self) -> &[u8];
    fn country(&self) -> Option<u8>;
}

/// Узкая точка загрузки honor ranks из World DB. Реализация живёт у
/// DB-owner-а игрока (старый `RsPlayerOwner`) и делегирует его методу;
/// sink-параметр generic, как у исходного owner-трейта.
pub trait HonorRanksDbOwner {
    fn load_honor_ranks<S: HonorRanksLoadSink>(
        &mut self,
        sink: &mut S,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = HonorRanksLoadOutcome>;
}

pub struct CHonorRanks {
    history: HonorRankDbLists,
    current: HonorRankDbLists,
    db_data: Option<HonorRanksDbDataSnapshot>,
    sort_day: u32,
}

impl Default for CHonorRanks {
    fn default() -> Self {
        Self::with_reached_process_state(Local::now().day())
    }
}

impl CHonorRanks {
 /// Создаёт process-static состояние после успешного original `getInstance`.
 ///
 /// `sort_day` — точный `SYSTEMTIME.wDay`, снятый lifecycle-owner-ом в
 /// момент создания; clock API не является частью состояния рангов.
    pub fn with_reached_process_state(sort_day: u32) -> Self {
        Self {
            history: Default::default(),
            current: Default::default(),
            db_data: None,
            sort_day,
        }
    }

    pub fn with_reached_save_state() -> Self {
        Self::default()
    }

    pub const fn sort_day(&self) -> u32 {
        self.sort_day
    }

    pub async fn load_honor_ranks<R: HonorRanksDbOwner>(
        &mut self,
        database: &mut R,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> HonorRanksLoadOutcome {
        database
            .load_honor_ranks(self, active_transaction)
            .await
    }

    pub fn generate_save_data(&mut self) {
        let history = self.history.clone();
        let current = self.current.clone();
        let now = Local::now();
        let copy_time = HonorRanksCopyTimeSnapshot::from_legacy_fields([
            u16::try_from(now.year()).expect("год SYSTEMTIME помещается в u16"),
            u16::try_from(now.month()).expect("месяц помещается в u16"),
            u16::try_from(now.weekday().num_days_from_sunday())
                .expect("день недели помещается в u16"),
            u16::try_from(now.day()).expect("день помещается в u16"),
            u16::try_from(now.hour()).expect("час помещается в u16"),
            u16::try_from(now.minute()).expect("минута помещается в u16"),
            u16::try_from(now.second()).expect("секунда помещается в u16"),
            u16::try_from(now.timestamp_subsec_millis()).expect("миллисекунды помещаются в u16"),
        ])
        .expect("chrono::Local всегда возвращает календарно валидный SYSTEMTIME");

        self.db_data = Some(HonorRanksDbDataSnapshot::from_legacy_copy(
            copy_time, history, current,
        ));
    }

    pub fn db_data_mut(&mut self) -> Option<&mut HonorRanksDbDataSnapshot> {
        self.db_data.as_mut()
    }

 /// Передаёт только уже сформированную DB-копию фоновому save-owner-у.
 /// Live history/current списки остаются у MainLoop и продолжают принимать
 /// новые результаты, пока прежний snapshot сохраняется в отдельном потоке.
    pub fn take_save_owner(&mut self) -> Self {
        Self {
            history: Default::default(),
            current: Default::default(),
            db_data: self.db_data.take(),
            sort_day: self.sort_day,
        }
    }

    pub fn history_mut(
        &mut self,
        rank_type: HonorRanksType,
        country: u8,
    ) -> Option<&mut Vec<HonorRankDbEntry>> {
        self.history[rank_type as usize].get_mut(usize::from(country))
    }

    pub fn current_mut(
        &mut self,
        rank_type: HonorRanksType,
        country: u8,
    ) -> Option<&mut Vec<HonorRankDbEntry>> {
        self.current[rank_type as usize].get_mut(usize::from(country))
    }

    pub fn clear_history_honor_ranks(&mut self) {
        for by_country in &mut self.history {
            for ranks in by_country {
                ranks.clear();
            }
        }
    }

    pub fn clear_current_honor_ranks(&mut self) {
        for by_country in &mut self.current {
            for ranks in by_country {
                ranks.clear();
            }
        }
    }

    pub fn copy_honor_ranks(&mut self, rank_mask: u32) -> bool {
        for country in 0..4 {
            if rank_mask & 0x01 != 0 {
                self.history[HonorRanksType::Day as usize][country] =
                    self.current[HonorRanksType::Day as usize][country].clone();
                self.current[HonorRanksType::Day as usize][country].clear();
            }
            if rank_mask & 0x02 != 0 {
                self.history[HonorRanksType::Week as usize][country] =
                    self.current[HonorRanksType::Week as usize][country].clone();
                self.current[HonorRanksType::Week as usize][country].clear();
            }
            if rank_mask & 0x04 != 0 {
                self.history[HonorRanksType::Month as usize][country] =
                    self.current[HonorRanksType::Month as usize][country].clone();
                self.current[HonorRanksType::Month as usize][country].clear();
            }
            if rank_mask & 0x08 != 0 {
                self.history[HonorRanksType::Total as usize][country] =
                    self.current[HonorRanksType::Total as usize][country].clone();
            }
        }
        true
    }

    pub fn update_ranks_on_world_server(
        game: &impl HonorRanksGameView,
        rank_mask: u32,
    ) -> Result<(), HonorRanksLocalQueueBlock> {
        let mut message = CMessage::new(0x0005_FD0C);
        message.base_mut().add_ulong(rank_mask);
        game.queue_honor_ranks_world_message(message)
    }

    pub fn on_new_day(
        &mut self,
        game: &impl HonorRanksGameView,
        _first_load: bool,
    ) -> Result<HonorRanksNewDayReport, HonorRanksNewDayBlock> {
        let now = Local::now();
        let sort_day = now.day();
        self.sort_day = sort_day;

        let mut rank_mask = 0x09;
        if sort_day == 1 {
            rank_mask = 0x0D;
        }
        if now.weekday().num_days_from_sunday() == 1 {
            rank_mask |= 0x02;
        }

        self.copy_honor_ranks(rank_mask);
        Self::update_ranks_on_world_server(game, rank_mask).map_err(|source| {
            HonorRanksNewDayBlock::LocalQueue {
                sort_day,
                rank_mask,
                source,
            }
        })?;
        let game_server_updates = self
            .update_ranks_on_game_server(game, rank_mask)
            .map_err(|source| HonorRanksNewDayBlock::Serialization {
                sort_day,
                rank_mask,
                source,
            })?;

        Ok(HonorRanksNewDayReport {
            sort_day,
            rank_mask,
            reset_message_queued: true,
            game_server_updates,
            legacy_result: true,
        })
    }

    pub fn push_to_ranks(
        &mut self,
        rank_type: HonorRanksType,
        country: u8,
        eliminate_num: u32,
        player: &impl HonorRankPlayerView,
    ) -> Result<HonorRankPushReport, HonorRankPushBlock> {
        if 4 <= country {
            return Err(HonorRankPushBlock::InvalidCountry { country });
        }
        let ranks = &mut self.current[rank_type as usize][usize::from(country)];
        let player_id = player.get_id();
        let replaced_existing = if let Some(rank) = ranks
            .iter_mut()
            .find(|rank| rank.player_id == player_id)
        {
            rank.level = player.get_level();
            rank.occupation_id = player.get_occupation();
            rank.appellation_id = player.get_appellation_id();
            rank.eliminate_num = eliminate_num;
            true
        } else {
            let source_name = player.get_name();
            let visible_name_length = source_name
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(source_name.len());
            if 20 <= visible_name_length {
                return Err(HonorRankPushBlock::NameTooLong {
                    player_id,
                    visible_name_length,
                });
            }
            let mut name = [0; 20];
            name[..visible_name_length]
                .copy_from_slice(&source_name[..visible_name_length]);
            ranks.push(HonorRankDbEntry {
                player_id,
                level: player.get_level(),
                name,
                occupation_id: player.get_occupation(),
                legacy_padding: [0; 2],
                appellation_id: player.get_appellation_id(),
                eliminate_num,
            });
            false
        };

        ranks.sort_by(|left, right| {
            right
                .eliminate_num
                .cmp(&left.eliminate_num)
                .then_with(|| right.level.cmp(&left.level))
        });
        let mut removed_player_ids = Vec::new();
        while 10 < ranks.len() {
            removed_player_ids.push(
                ranks
                    .pop()
                    .expect("длина honor ranks проверена перед удалением")
                    .player_id,
            );
        }
        let retained_in_top_ten = ranks.iter().any(|rank| rank.player_id == player_id);

        Ok(HonorRankPushReport {
            rank_type,
            country,
            player_id,
            replaced_existing,
            retained_in_top_ten,
            removed_player_ids,
            legacy_result: true,
        })
    }

    pub fn killed_one_player(
        &mut self,
        player: &impl HonorRankPlayerView,
        eliminate_counts: [u32; 4],
    ) -> Result<Option<HonorRanksKilledPlayerReport>, HonorRankPushBlock> {
        let Some(country) = player.country() else {
            return Ok(None);
        };
        let Some(country_index) = country.checked_sub(1).filter(|country| *country < 4) else {
            return Ok(None);
        };

        let mut updates = Vec::with_capacity(4);
        for (rank_type, eliminate_num) in [
            HonorRanksType::Day,
            HonorRanksType::Week,
            HonorRanksType::Month,
            HonorRanksType::Total,
        ]
        .into_iter()
        .zip(eliminate_counts)
        {
            updates.push(self.push_to_ranks(
                rank_type,
                country_index,
                eliminate_num,
                player,
            )?);
        }
        Ok(Some(HonorRanksKilledPlayerReport {
            country: country_index,
            updates,
        }))
    }

    pub fn update_ranks_on_game_server(
        &self,
        game: &impl HonorRanksGameView,
        rank_mask: u32,
    ) -> Result<Vec<HonorRanksGameServerUpdate>, HonorRanksSerializationBlock> {
        let mut updates = Vec::new();
        let sender = game.honor_ranks_game_server_sender();
        for (rank_type, bit, subtype) in [
            (HonorRanksType::Day, 0x01, 0x27),
            (HonorRanksType::Week, 0x02, 0x28),
            (HonorRanksType::Month, 0x04, 0x29),
            (HonorRanksType::Total, 0x08, 0x2A),
        ] {
            if rank_mask & bit == 0 {
                continue;
            }

            let mut payload = Vec::new();
            self.add_history_to_byte_array(&mut payload, rank_type, None)?;
            let mut message = CMessage::new(0x0007_F801);
            message.base_mut().add_long(subtype);
            if rank_type == HonorRanksType::Total {
                message.base_mut().add_ulong(rank_mask);
            }
            message.base_mut().add(&payload);
            updates.push(HonorRanksGameServerUpdate {
                rank_type,
                subtype,
                delivery: message.send_all(sender.as_ref()),
            });
        }
        Ok(updates)
    }

    pub fn add_history_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        rank_type: HonorRanksType,
        country: Option<u8>,
    ) -> Result<(), HonorRanksSerializationBlock> {
        let countries = match country {
            Some(country) if country < 4 => usize::from(country)..usize::from(country) + 1,
            Some(country) => {
                return Err(HonorRanksSerializationBlock::InvalidCountry { country });
            }
            None => 0..4,
        };
        let mut payload = Vec::new();
        for country_index in countries {
            let entries = &self.history[rank_type as usize][country_index];
            let count = i32::try_from(entries.len()).map_err(|_| {
                HonorRanksSerializationBlock::CountOutOfRange {
                    rank_type,
                    country: country_index as u8,
                    count: entries.len(),
                }
            })?;
            payload.extend_from_slice(&count.to_le_bytes());
            for (record_index, entry) in entries.iter().enumerate() {
                let name_end = entry.name.iter().position(|byte| *byte == 0).ok_or(
                    HonorRanksSerializationBlock::NameWithoutTerminator {
                        rank_type,
                        country: country_index as u8,
                        record_index,
                    },
                )?;
                payload.extend_from_slice(&entry.player_id.to_le_bytes());
                payload.push(entry.level);
                payload.extend_from_slice(&entry.name[..=name_end]);
                payload.push(entry.occupation_id);
                payload.extend_from_slice(&entry.appellation_id.to_le_bytes());
                payload.extend_from_slice(&entry.eliminate_num.to_le_bytes());
            }
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HonorRanksSerializationBlock {
    InvalidCountry {
        country: u8,
    },
    CountOutOfRange {
        rank_type: HonorRanksType,
        country: u8,
        count: usize,
    },
    NameWithoutTerminator {
        rank_type: HonorRanksType,
        country: u8,
        record_index: usize,
    },
}

impl fmt::Display for HonorRanksSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCountry { country } => {
                write!(formatter, "страна honor ranks вне диапазона 0..3: {country}")
            }
            Self::CountOutOfRange {
                rank_type,
                country,
                count,
            } => write!(
                formatter,
                "honor ranks {rank_type:?}/{country} содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::NameWithoutTerminator {
                rank_type,
                country,
                record_index,
            } => write!(
                formatter,
                "honor ranks {rank_type:?}/{country}, запись {record_index}: имя не содержит NUL"
            ),
        }
    }
}

impl Error for HonorRanksSerializationBlock {}

impl fmt::Display for HonorRanksNewDayBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalQueue {
                sort_day,
                rank_mask,
                source,
            } => write!(
                formatter,
                "honor rollover дня {sort_day}, mask {rank_mask:#X}, остановлен на локальной FIFO: {source}"
            ),
            Self::Serialization {
                sort_day,
                rank_mask,
                source,
            } => write!(
                formatter,
                "honor rollover дня {sort_day}, mask {rank_mask:#X}, остановлен на GameServer payload: {source}"
            ),
        }
    }
}

impl Error for HonorRanksNewDayBlock {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LocalQueue { source, .. } => Some(source),
            Self::Serialization { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for HonorRankPushBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCountry { country } => {
                write!(formatter, "индекс страны honor ranks вне 0..3: {country}")
            }
            Self::NameTooLong {
                player_id,
                visible_name_length,
            } => write!(
                formatter,
                "имя игрока {player_id} содержит {visible_name_length} видимых байт и переполняет tagHorRank::name[20] с NUL"
            ),
        }
    }
}

impl Error for HonorRankPushBlock {}

impl fmt::Display for HonorRanksLocalQueueBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "локальное World-сообщение {:#08X} не поставлено: s_pNetServer отсутствует",
            self.message_type
        )
    }
}

impl Error for HonorRanksLocalQueueBlock {}

impl HonorRanksLoadSink for CHonorRanks {
    fn clear_honor_ranks_period(&mut self, period: HonorRanksSavePeriod) {
        match period {
            HonorRanksSavePeriod::History => self.clear_history_honor_ranks(),
            HonorRanksSavePeriod::Current => self.clear_current_honor_ranks(),
        }
    }

    fn replace_honor_ranks_type(
        &mut self,
        period: HonorRanksSavePeriod,
        rank_type: HonorRanksType,
        lists: [Vec<HonorRankDbEntry>; 4],
    ) {
        match period {
            HonorRanksSavePeriod::History => self.history[rank_type as usize] = lists,
            HonorRanksSavePeriod::Current => self.current[rank_type as usize] = lists,
        }
    }
}
