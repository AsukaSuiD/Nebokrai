//! Реестр runtime-свойств навыков GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/skills/skillfactory.cpp`. `Rebuild` очищает
//! прежний registry до чтения count, проходит все signed-count slots, пропускает
//! нулевую длину и публикует decoded record по ключу
//! `skill_id << 16 | level & 0xffff`; повторный ключ заменяется. Имя остаётся
//! byte-exact C-string prefix, usage map использует last-write-wins.
//!
//! Точный цикл и malformed-record skip подтверждены дизассемблировкой RVA
//! `0x0006CE10`; WorldServer serializer является парной стороной wire-контракта.
//! `BTreeMap`, `Vec` и `Drop` заменяют MSVC map/heap plumbing. Старые overflow
//! и out-of-bounds на повреждённой длине остановлены typed error-ом без
//! придумывания side effects. Concrete `QuerySkill` ещё остаётся RAW до
//! восстановления иерархии skill/state owners; registry и все его lookup-ы уже
//! являются исполняемым Rust.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::skillbaseproperties::{CSkillBaseProperties, UNKNOWN_SKILL_TYPE};

pub(crate) const UNKNOWN_SKILL_ID: u32 = 0x7fff_ffff;
const MAX_SKILL_NAME_LENGTH: usize = 255;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillFactoryDecodeReport {
    pub(crate) declared_slots: usize,
    pub(crate) published_records: usize,
    pub(crate) empty_slots: usize,
    pub(crate) skipped_records: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillFactoryDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        required: usize,
        available: usize,
    },
    NegativeSlotCount {
        count: i32,
    },
    NameTooLong {
        slot: usize,
        length: usize,
    },
    RecordTooShort {
        slot: usize,
        length: usize,
        required: usize,
    },
    UsageTableTooLarge {
        slot: usize,
        count: u32,
    },
}

impl fmt::Display for SkillFactoryDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                required,
                available,
            } => write!(
                formatter,
                "skill snapshot обрывается на {field} в {offset}: нужно {required}, доступно {available}"
            ),
            Self::NegativeSlotCount { count } => {
                write!(
                    formatter,
                    "skill snapshot содержит отрицательное число slots {count}"
                )
            }
            Self::NameTooLong { slot, length } => write!(
                formatter,
                "skill slot {slot} содержит имя длиной {length} при максимуме {MAX_SKILL_NAME_LENGTH}"
            ),
            Self::RecordTooShort {
                slot,
                length,
                required,
            } => write!(
                formatter,
                "skill slot {slot} содержит {length} байт, требуется {required}"
            ),
            Self::UsageTableTooLarge { slot, count } => write!(
                formatter,
                "skill slot {slot} содержит непредставимое число usage-записей {count}"
            ),
        }
    }
}

impl Error for SkillFactoryDecodeError {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CSkillFactory {
    properties: BTreeMap<u32, CSkillBaseProperties>,
}

impl CSkillFactory {
    pub(crate) const fn properties(&self) -> &BTreeMap<u32, CSkillBaseProperties> {
        &self.properties
    }

    pub(crate) fn clear_skill_cache(&mut self) {
        self.properties.clear();
    }

