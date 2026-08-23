//! Владелец таблиц почётных рангов исторического `WorldServer`.
//!
//! Статус `CHonorRanks::GenerateSaveData` RVA `0x0001B090` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`; `LoadHonorRanks` RVA `0x0001A5A0`,
//! `AddToByteArray` RVA `0x0001A6F0`,
//! accessors/clear RVA `0x0001A540..0x0001A890`, `UpdateRanksOnWorldServer`
//! RVA `0x0001A4C0`, `UpdateRanksOnGameServer` RVA `0x0001ABD0`,
//! `CopyHonorRanks` RVA `0x0001AE20` и `OnNewDay` RVA `0x0001B280` —
//! `IMPLEMENTED`; `PushToRanks` RVA `0x0001B510` и `KilledOnePlayer` RVA
//! `0x0001B680` также `IMPLEMENTED`. Остальные функции ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.h`,
//! `e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:372`
//! и reached constructor
//! `e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp`.
//!
//! Exact PDB задаёт live `m_HistoryHonorRanks/m_NowHonorRanks` и DB-копии как
//! два массива `std::list<tagHorRank>[4][4]`. Первый индекс — rank-type
//! day/week/month/total, второй — country `0..3`; это независимо подтверждают
//! `GetHistoryHonorRanks/GetNowHonorRanks`. `tagHorRank` занимает `0x24` и
//! целиком копируется вместе с двумя padding-байтами. Rust использует уже
//! доказанный `HonorRankDbEntry` и вложенные `Vec`, сохраняя оба порядка без
//! копирования MSVC list-layout.
//!
//! Generator для каждой из шестнадцати позиций сначала полностью очищает
//! прежний history DB-list и копирует соответствующий live history-list,
//! затем делает то же для current. Live-списки не меняются. Raw ошибочно
//! показывал `return` после первого освобождённого node. Точечный exact
//! диапазон `0x0041B090..0x0041B1C8` подтвердил полный cleanup-loop, вызов
//! list range-insert, ровно шестнадцать итераций и только один return после
//! `GetLocalTime`; после ответа disassembly прекращён.
//!
//! Полученный `SYSTEMTIME` записывается всеми четырьмя DWORD, то есть сохраняет
//! восемь `u16` полей `tagTime`. `chrono::Local::now` заменяет только Windows
//! clock API. `HonorRanksDbDataSnapshot` остаётся отдельной owned DB-копией:
//! последующий `SaveHonorRanksByType` может дренировать её, не меняя live
//! таблицы. До первого generator-вызова `Option::None` выражает нулевую, но
//! календарно невалидную constructor-копию `tagTime`; DB-owner такую дату не
//! наблюдает.
//!
//! Initial-config serializer читает только history-массив. Для выбранного
//! day/week/month/total type и country `-1` он последовательно пишет четыре
//! country-секции: signed count и записи `i32 player + u8 level + C-string
//! name + u8 occupation + u32 appellation + u32 eliminate`. Exact Game decoder
//! подтверждает те же границы. `Vec` заменяет `std::list`, сохраняя insertion
//! order; fixed `[u8; 20]` остаётся storage-контрактом, а отсутствие NUL теперь
//! typed-блокирует сериализацию вместо чтения C++ за пределами массива.
//!
//! Rank rollover принимает исходную битовую маску day/week/month/total и идёт
//! по четырём странам в прежнем порядке. Для day/week/month current-list после
//! полной копии в history очищается; total history тоже заменяется копией, но
//! current total намеренно остаётся накопительным. `Vec::clone/clear` заменяют
//! только MSVC list allocation/cleanup, не меняя порядок записей.
//!
//! GameServer update проверяет mask-биты по порядку day/week/month/total и
//! рассылает `0x7F801` subtype `0x27..0x2A`. Только total subtype сначала
//! содержит полный исходный mask, затем тот же history payload. Транспорт
//! использует готовый Rust network-owner; промежуточный MSVC vector для total
//! не переносится, поскольку framing и порядок bytes сохраняются напрямую.
//!
//! Суточный rollover сначала записывает текущий день месяца, вычисляет mask
//! `day|total`, добавляет month первого числа и week по понедельникам, затем
//! строго выполняет copy → локальный `0x5FD0C` в receive FIFO → GameServer
//! broadcasts. В отличие от старого Linux C++ донора, exact EXE не staging-ит
//! копии и не переставляет запись sort-day после успешной сериализации; Rust
//! сохраняет машинный порядок, а отсутствие обязательного net-server выражает
//! typed-блоком на месте прежнего null-dereference.
//!
//! `PushToRanks` RVA `0x0001B510` обновляет существующую запись без смены
//! имени либо добавляет новый snapshot игрока, stable-сортирует по убыванию
//! eliminate count, затем level, и удаляет хвост до десяти записей. Comparator
//! и цикл обрезки подтверждены exact диапазонами `0x0041B204..0x0041B219` и
//! `0x0041B629..0x0041B65E`. `Vec::sort_by` сохраняет прежний порядок полных
//! ties. Два несемантических padding-байта новой записи обнуляются вместо
//! публикации неопределённого stack-содержимого в DB blob.
//!
//! `CHonorRanks::getInstance` в EXE лениво выделяет единственный
//! process-global owner и при первом успехе записывает `SYSTEMTIME.wDay` в
//! `m_nSortDate`; деструктор освобождает только технические MSVC list-node.
//! В Rust `CHonorRanks` создаётся внешним lifecycle-owner-ом через
//! `with_reached_process_state`, а все достигнутые ingress получают один
//! `&mut CHonorRanks` через `WorldMainLoopOwners`. Это сохраняет начальный
//! sort-day и всё последующее наблюдаемое состояние, не перенося singleton,
//! `operator_new`, утечку process-global объекта и ручной cleanup list-node.

