//! Startup snapshot `CHonorRanks` исторического GameServer.
//!
//! `DecordFromByteArray` RVA `0x0000D390` подтверждает 4 rank types × 4
//! country lists. Для country `-1` списки очищаются и декодируются по порядку;
//! каждый record содержит player ID, level byte, NUL-name, occupation byte,
//! appellation ID и eliminate count. Non-positive count означает пустой list.
//! Reached honor NPC scripts материализуют также `GetPlayerPosition` и
//! `AddToByteArray`: первый сохраняет 1-based snapshot order, второй — exact
//! count/record payload для адресного client `0xBFF35`.
//! Точная пара и исходный owner указаны ниже.
//!
//! `Vec` заменяет только `std::list`; уже очищенные списки и полностью
//! прочитанный prefix сохраняются при безопасном отказе на обрыве wire. Остальной
//! сырой C++ ниже остаётся доказательной документацией незакрытых методов.

use thiserror::Error;

const RANK_TYPE_COUNT: usize = 4;
const COUNTRY_COUNT: usize = 4;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct HonorRank {
    pub(crate) player_id: i32,
    pub(crate) level: u8,
    pub(crate) name: Vec<u8>,
    pub(crate) occupation_id: u8,
    pub(crate) appellation_id: u32,
    pub(crate) eliminate_count: u32,
}

pub(crate) struct CHonorRanks {
    history: [[Vec<HonorRank>; COUNTRY_COUNT]; RANK_TYPE_COUNT],
}

impl Default for CHonorRanks {
    fn default() -> Self {
        Self {
            history: std::array::from_fn(|_| std::array::from_fn(|_| Vec::new())),
        }
    }
}

impl CHonorRanks {
    pub(crate) fn history(&self, rank_type: i32, country: i32) -> Option<&[HonorRank]> {
        let rank_type = valid_index(rank_type, RANK_TYPE_COUNT)?;
        let country = valid_index(country, COUNTRY_COUNT)?;
        Some(&self.history[rank_type][country])
    }

    /// Exact `GetPlayerPosition`: порядок World snapshot уже является местом
    /// игрока, поэтому GameServer только возвращает 1-based index либо ноль.
    pub(crate) fn player_position(&self, rank_type: i32, country: i32, player_id: i32) -> i32 {
        self.history(rank_type, country)
            .and_then(|ranks| ranks.iter().position(|rank| rank.player_id == player_id))
            .and_then(|position| i32::try_from(position + 1).ok())
            .unwrap_or(0)
    }

