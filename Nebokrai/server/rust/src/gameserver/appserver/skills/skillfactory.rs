//! Реестр runtime-свойств навыков GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/skills/skillfactory.cpp`. `Rebuild` очищает
//! прежний реестр до чтения `count`, проходит все ячейки знакового количества, пропускает
//! нулевую длину и публикует декодированную запись по ключу
//! `skill_id << 16 | level & 0xffff`; повторный ключ заменяется. Имя остаётся
//! точный по байтам префикс C-string, а карта usage использует последнее значение ключа.
//!
//! Точный цикл и malformed-record skip подтверждены дизассемблировкой RVA
//! `0x0006CE10`; WorldServer serializer является парной стороной wire-контракта.
//! `BTreeMap`, `Vec` и `Drop` заменяют служебный код MSVC map/heap. Старые
//! переполнения и выходы за границу при повреждённой длине остановлены
//! типизированной ошибкой без дополнительных side effects. Конкретный
//! `QuerySkill` ещё остаётся RAW до восстановления иерархии владельцев
//! skill/state; реестр и все его операции поиска уже исполняются в Rust.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::skillbaseproperties::{CSkillBaseProperties, UNKNOWN_SKILL_TYPE};
use super::super::legacycodec::LegacyReader;

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

    let mut reader = LegacyReader::new(record);
    let skill_type = reader.read_u32().expect("проверен префикс записи");
    let skill_id = reader.read_u32().expect("проверен префикс записи");
    let level = reader.read_u32().expect("проверен префикс записи");
    let is_target_self = reader.read_i32().expect("проверен префикс записи");
    let name_length = reader.read_u32().expect("проверен префикс записи") as usize;
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

    let name = visible_c_string(
        reader
            .read_bytes(name_length)
            .expect("проверена длина имени"),
    )
    .to_vec();
    let usage_count = reader.read_u32().expect("проверено число usage-записей");
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
    for _ in 0..usage_count {
        let usage = reader.read_u32().expect("проверена usage-пара");
        let value = reader.read_u32().expect("проверена usage-пара");
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
    let mut reader = skill_reader(source, *cursor, field, 4)?;
    let value = reader.read_i32().map_err(|block| skill_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, SkillFactoryDecodeError> {
    let mut reader = skill_reader(source, *cursor, field, 4)?;
    let value = reader.read_u32().map_err(|block| skill_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn take_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    required: usize,
    field: &'static str,
) -> Result<&'a [u8], SkillFactoryDecodeError> {
    let mut reader = skill_reader(source, *cursor, field, required)?;
    let bytes = reader
        .read_bytes(required)
        .map_err(|block| skill_read_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn skill_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    required: usize,
) -> Result<LegacyReader<'source>, SkillFactoryDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| SkillFactoryDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        required,
        available: block.available,
    })
}

fn skill_read_error(
    field: &'static str,
    block: super::super::legacycodec::LegacyReadBlock,
) -> SkillFactoryDecodeError {
    SkillFactoryDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        required: block.needed,
        available: block.available,
    }
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