use std::error::Error;
use std::fmt;

use chrono::{Datelike, Local, Timelike};

use crate::dbaccess::worlddb::rsplayer::{
    HonorRankDbEntry, HonorRankDbLists, HonorRanksCopyTimeSnapshot, HonorRanksDbDataSnapshot,
    HonorRanksLoadOutcome, HonorRanksLoadSink, HonorRanksSavePeriod, HonorRanksType, RsPlayerOwner,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::appworld::player::CPlayer;
use crate::worldserver::worldserver::game::{CGame, WorldLocalMessageQueueBlock};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct HonorRanksGameServerUpdate {
    pub(crate) rank_type: HonorRanksType,
    pub(crate) subtype: i32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Полный наблюдаемый результат одного `CHonorRanks::OnNewDay`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct HonorRanksNewDayReport {
    pub(crate) sort_day: u32,
    pub(crate) rank_mask: u32,
    pub(crate) reset_message_queued: bool,
    pub(crate) game_server_updates: Vec<HonorRanksGameServerUpdate>,
    pub(crate) legacy_result: bool,
}

/// Первая безопасная граница уже начатого rollover-прохода.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum HonorRanksNewDayBlock {
    LocalQueue {
        sort_day: u32,
        rank_mask: u32,
        source: WorldLocalMessageQueueBlock,
    },
    Serialization {
        sort_day: u32,
        rank_mask: u32,
        source: HonorRanksSerializationBlock,
    },
}

/// Изменение одной current top-10 секции.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct HonorRankPushReport {
    pub(crate) rank_type: HonorRanksType,
    pub(crate) country: u8,
    pub(crate) player_id: i32,
    pub(crate) replaced_existing: bool,
    pub(crate) retained_in_top_ten: bool,
    pub(crate) removed_player_ids: Vec<i32>,
    pub(crate) legacy_result: bool,
}

/// Полный четыре-типа проход `KilledOnePlayer` для допустимой страны.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct HonorRanksKilledPlayerReport {
    pub(crate) country: u8,
    pub(crate) updates: Vec<HonorRankPushReport>,
}

/// Safe-граница исходного `strcpy(tagHorRank::name[20], player-name)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HonorRankPushBlock {
    InvalidCountry {
        country: u8,
    },
    NameTooLong {
        player_id: i32,
        visible_name_length: usize,
    },
}

/// Достигнутая save-часть process-static `CHonorRanks` state.
pub(crate) struct CHonorRanks {
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
    pub(crate) fn with_reached_process_state(sort_day: u32) -> Self {
        Self {
            history: Default::default(),
            current: Default::default(),
            db_data: None,
            sort_day,
        }
    }

    /// Создаёт доказанные пустые live/DB list-массивы, снимая текущий local day.
    pub(crate) fn with_reached_save_state() -> Self {
        Self::default()
    }

