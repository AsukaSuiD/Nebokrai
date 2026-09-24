//! Полученный реестр runtime-свойств навыков Zone.
//! Источник: `gameserver.exe` + `GameServer.pdb`,
//! `server/gameserver/appserver/skills/skillfactory.cpp` и
//! `skillbaseproperties.cpp`. Точный цикл `Rebuild` и malformed-record skip
//! подтверждены дизассемблировкой RVA `0x0006CE10`; сериализатор WorldServer
//! является парной стороной wire-контракта. `Rebuild` очищает прежний реестр
//! до чтения `count`, проходит все ячейки знакового количества, пропускает
//! нулевую длину и публикует декодированную запись по ключу
//! `skill_id << 16 | level & 0xffff`; повторный ключ заменяется. Имя остаётся
//! точный по байтам префикс C-string, а карта usage использует последнее
//! значение ключа. Диспетчеризация конкретных владельцев навыков остаётся у
//! переходного Game.

use std::collections::BTreeMap;

use nebokrai_shared::protocol::LegacyReader;

pub const UNKNOWN_SKILL_TYPE: u32 = 0x7fff_ffff;
const MAX_SKILL_NAME_LENGTH: usize = 255;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CSkillBaseProperties {
    skill_type: u32,
    is_target_self: i32,
    skill_name: Vec<u8>,
    properties: BTreeMap<u32, u32>,
}

impl CSkillBaseProperties {
    pub fn new(skill_type: u32, is_target_self: i32, skill_name: Vec<u8>) -> Self {
        Self {
            skill_type,
            is_target_self,
            skill_name,
            properties: BTreeMap::new(),
        }
    }

    pub const fn skill_type(&self) -> u32 {
        self.skill_type
    }

    pub const fn is_target_self(&self) -> i32 {
        self.is_target_self
    }

    pub fn skill_name(&self) -> &[u8] {
        &self.skill_name
    }

    pub fn query_property(&self, usage: u32) -> u32 {
        self.properties.get(&usage).copied().unwrap_or(0)
    }

    pub fn set_property(&mut self, usage: u32, value: u32) -> Option<u32> {
        self.properties.insert(usage, value)
    }

    pub const fn properties(&self) -> &BTreeMap<u32, u32> {
        &self.properties
    }
}

#[derive(Debug, thiserror::Error, Clone, Copy, Eq, PartialEq)]
pub enum SkillPropertiesDecodeError {
    #[error(
        "skill snapshot обрывается на {field} в {offset}: нужно {required}, доступно {available}"
    )]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        required: usize,
        available: usize,
    },
    #[error("skill snapshot содержит отрицательное число slots {count}")]
    NegativeSlotCount { count: i32 },
    #[error("skill slot {slot} содержит имя длиной {length} при максимуме {MAX_SKILL_NAME_LENGTH}")]
    NameTooLong { slot: usize, length: usize },
    #[error("skill slot {slot} содержит {length} байт, требуется {required}")]
    RecordTooShort {
        slot: usize,
        length: usize,
        required: usize,
    },
    #[error("skill slot {slot} содержит непредставимое число usage-записей {count}")]
    UsageTableTooLarge { slot: usize, count: u32 },
}

/// Опубликованные определения Zone; живые экземпляры навыков и реестр
/// владельцев остаются у переходного Game.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SkillPropertiesCatalog {
    properties: BTreeMap<u32, CSkillBaseProperties>,
}

impl SkillPropertiesCatalog {
    pub const fn properties(&self) -> &BTreeMap<u32, CSkillBaseProperties> {
        &self.properties
    }

    pub fn clear(&mut self) {
        self.properties.clear();
    }

