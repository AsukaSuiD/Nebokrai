//! Владелец таблиц почётных рангов исторического `WorldServer`.
//!
//! Статус `CHonorRanks::GenerateSaveData` RVA `0x0001B090` —
//! `IMPLEMENTED/VERIFIED_DISASSEMBLY`; `AddToByteArray` RVA `0x0001A6F0`,
//! accessors/clear RVA `0x0001A540..0x0001A890`, `UpdateRanksOnGameServer` RVA
//! `0x0001ABD0` и `CopyHonorRanks` RVA `0x0001AE20` — `IMPLEMENTED`. Остальные
//! функции ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
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

use std::error::Error;
use std::fmt;

use chrono::{Datelike, Local, Timelike};

use crate::dbaccess::worlddb::rsplayer::{
    HonorRankDbEntry, HonorRankDbLists, HonorRanksCopyTimeSnapshot, HonorRanksDbDataSnapshot,
    HonorRanksType,
};
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct HonorRanksGameServerUpdate {
    pub(crate) rank_type: HonorRanksType,
    pub(crate) subtype: i32,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

/// Достигнутая save-часть process-static `CHonorRanks` state.
#[derive(Default)]
pub(crate) struct CHonorRanks {
    history: HonorRankDbLists,
    current: HonorRankDbLists,
    db_data: Option<HonorRanksDbDataSnapshot>,
}

impl CHonorRanks {
    /// Создаёт доказанные пустые live/DB list-массивы.
    pub(crate) fn with_reached_save_state() -> Self {
        Self::default()
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp


// ============================================================================
// FUNCTION: MyStringTable::~MyStringTable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x000017B0
// ADDRESS: 004017b0
// PROTOTYPE: void __thiscall ~MyStringTable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// ============================================================================
// FUNCTION: Catch@00404050
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00004050
// ADDRESS: 00404050
// PROTOTYPE: undefined Catch@00404050()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00404078
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00004078
// ADDRESS: 00404078
// PROTOTYPE: undefined FUN_00404078()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00404253
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00004253
// ADDRESS: 00404253
// PROTOTYPE: undefined Catch@00404253()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0040427e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000427E
// ADDRESS: 0040427e
// PROTOTYPE: undefined FUN_0040427e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004042ed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x000042ED
// ADDRESS: 004042ed
// PROTOTYPE: undefined Catch@004042ed()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CGame::tagSetup::~tagSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00004DE0
// ADDRESS: 00404de0
// PROTOTYPE: void __thiscall ~tagSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00405bf3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00005BF3
// ADDRESS: 00405bf3
// PROTOTYPE: undefined Catch@00405bf3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00405c1b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00005C1B
// ADDRESS: 00405c1b
// PROTOTYPE: undefined FUN_00405c1b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00405ec9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00005EC9
// ADDRESS: 00405ec9
// PROTOTYPE: undefined Catch@00405ec9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00405ef1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00005EF1
// ADDRESS: 00405ef1
// PROTOTYPE: undefined FUN_00405ef1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004068bf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x000068BF
// ADDRESS: 004068bf
// PROTOTYPE: undefined Catch@004068bf()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004068e7
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x000068E7
// ADDRESS: 004068e7
// PROTOTYPE: undefined FUN_004068e7()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00406aee
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00006AEE
// ADDRESS: 00406aee
// PROTOTYPE: undefined Catch@00406aee()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00406b16
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00006B16
// ADDRESS: 00406b16
// PROTOTYPE: undefined FUN_00406b16()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::tagGameServer::~tagGameServer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00006DA0
// ADDRESS: 00406da0
// PROTOTYPE: void __thiscall ~tagGameServer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Connection15::Open
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007010
// ADDRESS: 00407010
// PROTOTYPE: long __thiscall Open(_bstr_t param_1, _bstr_t param_2, _bstr_t param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagVilWarSetup::~tagVilWarSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007470
// ADDRESS: 00407470
// PROTOTYPE: void __thiscall ~tagVilWarSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagAttackCityTime::~tagAttackCityTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007480
// ADDRESS: 00407480
// PROTOTYPE: void __thiscall ~tagAttackCityTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004075e8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x000075E8
// ADDRESS: 004075e8
// PROTOTYPE: undefined Catch@004075e8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00407610
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007610
// ADDRESS: 00407610
// PROTOTYPE: undefined FUN_00407610()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040775b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000775B
// ADDRESS: 0040775b
// PROTOTYPE: undefined Catch@0040775b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00407783
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007783
// ADDRESS: 00407783
// PROTOTYPE: undefined FUN_00407783()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004078f8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x000078F8
// ADDRESS: 004078f8
// PROTOTYPE: undefined Catch@004078f8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00407920
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007920
// ADDRESS: 00407920
// PROTOTYPE: undefined FUN_00407920()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00407a68
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007A68
// ADDRESS: 00407a68
// PROTOTYPE: undefined Catch@00407a68()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00407a90
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007A90
// ADDRESS: 00407a90
// PROTOTYPE: undefined FUN_00407a90()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00407bd8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007BD8
// ADDRESS: 00407bd8
// PROTOTYPE: undefined Catch@00407bd8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00407c00
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00007C00
// ADDRESS: 00407c00
// PROTOTYPE: undefined FUN_00407c00()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::tagSysBroadcast::tagSysBroadcast
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00008FA0
// ADDRESS: 00408fa0
// PROTOTYPE: undefined __thiscall tagSysBroadcast(tagSysBroadcast * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040aa86
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000AA86
// ADDRESS: 0040aa86
// PROTOTYPE: undefined Catch@0040aa86()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040cbbc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000CBBC
// ADDRESS: 0040cbbc
// PROTOTYPE: undefined Catch@0040cbbc()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040e05e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000E05E
// ADDRESS: 0040e05e
// PROTOTYPE: undefined Catch@0040e05e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040e0e5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000E0E5
// ADDRESS: 0040e0e5
// PROTOTYPE: undefined Catch@0040e0e5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040e652
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000E652
// ADDRESS: 0040e652
// PROTOTYPE: undefined Catch@0040e652()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040e6e2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000E6E2
// ADDRESS: 0040e6e2
// PROTOTYPE: undefined Catch@0040e6e2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLingBaoSetup::stNodeInfo::~stNodeInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000E740
// ADDRESS: 0040e740
// PROTOTYPE: void __thiscall ~stNodeInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040f6e5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000F6E5
// ADDRESS: 0040f6e5
// PROTOTYPE: undefined Catch@0040f6e5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::tagDBData::~tagDBData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0000F760
// ADDRESS: 0040f760
// PROTOTYPE: void __thiscall ~tagDBData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::tagDBData::tagDBData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x00011F60
// ADDRESS: 00411f60
// PROTOTYPE: undefined __thiscall tagDBData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:59
// RVA: 0x0001A460
// ADDRESS: 0041a460
// PROTOTYPE: CHonorRanks * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::UpdateRanksOnWorldServer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:137
// RVA: 0x0001A4C0
// ADDRESS: 0041a4c0
// PROTOTYPE: bool __cdecl UpdateRanksOnWorldServer(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::GetHistoryHonorRanks
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:328
// RVA: 0x0001A540
// ADDRESS: 0041a540
// PROTOTYPE: list<tagHorRank,std::allocator<tagHorRank>_> * __cdecl GetHistoryHonorRanks(int param_1, int param_2)
//
// Реализовано выше как typed history accessor.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::GetNowHonorRanks
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:337
// RVA: 0x0001A570
// ADDRESS: 0041a570
// PROTOTYPE: list<tagHorRank,std::allocator<tagHorRank>_> * __cdecl GetNowHonorRanks(int param_1, int param_2)
//
// Реализовано выше как typed current accessor.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::LoadHonorRanks
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:72
// RVA: 0x0001A5A0
// ADDRESS: 0041a5a0
// PROTOTYPE: bool __cdecl LoadHonorRanks(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::~CHonorRanks
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:47
// RVA: 0x0001A650
// ADDRESS: 0041a650
// PROTOTYPE: void __thiscall ~CHonorRanks(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:187
// RVA: 0x0001A6F0
// ADDRESS: 0041a6f0
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2, int param_3)
//
// Реализовано выше без копирования ABI-layout `std::list`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::ClearHistoryHonorRanks
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:346
// RVA: 0x0001A840
// ADDRESS: 0041a840
// PROTOTYPE: void __cdecl ClearHistoryHonorRanks(void)
//
// Реализовано выше через полную очистку шестнадцати `Vec`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::ClearNowHonorRanks
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:357
// RVA: 0x0001A890
// ADDRESS: 0041a890
// PROTOTYPE: void __cdecl ClearNowHonorRanks(void)
//
// Реализовано выше через полную очистку шестнадцати `Vec`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::UpdateRanksOnGameServer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:147
// RVA: 0x0001ABD0
// ADDRESS: 0041abd0
// PROTOTYPE: bool __cdecl UpdateRanksOnGameServer(ulong param_1)
//
// Реализовано выше через готовый Rust network-owner и точный subtype framing.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::CopyHonorRanks
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:102
// RVA: 0x0001AE20
// ADDRESS: 0041ae20
// PROTOTYPE: bool __cdecl CopyHonorRanks(ulong param_1)
//
// Реализовано выше с исходной битовой маской и накопительным total-list.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED/VERIFIED_DISASSEMBLY выше: CHonorRanks::GenerateSaveData,
// WorldServer RVA 0x0001B090. Полный заменённый raw и STL cleanup удалены.

// ============================================================================
// FUNCTION: CHonorRanks::OnNewDay
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:81
// RVA: 0x0001B280
// ADDRESS: 0041b280
// PROTOTYPE: bool __cdecl OnNewDay(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::PushToRanks
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:297
// RVA: 0x0001B510
// ADDRESS: 0041b510
// PROTOTYPE: bool __cdecl PushToRanks(int param_1, int param_2, ulong param_3, CPlayer * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::KilledOnePlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp:285
// RVA: 0x0001B680
// ADDRESS: 0041b680
// PROTOTYPE: void __cdecl KilledOnePlayer(CPlayer * param_1, ulong param_2, ulong param_3, ulong param_4, ulong param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// ============================================================================
// FUNCTION: Unwind@0052b820
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0012B820
// ADDRESS: 0052b820
// PROTOTYPE: undefined Unwind@0052b820()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//














// ============================================================================
// FUNCTION: Unwind@0052bca0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0012BCA0
// ADDRESS: 0052bca0
// PROTOTYPE: undefined Unwind@0052bca0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@0052bd20
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0012BD20
// ADDRESS: 0052bd20
// PROTOTYPE: undefined Unwind@0052bd20()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052bd50
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0012BD50
// ADDRESS: 0052bd50
// PROTOTYPE: undefined Unwind@0052bd50()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052bd70
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0012BD70
// ADDRESS: 0052bd70
// PROTOTYPE: undefined Unwind@0052bd70()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



































































































































// ============================================================================
// FUNCTION: Unwind@0052c900
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0012C900
// ADDRESS: 0052c900
// PROTOTYPE: undefined Unwind@0052c900()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: FUN_0053bb40
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0013BB40
// ADDRESS: 0053bb40
// PROTOTYPE: undefined FUN_0053bb40()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0053bb60
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\honorranks.cpp
// RVA: 0x0013BB60
// ADDRESS: 0053bb60
// PROTOTYPE: undefined FUN_0053bb60()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