    /// Возвращает exact process-static `m_nSortDate`.
    pub(crate) const fn sort_day(&self) -> u32 {
        self.sort_day
    }

    /// Делегирует exact DB-проход `CRsPlayer`, сохраняя последовательную публикацию.
    pub(crate) async fn load_honor_ranks<R: RsPlayerOwner>(
        &mut self,
        database: &mut R,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> HonorRanksLoadOutcome {
        database
            .load_honor_ranks(self, active_transaction)
            .await
    }

    /// Заменяет все 32 DB-list полными ordered-копиями и затем снимает время.
    pub(crate) fn generate_save_data(&mut self) {
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

    /// Возвращает DB-копию для уже восстановленной Save HonorRanks phase.
    pub(crate) fn db_data_mut(&mut self) -> Option<&mut HonorRanksDbDataSnapshot> {
        self.db_data.as_mut()
    }

    /// Передаёт только уже сформированную DB-копию фоновому save-owner-у.
    /// Live history/current списки остаются у MainLoop и продолжают принимать
    /// новые результаты, пока прежний snapshot сохраняется в отдельном потоке.
    pub(crate) fn take_save_owner(&mut self) -> Self {
        Self {
            history: Default::default(),
            current: Default::default(),
            db_data: self.db_data.take(),
            sort_day: self.sort_day,
        }
    }

    /// Даёт loader/runtime-owner-у одну доказанную history-секцию.
    pub(crate) fn history_mut(
        &mut self,
        rank_type: HonorRanksType,
        country: u8,
    ) -> Option<&mut Vec<HonorRankDbEntry>> {
        self.history[rank_type as usize].get_mut(usize::from(country))
    }

    /// Даёт loader/runtime-owner-у одну доказанную current-секцию.
    pub(crate) fn current_mut(
        &mut self,
        rank_type: HonorRanksType,
        country: u8,
    ) -> Option<&mut Vec<HonorRankDbEntry>> {
        self.current[rank_type as usize].get_mut(usize::from(country))
    }

    /// Полностью очищает все шестнадцать history-list в исходном порядке.
    pub(crate) fn clear_history_honor_ranks(&mut self) {
        for by_country in &mut self.history {
            for ranks in by_country {
                ranks.clear();
            }
        }
    }

    /// Полностью очищает все шестнадцать current-list в исходном порядке.
    pub(crate) fn clear_current_honor_ranks(&mut self) {
        for by_country in &mut self.current {
            for ranks in by_country {
                ranks.clear();
            }
        }
    }

    /// Копирует выбранные rank-типы current → history перед новым периодом.
    pub(crate) fn copy_honor_ranks(&mut self, rank_mask: u32) -> bool {
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

    /// Ставит точный `0x5FD0C + mask` в World receive FIFO.
    pub(crate) fn update_ranks_on_world_server(
        game: &CGame,
        rank_mask: u32,
    ) -> Result<(), WorldLocalMessageQueueBlock> {
        let mut message = CMessage::new(0x0005_FD0C);
        message.base_mut().add_ulong(rank_mask);
        game.queue_local_world_message(message)
    }

    /// Выполняет точный суточный rollover; `first_load` исходник не читал.
    pub(crate) fn on_new_day(
        &mut self,
        game: &CGame,
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

    /// Обновляет одну current top-10 секцию по exact `PushToRanks`.
    pub(crate) fn push_to_ranks(
        &mut self,
        rank_type: HonorRanksType,
        country: u8,
        eliminate_num: u32,
        player: &CPlayer,
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

    /// Последовательно обновляет day/week/month/total для допустимой страны.
    pub(crate) fn killed_one_player(
        &mut self,
        player: &CPlayer,
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

    /// Рассылает выбранные history-типы всем подключённым GameServer-ам.
    pub(crate) fn update_ranks_on_game_server(
        &self,
        game: &CGame,
        rank_mask: u32,
    ) -> Result<Vec<HonorRanksGameServerUpdate>, HonorRanksSerializationBlock> {
        let mut updates = Vec::new();
        let sender = game.current_game_server_sender();
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

    /// Повторяет `AddToByteArray(type, country)`; `None` соответствует `-1`.
    pub(crate) fn add_history_to_byte_array(
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

/// Safe-границы старого count/C-string wire почётных рангов.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HonorRanksSerializationBlock {
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