    /// Устанавливает полученный snapshot с исходными частичными эффектами:
    /// очистка до count, пропуск пустых и повреждённых записей, публикация
    /// остальных; повторный ключ заменяется.
    pub fn rebuild(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), SkillPropertiesDecodeError> {
        self.clear();
        let count = read_i32(source, cursor, "slot count")?;
        if count < 0 {
            return Err(SkillPropertiesDecodeError::NegativeSlotCount { count });
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

        tracing::trace!(
            declared_slots,
            published_records = self.properties.len(),
            empty_slots,
            skipped_records,
            "реестр навыков декодирован"
        );
        Ok(())
    }

    pub fn query_skill_base_properties(
        &self,
        skill_id: u32,
        level: i32,
    ) -> Option<&CSkillBaseProperties> {
        self.properties.get(&skill_key(skill_id, level as u32))
    }

    pub fn query_skill_type(&self, skill_id: u32, level: i32) -> u32 {
        self.query_skill_base_properties(skill_id, level)
            .map(CSkillBaseProperties::skill_type)
            .unwrap_or(UNKNOWN_SKILL_TYPE)
    }

    pub fn query_skill_id(&self, name: Option<&[u8]>) -> Option<u32> {
        let name = name.map(visible_c_string)?;
        self.properties
            .iter()
            .find_map(|(&key, properties)| (properties.skill_name() == name).then_some(key >> 16))
    }

    pub fn query_skill_name(&self, skill_id: i32) -> Option<&[u8]> {
        self.properties.iter().find_map(|(&key, properties)| {
            ((key >> 16) == skill_id as u32).then_some(properties.skill_name())
        })
    }
}

pub const fn skill_failed_message_color() -> u32 {
    0xffff_0000
}

pub const fn is_war_soul_skill(skill_id: u32) -> bool {
    skill_id > 0x211 && skill_id < 0x225
}

pub const fn is_need_float(skill_id: u32) -> bool {
    (skill_id > 0x211 && skill_id < 0x222) || skill_id == 0x224
}

fn decode_record(
    record: &[u8],
    slot: usize,
) -> Result<Option<(u32, CSkillBaseProperties)>, SkillPropertiesDecodeError> {
    const FIXED_PREFIX: usize = 20;
    if record.len() < FIXED_PREFIX {
        return Err(SkillPropertiesDecodeError::RecordTooShort {
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
        return Err(SkillPropertiesDecodeError::NameTooLong {
            slot,
            length: name_length,
        });
    }
    let usage_count_offset = FIXED_PREFIX.checked_add(name_length).ok_or(
        SkillPropertiesDecodeError::RecordTooShort {
            slot,
            length: record.len(),
            required: usize::MAX,
        },
    )?;
    let usage_data_offset =
        usage_count_offset
            .checked_add(4)
            .ok_or(SkillPropertiesDecodeError::RecordTooShort {
                slot,
                length: record.len(),
                required: usize::MAX,
            })?;
    if record.len() < usage_data_offset {
        return Err(SkillPropertiesDecodeError::RecordTooShort {
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
        .ok_or(SkillPropertiesDecodeError::UsageTableTooLarge {
            slot,
            count: usage_count,
        })?;
    let required = usage_data_offset.checked_add(usage_bytes).ok_or(
        SkillPropertiesDecodeError::UsageTableTooLarge {
            slot,
            count: usage_count,
        },
    )?;
    if record.len() < required {
        return Err(SkillPropertiesDecodeError::RecordTooShort {
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
) -> Result<i32, SkillPropertiesDecodeError> {
    let mut reader = skill_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_i32()
        .map_err(|block| skill_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, SkillPropertiesDecodeError> {
    let mut reader = skill_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_u32()
        .map_err(|block| skill_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn take_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    required: usize,
    field: &'static str,
) -> Result<&'a [u8], SkillPropertiesDecodeError> {
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
) -> Result<LegacyReader<'source>, SkillPropertiesDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| SkillPropertiesDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        required,
        available: block.available,
    })
}

fn skill_read_error(
    field: &'static str,
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> SkillPropertiesDecodeError {
    SkillPropertiesDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        required: block.needed,
        available: block.available,
    }
}