    /// Exact payload `AddToByteArray` для client honor-list: signed count,
    /// byte level, NUL-name, byte occupation и два DWORD следуют без padding.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        rank_type: i32,
        country: i32,
    ) -> bool {
        if valid_index(rank_type, RANK_TYPE_COUNT).is_none() {
            return false;
        }
        if country == -1 {
            for country in 0..COUNTRY_COUNT as i32 {
                if !self.add_to_byte_array(destination, rank_type, country) {
                    return false;
                }
            }
            return true;
        }
        let Some(ranks) = self.history(rank_type, country) else {
            return false;
        };
        let Ok(count) = i32::try_from(ranks.len()) else {
            return false;
        };
        destination.extend_from_slice(&count.to_le_bytes());
        for rank in ranks {
            destination.extend_from_slice(&rank.player_id.to_le_bytes());
            destination.push(rank.level);
            destination.extend_from_slice(&rank.name);
            destination.push(0);
            destination.push(rank.occupation_id);
            destination.extend_from_slice(&rank.appellation_id.to_le_bytes());
            destination.extend_from_slice(&rank.eliminate_count.to_le_bytes());
        }
        true
    }

    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        rank_type: i32,
        country: i32,
    ) -> Result<HonorRanksDecodeReport, HonorRanksDecodeError> {
        let rank_type_index = valid_index(rank_type, RANK_TYPE_COUNT)
            .ok_or(HonorRanksDecodeError::InvalidRankType { rank_type })?;
        let countries = if country == -1 {
            0..COUNTRY_COUNT
        } else {
            let country_index = valid_index(country, COUNTRY_COUNT)
                .ok_or(HonorRanksDecodeError::InvalidCountry { country })?;
            country_index..country_index + 1
        };

        let mut decoded_countries = 0usize;
        let mut decoded_entries = 0usize;
        for country_index in countries {
            let ranks = &mut self.history[rank_type_index][country_index];
            ranks.clear();
            let count = read_i32(source, cursor, "rank count", rank_type, country_index, None)?;
            for rank_index in 0..count.max(0) {
                let player_id = read_i32(
                    source,
                    cursor,
                    "player ID",
                    rank_type,
                    country_index,
                    Some(rank_index),
                )?;
                let level = read_u8(
                    source,
                    cursor,
                    "level",
                    rank_type,
                    country_index,
                    Some(rank_index),
                )?;
                let name =
                    read_c_string(source, cursor, "name", rank_type, country_index, rank_index)?;
                let occupation_id = read_u8(
                    source,
                    cursor,
                    "occupation ID",
                    rank_type,
                    country_index,
                    Some(rank_index),
                )?;
                let appellation_id = read_u32(
                    source,
                    cursor,
                    "appellation ID",
                    rank_type,
                    country_index,
                    Some(rank_index),
                )?;
                let eliminate_count = read_u32(
                    source,
                    cursor,
                    "eliminate count",
                    rank_type,
                    country_index,
                    Some(rank_index),
                )?;
                ranks.push(HonorRank {
                    player_id,
                    level,
                    name,
                    occupation_id,
                    appellation_id,
                    eliminate_count,
                });
                decoded_entries += 1;
            }
            decoded_countries += 1;
        }

        Ok(HonorRanksDecodeReport {
            rank_type,
            decoded_countries,
            decoded_entries,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HonorRanksDecodeReport {
    pub(crate) rank_type: i32,
    pub(crate) decoded_countries: usize,
    pub(crate) decoded_entries: usize,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum HonorRanksDecodeError {
    #[error("CHonorRanks получил rank type {rank_type} вне 0..4")]
    InvalidRankType {
        rank_type: i32,
    },
    #[error("CHonorRanks получил country {country} вне 0..4/-1")]
    InvalidCountry {
        country: i32,
    },
    #[error("CHonorRanks type {rank_type}, country {country}, record {rank_index:?} обрывается на {field} в {offset}: нужно {required}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        rank_type: i32,
        country: usize,
        rank_index: Option<i32>,
        offset: usize,
        required: usize,
        available: usize,
    },
}

fn valid_index(value: i32, length: usize) -> Option<usize> {
    let value = usize::try_from(value).ok()?;
    (value < length).then_some(value)
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    rank_type: i32,
    country: usize,
    rank_index: Option<i32>,
) -> Result<i32, HonorRanksDecodeError> {
    Ok(i32::from_le_bytes(read_array(
        source, cursor, field, rank_type, country, rank_index,
    )?))
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    rank_type: i32,
    country: usize,
    rank_index: Option<i32>,
) -> Result<u32, HonorRanksDecodeError> {
    Ok(u32::from_le_bytes(read_array(
        source, cursor, field, rank_type, country, rank_index,
    )?))
}

fn read_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    rank_type: i32,
    country: usize,
    rank_index: Option<i32>,
) -> Result<u8, HonorRanksDecodeError> {
    Ok(read_array::<1>(source, cursor, field, rank_type, country, rank_index)?[0])
}

fn read_array<const SIZE: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    rank_type: i32,
    country: usize,
    rank_index: Option<i32>,
) -> Result<[u8; SIZE], HonorRanksDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(SIZE) else {
        return Err(HonorRanksDecodeError::UnexpectedEnd {
            field,
            rank_type,
            country,
            rank_index,
            offset,
            required: SIZE,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(HonorRanksDecodeError::UnexpectedEnd {
            field,
            rank_type,
            country,
            rank_index,
            offset,
            required: SIZE,
            available,
        });
    };
    *cursor = end;
    Ok(bytes
        .try_into()
        .expect("размер CHonorRanks scalar уже проверен"))
}

fn read_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    rank_type: i32,
    country: usize,
    rank_index: i32,
) -> Result<Vec<u8>, HonorRanksDecodeError> {
    let offset = *cursor;
    let remaining = source.get(offset..).unwrap_or_default();
    let Some(length) = remaining.iter().position(|byte| *byte == 0) else {
        return Err(HonorRanksDecodeError::UnexpectedEnd {
            field,
            rank_type,
            country,
            rank_index: Some(rank_index),
            offset,
            required: 1,
            available: remaining.len(),
        });
    };
    *cursor += length + 1;
    Ok(remaining[..length].to_vec())
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\honorranks.cpp

// ============================================================================
// FUNCTION: CHonorRanks::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\honorranks.cpp:54
// RVA: 0x0000CFA0
// ADDRESS: 0040cfa0
// PROTOTYPE: CHonorRanks * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// `GetHistoryHonorRanks` материализован выше как checked slice lookup.

// ============================================================================
// FUNCTION: CHonorRanks::~CHonorRanks
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\honorranks.cpp:43
// RVA: 0x0000D0D0
// ADDRESS: 0040d0d0
// PROTOTYPE: void __thiscall ~CHonorRanks(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHonorRanks::AddToByteArray
// STATUS: IMPLEMENTED ABOVE; DECOMPILER STORED LOCALLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\honorranks.cpp:67
// RVA: 0x0000D120
// ADDRESS: 0040d120
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// `DecordFromByteArray` материализован выше с последовательной prefix-мутацией.

// ============================================================================
// FUNCTION: $E4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\honorranks.cpp:9
// RVA: 0x00249D40
// ADDRESS: 00649d40
// PROTOTYPE: void __cdecl $E4(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $E2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\gameserver\honorranks.cpp
// RVA: 0x0024A640
// ADDRESS: 0064a640
// PROTOTYPE: void __cdecl $E2(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