    pub(crate) fn rebuild(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<SkillFactoryDecodeReport, SkillFactoryDecodeError> {
        self.clear_skill_cache();
        let count = read_i32(source, cursor, "slot count")?;
        if count < 0 {
            return Err(SkillFactoryDecodeError::NegativeSlotCount { count });
        }

        let declared_slots = count as usize;
        let mut empty_slots = 0;
        let mut skipped_records = 0;
        for slot in 0..declared_slots {
            let length = read_u32(source, cursor, "record length")? as usize;
            if length == 0 {
                empty_slots += 1;
                continue;
            }
            let record = take_bytes(source, cursor, length, "skill record")?;
            let Some((key, properties)) = decode_record(record, slot)? else {
                skipped_records += 1;
                continue;
            };
            self.properties.insert(key, properties);
        }

        Ok(SkillFactoryDecodeReport {
            declared_slots,
            published_records: self.properties.len(),
            empty_slots,
            skipped_records,
        })
    }

    pub(crate) fn query_skill_base_properties(
        &self,
        skill_id: u32,
        level: i32,
    ) -> Option<&CSkillBaseProperties> {
        self.properties.get(&skill_key(skill_id, level as u32))
    }

    pub(crate) fn query_skill_type(&self, skill_id: u32, level: i32) -> u32 {
        self.query_skill_base_properties(skill_id, level)
            .map(CSkillBaseProperties::skill_type)
            .unwrap_or(UNKNOWN_SKILL_TYPE)
    }

    pub(crate) fn query_skill_id(&self, name: Option<&[u8]>) -> u32 {
        let Some(name) = name.map(visible_c_string) else {
            return UNKNOWN_SKILL_ID;
        };
        self.properties
            .iter()
            .find_map(|(&key, properties)| (properties.skill_name() == name).then_some(key >> 16))
            .unwrap_or(UNKNOWN_SKILL_ID)
    }

    pub(crate) fn query_skill_name(&self, skill_id: i32) -> Option<&[u8]> {
        self.properties.iter().find_map(|(&key, properties)| {
            ((key >> 16) == skill_id as u32).then_some(properties.skill_name())
        })
    }

    pub(crate) const fn get_skill_failed_message_color() -> u32 {
        0xffff_0000
    }

    pub(crate) const fn is_war_soul_skill(skill_id: u32) -> bool {
        skill_id > 0x211 && skill_id < 0x225
    }

    pub(crate) const fn is_need_float(skill_id: u32) -> bool {
        (skill_id > 0x211 && skill_id < 0x222) || skill_id == 0x224
    }
}

fn decode_record(
    record: &[u8],
    slot: usize,
) -> Result<Option<(u32, CSkillBaseProperties)>, SkillFactoryDecodeError> {
    const FIXED_PREFIX: usize = 20;
    if record.len() < FIXED_PREFIX {
        return Err(SkillFactoryDecodeError::RecordTooShort {
            slot,
            length: record.len(),
            required: FIXED_PREFIX,
        });
    }

    let skill_type = u32::from_le_bytes(record[0..4].try_into().expect("checked prefix"));
    let skill_id = u32::from_le_bytes(record[4..8].try_into().expect("checked prefix"));
    let level = u32::from_le_bytes(record[8..12].try_into().expect("checked prefix"));
    let is_target_self = i32::from_le_bytes(record[12..16].try_into().expect("checked prefix"));
    let name_length =
        u32::from_le_bytes(record[16..20].try_into().expect("checked prefix")) as usize;
    if name_length > MAX_SKILL_NAME_LENGTH {
        return Err(SkillFactoryDecodeError::NameTooLong {
            slot,
            length: name_length,
        });
    }
    let usage_count_offset =
        FIXED_PREFIX
            .checked_add(name_length)
            .ok_or(SkillFactoryDecodeError::RecordTooShort {
                slot,
                length: record.len(),
                required: usize::MAX,
            })?;
    let usage_data_offset =
        usage_count_offset
            .checked_add(4)
            .ok_or(SkillFactoryDecodeError::RecordTooShort {
                slot,
                length: record.len(),
                required: usize::MAX,
            })?;
    if record.len() < usage_data_offset {
        return Err(SkillFactoryDecodeError::RecordTooShort {
            slot,
            length: record.len(),
            required: usage_data_offset,
        });
    }

    let name = visible_c_string(&record[FIXED_PREFIX..usage_count_offset]).to_vec();
    let usage_count = u32::from_le_bytes(
        record[usage_count_offset..usage_data_offset]
            .try_into()
            .expect("checked usage count"),
    );
    if usage_count != 0 && usage_data_offset >= record.len() {
        return Ok(None);
    }
    let usage_bytes = usize::try_from(usage_count)
        .ok()
        .and_then(|count| count.checked_mul(8))
        .ok_or(SkillFactoryDecodeError::UsageTableTooLarge {
            slot,
            count: usage_count,
        })?;
    let required = usage_data_offset.checked_add(usage_bytes).ok_or(
        SkillFactoryDecodeError::UsageTableTooLarge {
            slot,
            count: usage_count,
        },
    )?;
    if record.len() < required {
        return Err(SkillFactoryDecodeError::RecordTooShort {
            slot,
            length: record.len(),
            required,
        });
    }

    let mut properties = CSkillBaseProperties::new(skill_type, is_target_self, name);
    for pair in record[usage_data_offset..required].chunks_exact(8) {
        let usage = u32::from_le_bytes(pair[0..4].try_into().expect("usage pair"));
        let value = u32::from_le_bytes(pair[4..8].try_into().expect("usage pair"));
        properties.set_property(usage, value);
    }
    Ok(Some((skill_key(skill_id, level), properties)))
}

const fn skill_key(skill_id: u32, level: u32) -> u32 {
    skill_id.wrapping_shl(16) | (level & 0xffff)
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, SkillFactoryDecodeError> {
    Ok(i32::from_le_bytes(
        take_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("i32 содержит четыре байта"),
    ))
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, SkillFactoryDecodeError> {
    Ok(u32::from_le_bytes(
        take_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("u32 содержит четыре байта"),
    ))
}

fn take_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    required: usize,
    field: &'static str,
) -> Result<&'a [u8], SkillFactoryDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(required) else {
        return Err(SkillFactoryDecodeError::UnexpectedEnd {
            field,
            offset,
            required,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(SkillFactoryDecodeError::UnexpectedEnd {
            field,
            offset,
            required,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skillfactory.cpp

// ============================================================================
// FUNCTION: CSkillFactory::QuerySkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skillfactory.cpp:354
// RVA: 0x00069870
// ADDRESS: 00469870
// PROTOTYPE: CSkill * __cdecl QuerySkill(tagSkillID param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
