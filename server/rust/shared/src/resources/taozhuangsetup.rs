//! Конфигурация комплектов TaoZhuang исторического Miracle.
//!
//! World `CTaoZhuangSetup::ReadFile/AddByteToArray` подтверждены точной парой
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, Game
//! `DeCodeFromByte` — парой `GameServer/gameserver.exe + GameServer/GameServer.pdb`.
//! Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp/.h`. Gameplay-query
//! сохраняет first matching set в unsigned ID-order, exact-count completion и
//! ordered threshold-prefix; применение результата остаётся у `CPlayer`.
//!
//! Wire сначала содержит ordered skill set, затем ordered item map. Item:
//! четыре `u32`, три C-string, фактический equipment-name count и ordered
//! имена, фактический add-item count и nested ordered property/skill maps.
//! Поля `item_count/equipment_count/add_property_count/add_skill_count`
//! передаются как сохранённые declared `u32`, а не пересчитываются; это
//! отличается от следующих за ними фактических container count-ов и сохранено
//! явно. `BTreeSet/BTreeMap` заменяют STL и сохраняют unsigned/byte order,
//! owned bytes — string lifetime. Внутренний NUL и невозможные signed counts
//! блокируют весь append до изменения destination.
//!
//! Text-loader читает `data/taozhuang.ini` как whitespace stream. Открытый
//! ресурс сначала очищает оба owner-а; missing resource оставляет прежнее
//! state и пишет оригинал GBK-log. Вложенные set/map используют `insert`: дубли
//! skill, equipment name, property, added skill и add-item прерывают загрузку
//! с отдельным log, сохраняя уже построенное partial state. Дубли item ID не
//! проверяются и оставляют первую запись — это доказанная особенность
//! не исправляемая как внутренний дефект. Rust
//! отклоняет count больше размера source, не перенося конфигурационный DoS/OOM.
//! Game decoder очищает оба owner-а до count, но duplicate records молча
//! оставляет первыми через `insert`. Полный ранее декодированный item-prefix
//! сохраняется на safe short-buffer границе; незавершённый local item не
//! публикуется. Динамические byte strings заменяют старый 1028-byte stack
//! buffer и не воспроизводят его overflow.

//! Установленный экземпляр и его потребители остаются у владельца роли.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use crate::protocol::LegacyReader;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaoZhuangAddItem {
    pub number: u32,
    pub declared_property_count: u32,
    pub properties: BTreeMap<u32, u32>,
    pub declared_skill_count: u32,
    pub skills: BTreeMap<u32, u32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TaoZhuangItem {
    pub id: u32,
    pub color: u32,
    pub declared_item_count: u32,
    pub declared_equipment_count: u32,
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub script: Vec<u8>,
    pub equipment_names: BTreeSet<Vec<u8>>,
    additions: BTreeMap<u32, TaoZhuangAddItem>,
}

impl TaoZhuangItem {
    pub fn insert_addition(&mut self, addition: TaoZhuangAddItem) -> Option<TaoZhuangAddItem> {
        self.additions.insert(addition.number, addition)
    }

    pub fn additions(&self) -> &BTreeMap<u32, TaoZhuangAddItem> {
        &self.additions
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CTaoZhuangSetup {
    skill_ids: BTreeSet<u32>,
    items: BTreeMap<u32, TaoZhuangItem>,
}

impl CTaoZhuangSetup {
    /// Читает точный World text-format из уже выбранного resource backend-а.
    pub fn read_file(
        &mut self,
        source: Option<&[u8]>,
        mut add_log_text: impl FnMut(&[u8]),
    ) -> bool {
        const MISSING_FILE: &[u8] =
            b"\xCC\xD7\xD7\xB0\xCE\xC4\xBC\xFE\xC5\xE4\xD6\xC3\xB2\xBB\xB4\xE6\xD4\xDA";
        const DUPLICATE_SKILL: &[u8] =
            b"\xCC\xD7\xD7\xB0\xBC\xBC\xC4\xDC\x49\x64\xD3\xD0\xD6\xD8\xB8\xB4";
        const DUPLICATE_EQUIPMENT: &[u8] =
            b"\xCC\xD7\xD7\xB0\xD7\xB0\xB1\xB8\xD4\xAD\xCA\xBC\xC3\xFB\xD3\xD0\xD6\xD8\xB8\xB4";
        const DUPLICATE_PROPERTY: &[u8] =
            b"\xCC\xD7\xD7\xB0\xCC\xED\xBC\xD3\xCA\xF4\xD0\xD4\xD3\xD0\xD6\xD8\xB8\xB4";
        const DUPLICATE_ADDED_SKILL: &[u8] =
            b"\xCC\xD7\xD7\xB0\xCC\xED\xBC\xD3\xBC\xBC\xC4\xDC\xD3\xD0\xD6\xD8\xB8\xB4";
        const DUPLICATE_ADDITION: &[u8] =
            b"\xCC\xD7\xD7\xB0\xBC\xFE\xCA\xFD\xB7\xD6\xC5\xE4\xD3\xD0\xD6\xD8\xB8\xB4";

        let Some(source) = source else {
            add_log_text(MISSING_FILE);
            return false;
        };
        self.clear();
        let mut input = TaoZhuangTokenStream::new(source);

        let skill_count = input.read_labeled_u32();
        if !count_fits_source(skill_count, source) {
            return false;
        }
        for _ in 0..skill_count {
            self.skill_ids.insert(input.read_u32());
        }
        if self.skill_ids.len() != skill_count as usize {
            add_log_text(DUPLICATE_SKILL);
            return false;
        }

        let item_count = input.read_labeled_u32();
        if !count_fits_source(item_count, source) {
            return false;
        }
        for _ in 0..item_count {
            let id = input.read_labeled_u32();
            let color = input.read_labeled_u32();
            let name = input.read_labeled_bytes();
            let declared_equipment_count = input.read_labeled_u32();
            input.read_bytes();
            if !count_fits_source(declared_equipment_count, source) {
                return false;
            }
            let mut equipment_names = BTreeSet::new();
            for _ in 0..declared_equipment_count {
                equipment_names.insert(input.read_bytes());
            }
            if equipment_names.len() != declared_equipment_count as usize {
                add_log_text(DUPLICATE_EQUIPMENT);
                return false;
            }

            let description = input.read_labeled_bytes();
            let script = input.read_labeled_bytes();
            let declared_item_count = input.read_labeled_u32();
            if !count_fits_source(declared_item_count, source) {
                return false;
            }
            let mut additions = BTreeMap::new();
            for _ in 0..declared_item_count {
                let number = input.read_labeled_u32();
                let declared_property_count = input.read_labeled_u32();
                if !count_fits_source(declared_property_count, source) {
                    return false;
                }
                let mut properties = BTreeMap::new();
                for _ in 0..declared_property_count {
                    input.read_bytes();
                    input.read_bytes();
                    let property_id = input.read_u32();
                    input.read_bytes();
                    let value = input.read_u32();
                    properties.entry(property_id).or_insert(value);
                }
                if properties.len() != declared_property_count as usize {
                    add_log_text(DUPLICATE_PROPERTY);
                    return false;
                }

                let declared_skill_count = input.read_labeled_u32();
                if !count_fits_source(declared_skill_count, source) {
                    return false;
                }
                let mut skills = BTreeMap::new();
                for _ in 0..declared_skill_count {
                    input.read_bytes();
                    let skill_id = input.read_u32();
                    input.read_bytes();
                    let value = input.read_u32();
                    skills.entry(skill_id).or_insert(value);
                }
                if skills.len() != declared_skill_count as usize {
                    add_log_text(DUPLICATE_ADDED_SKILL);
                    return false;
                }
                additions.entry(number).or_insert(TaoZhuangAddItem {
                    number,
                    declared_property_count,
                    properties,
                    declared_skill_count,
                    skills,
                });
            }
            if additions.len() != declared_item_count as usize {
                add_log_text(DUPLICATE_ADDITION);
                return false;
            }
            self.items.entry(id).or_insert(TaoZhuangItem {
                id,
                color,
                declared_item_count,
                declared_equipment_count,
                name,
                description,
                script,
                equipment_names,
                additions,
            });
        }
        true
    }

    pub fn clear(&mut self) {
        self.skill_ids.clear();
        self.items.clear();
    }

    pub fn insert_skill(&mut self, skill_id: u32) -> bool {
        self.skill_ids.insert(skill_id)
    }

    pub fn insert_item(&mut self, item: TaoZhuangItem) -> Option<TaoZhuangItem> {
        self.items.insert(item.id, item)
    }

    pub fn skill_ids(&self) -> &BTreeSet<u32> {
        &self.skill_ids
    }

    pub fn items(&self) -> &BTreeMap<u32, TaoZhuangItem> {
        &self.items
    }

    pub fn query_id_by_equipment_name(&self, name: &[u8]) -> Option<u32> {
        self.items
            .values()
            .find(|item| item.equipment_names.contains(name))
            .map(|item| item.id)
    }

    pub fn item(&self, id: u32) -> Option<&TaoZhuangItem> {
        self.items.get(&id)
    }

    pub fn add_byte_to_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), TaoZhuangSerializationBlock> {
        let mut payload = Vec::new();
        write_tao_zhuang_count(
            &mut payload,
            TaoZhuangCountSection::SkillIds,
            self.skill_ids.len(),
        )?;
        for skill_id in &self.skill_ids {
            payload.extend_from_slice(&skill_id.to_le_bytes());
        }

        write_tao_zhuang_count(&mut payload, TaoZhuangCountSection::Items, self.items.len())?;
        for item in self.items.values() {
            for value in [
                item.id,
                item.color,
                item.declared_item_count,
                item.declared_equipment_count,
            ] {
                payload.extend_from_slice(&value.to_le_bytes());
            }
            write_tao_zhuang_string(
                &mut payload,
                item.id,
                None,
                TaoZhuangStringField::Name,
                &item.name,
            )?;
            write_tao_zhuang_string(
                &mut payload,
                item.id,
                None,
                TaoZhuangStringField::Description,
                &item.description,
            )?;
            write_tao_zhuang_string(
                &mut payload,
                item.id,
                None,
                TaoZhuangStringField::Script,
                &item.script,
            )?;

            write_tao_zhuang_count(
                &mut payload,
                TaoZhuangCountSection::EquipmentNames { item_id: item.id },
                item.equipment_names.len(),
            )?;
            for (equipment_index, name) in item.equipment_names.iter().enumerate() {
                write_tao_zhuang_string(
                    &mut payload,
                    item.id,
                    Some(equipment_index),
                    TaoZhuangStringField::EquipmentName,
                    name,
                )?;
            }

            write_tao_zhuang_count(
                &mut payload,
                TaoZhuangCountSection::Additions { item_id: item.id },
                item.additions.len(),
            )?;
            for addition in item.additions.values() {
                payload.extend_from_slice(&addition.number.to_le_bytes());
                payload.extend_from_slice(&addition.declared_property_count.to_le_bytes());
                for (&property_id, &value) in &addition.properties {
                    payload.extend_from_slice(&property_id.to_le_bytes());
                    payload.extend_from_slice(&value.to_le_bytes());
                }
                payload.extend_from_slice(&addition.declared_skill_count.to_le_bytes());
                for (&skill_id, &value) in &addition.skills {
                    payload.extend_from_slice(&skill_id.to_le_bytes());
                    payload.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }

    /// Декодирует GameServer startup projection `DeCodeFromByte`.
    pub fn decode_from_byte(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), TaoZhuangDecodeError> {
        self.clear();

        let skill_count = read_wire_i32(source, cursor, "skill count")?;
        for _ in 0..skill_count.max(0) {
            self.skill_ids
                .insert(read_wire_u32(source, cursor, "skill ID")?);
        }

        let item_count = read_wire_i32(source, cursor, "item count")?;
        for _ in 0..item_count.max(0) {
            let id = read_wire_u32(source, cursor, "item ID")?;
            let color = read_wire_u32(source, cursor, "item color")?;
            let declared_item_count = read_wire_u32(source, cursor, "declared addition count")?;
            let declared_equipment_count =
                read_wire_u32(source, cursor, "declared equipment count")?;
            let name = read_wire_c_string(source, cursor, "item name")?;
            let description = read_wire_c_string(source, cursor, "item description")?;
            let script = read_wire_c_string(source, cursor, "item script")?;

            let equipment_count = read_wire_i32(source, cursor, "equipment name count")?;
            let mut equipment_names = BTreeSet::new();
            for _ in 0..equipment_count.max(0) {
                equipment_names.insert(read_wire_c_string(source, cursor, "equipment name")?);
            }

            let addition_count = read_wire_i32(source, cursor, "addition count")?;
            let mut additions = BTreeMap::new();
            for _ in 0..addition_count.max(0) {
                let number = read_wire_u32(source, cursor, "addition number")?;
                let declared_property_count =
                    read_wire_u32(source, cursor, "addition property count")?;
                let mut properties = BTreeMap::new();
                for _ in 0..declared_property_count {
                    let property_id = read_wire_u32(source, cursor, "property ID")?;
                    let value = read_wire_u32(source, cursor, "property value")?;
                    properties.entry(property_id).or_insert(value);
                }

                let declared_skill_count = read_wire_u32(source, cursor, "addition skill count")?;
                let mut skills = BTreeMap::new();
                for _ in 0..declared_skill_count {
                    let skill_id = read_wire_u32(source, cursor, "added skill ID")?;
                    let value = read_wire_u32(source, cursor, "added skill value")?;
                    skills.entry(skill_id).or_insert(value);
                }

                additions.entry(number).or_insert(TaoZhuangAddItem {
                    number,
                    declared_property_count,
                    properties,
                    declared_skill_count,
                    skills,
                });
            }

            self.items.entry(id).or_insert(TaoZhuangItem {
                id,
                color,
                declared_item_count,
                declared_equipment_count,
                name,
                description,
                script,
                equipment_names,
                additions,
            });
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaoZhuangCountSection {
    SkillIds,
    Items,
    EquipmentNames { item_id: u32 },
    Additions { item_id: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaoZhuangStringField {
    Name,
    Description,
    Script,
    EquipmentName,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaoZhuangSerializationBlock {
    CountOutOfRange {
        section: TaoZhuangCountSection,
        count: usize,
    },
    StringContainsNul {
        item_id: u32,
        equipment_index: Option<usize>,
        field: TaoZhuangStringField,
    },
}

impl fmt::Display for TaoZhuangSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { section, count } => write!(
                formatter,
                "CTaoZhuangSetup {section:?} содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul {
                item_id,
                equipment_index,
                field,
            } => write!(
                formatter,
                "CTaoZhuangSetup item {item_id}, equipment {equipment_index:?}: поле {field:?} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for TaoZhuangSerializationBlock {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaoZhuangDecodeError {
    pub field: &'static str,
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
}

impl fmt::Display for TaoZhuangDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "TaoZhuang snapshot, поле {} на {}: нужно {}, доступно {}",
            self.field, self.offset, self.needed, self.available
        )
    }
}

impl Error for TaoZhuangDecodeError {}

fn read_wire_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, TaoZhuangDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(|block| TaoZhuangDecodeError {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}

fn read_wire_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, TaoZhuangDecodeError> {
    LegacyReader::read_u32_from(source, cursor).map_err(|block| TaoZhuangDecodeError {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}

fn read_wire_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, TaoZhuangDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    if offset > source.len() {
        return Err(TaoZhuangDecodeError {
            field,
            offset,
            needed: 1,
            available,
        });
    }
    LegacyReader::read_c_string_from(source, cursor, available)
        .map(|value| value.to_vec())
        .map_err(|_| TaoZhuangDecodeError {
            field,
            offset,
            needed: available.saturating_add(1),
            available,
        })
}

fn write_tao_zhuang_count(
    destination: &mut Vec<u8>,
    section: TaoZhuangCountSection,
    count: usize,
) -> Result<(), TaoZhuangSerializationBlock> {
    let count = i32::try_from(count)
        .map_err(|_| TaoZhuangSerializationBlock::CountOutOfRange { section, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn write_tao_zhuang_string(
    destination: &mut Vec<u8>,
    item_id: u32,
    equipment_index: Option<usize>,
    field: TaoZhuangStringField,
    value: &[u8],
) -> Result<(), TaoZhuangSerializationBlock> {
    if value.contains(&0) {
        return Err(TaoZhuangSerializationBlock::StringContainsNul {
            item_id,
            equipment_index,
            field,
        });
    }
    destination.extend_from_slice(value);
    destination.push(0);
    Ok(())
}

fn count_fits_source(count: u32, source: &[u8]) -> bool {
    usize::try_from(count).is_ok_and(|count| count <= source.len())
}

struct TaoZhuangTokenStream<'a> {
    source: &'a [u8],
    position: usize,
    failed: bool,
}

impl<'a> TaoZhuangTokenStream<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            position: 0,
            failed: false,
        }
    }

    fn read_labeled_u32(&mut self) -> u32 {
        self.read_bytes();
        self.read_u32()
    }

    fn read_labeled_bytes(&mut self) -> Vec<u8> {
        self.read_bytes();
        self.read_bytes()
    }

    fn read_bytes(&mut self) -> Vec<u8> {
        if self.failed {
            return Vec::new();
        }
        while self
            .source
            .get(self.position)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.position += 1;
        }
        if self.position == self.source.len() {
            self.failed = true;
            return Vec::new();
        }
        let start = self.position;
        while self
            .source
            .get(self.position)
            .is_some_and(|byte| !byte.is_ascii_whitespace())
        {
            self.position += 1;
        }
        self.source[start..self.position].to_vec()
    }

    fn read_u32(&mut self) -> u32 {
        let token = self.read_bytes();
        if self.failed {
            return 0;
        }
        match parse_legacy_u32(&token) {
            Some(value) => value,
            None => {
                self.failed = true;
                0
            }
        }
    }
}

fn parse_legacy_u32(token: &[u8]) -> Option<u32> {
    let (negative, digits) = match token {
        [b'-', rest @ ..] => (true, rest),
        [b'+', rest @ ..] => (false, rest),
        _ => (false, token),
    };
    if digits.is_empty() {
        return None;
    }
    let mut value = 0u64;
    for &digit in digits {
        if !digit.is_ascii_digit() {
            return None;
        }
        value = value
            .checked_mul(10)?
            .checked_add(u64::from(digit - b'0'))?;
    }
    if negative {
        (value <= u64::from(u32::MAX) + 1).then(|| (value as u32).wrapping_neg())
    } else {
        u32::try_from(value).ok()
    }
}
