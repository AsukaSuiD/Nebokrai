//! Две static таблицы преобразования экипировки исторического Miracle.
//!
//! `LoadList/AddToByteArray` подтверждены точной парой
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, а
//! `DecordFromByteArray/GetFirstCompose/GetSecondCompose` — точной парой
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`. Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp`.
//!
//! Wire состоит из двух последовательных ordered map: для каждой сначала
//! signed count, затем пары `u32 source + u32 target`. Loader и Game decoder
//! использовали `map::insert`, поэтому duplicate source сохраняет первое
//! значение. Game decoder сначала очищает обе таблицы и сохраняет полностью
//! прочитанный prefix; исходная функция всегда возвращала `false` даже после
//! успешной записи, но reconnect caller игнорировал результат, поэтому Rust
//! возвращает содержательный report. Отрицательный wire-count не создаётся
//! парным World owner-ом; вместо legacy unbounded-read он трактуется как пустая
//! секция, как и в достигнутых соседних snapshot decoder-ах.
//! `BTreeMap` заменяет MSVC tree без изменения unsigned key-order.
//! `LoadList` очищает обе таблицы до попытки чтения, ищет два точных маркера
//! `#`, пропускает следующий label и читает signed count с парами signed
//! `long`, сохраняя их 32-битный шаблон как unsigned key/value. Exact
//! возвращает `0` только при ошибке открытия и `1`
//! после любого открытого stream, даже если секции неполны; это legacy-
//! различие между доступностью ресурса и полнотой данных сохранено.
//! Установленный экземпляр и его потребители остаются у владельца роли.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::protocol::LegacyReader;

use crate::resources::read_to_marker as read_to;

/// Safe owner исходных static `m_mapList1` и `m_mapList2`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EquipmentComposeList {
    first: BTreeMap<u32, u32>,
    second: BTreeMap<u32, u32>,
}

impl EquipmentComposeList {
    /// Загружает две ordered map из уже выбранного resource backend-а.
    pub fn load_list(&mut self, source: Option<&[u8]>) -> bool {
        self.clear();
        let Some(source) = source else {
            return false;
        };

        let mut tokens = source
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty());
        let mut stream_failed = false;
        load_section(&mut tokens, &mut stream_failed, &mut self.first);
        load_section(&mut tokens, &mut stream_failed, &mut self.second);
        true
    }

    pub fn insert_first(&mut self, source: u32, target: u32) -> bool {
        insert_first_wins(&mut self.first, source, target)
    }

    pub fn insert_second(&mut self, source: u32, target: u32) -> bool {
        insert_first_wins(&mut self.second, source, target)
    }

    pub fn clear(&mut self) {
        self.first.clear();
        self.second.clear();
    }

    /// Дописывает оригинал `map1 + map2` wire.
    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), EquipmentComposeSerializeError> {
        write_map(destination, &self.first, EquipmentComposeSection::First)?;
        write_map(destination, &self.second, EquipmentComposeSection::Second)?;
        Ok(())
    }

    /// Повторяет `GetFirstCompose`: отсутствующий source даёт нулевой TID.
    pub fn get_first_compose(&self, source: u32) -> u32 {
        self.first.get(&source).copied().unwrap_or(0)
    }

    /// Повторяет `GetSecondCompose`: отсутствующий source даёт нулевой TID.
    pub fn get_second_compose(&self, source: u32) -> u32 {
        self.second.get(&source).copied().unwrap_or(0)
    }

    /// Декодирует GameServer startup snapshot, сохраняя полностью прочитанный
    /// prefix при безопасной short-buffer границе.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), EquipmentComposeDecodeError> {
        self.clear();
        decode_map(
            source,
            cursor,
            EquipmentComposeSection::First,
            &mut self.first,
        )?;
        decode_map(
            source,
            cursor,
            EquipmentComposeSection::Second,
            &mut self.second,
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentComposeSection {
    First,
    Second,
}

impl fmt::Display for EquipmentComposeSection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::First => "первая equipment-compose таблица",
            Self::Second => "вторая equipment-compose таблица",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EquipmentComposeSerializeError {
    pub section: EquipmentComposeSection,
    pub count: usize,
}

impl fmt::Display for EquipmentComposeSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} содержит {} записей вне signed 32-битного диапазона",
            self.section, self.count
        )
    }
}

impl Error for EquipmentComposeSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EquipmentComposeDecodeError {
    pub section: EquipmentComposeSection,
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
}

impl fmt::Display for EquipmentComposeDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} snapshot обрывается на {}: нужно {}, доступно {}",
            self.section, self.offset, self.needed, self.available
        )
    }
}

impl Error for EquipmentComposeDecodeError {}

fn insert_first_wins(values: &mut BTreeMap<u32, u32>, source: u32, target: u32) -> bool {
    if values.contains_key(&source) {
        return false;
    }
    values.insert(source, target);
    true
}

fn decode_map(
    source: &[u8],
    cursor: &mut usize,
    section: EquipmentComposeSection,
    destination: &mut BTreeMap<u32, u32>,
) -> Result<(), EquipmentComposeDecodeError> {
    let count = read_wire_i32(source, cursor, section)?;
    for _ in 0..count.max(0) {
        let source_id = read_wire_u32(source, cursor, section)?;
        let target_id = read_wire_u32(source, cursor, section)?;
        insert_first_wins(destination, source_id, target_id);
    }
    Ok(())
}

fn read_wire_i32(
    source: &[u8],
    cursor: &mut usize,
    section: EquipmentComposeSection,
) -> Result<i32, EquipmentComposeDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(|block| EquipmentComposeDecodeError {
        section,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}

fn read_wire_u32(
    source: &[u8],
    cursor: &mut usize,
    section: EquipmentComposeSection,
) -> Result<u32, EquipmentComposeDecodeError> {
    LegacyReader::read_u32_from(source, cursor).map_err(|block| EquipmentComposeDecodeError {
        section,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}

fn load_section<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    stream_failed: &mut bool,
    destination: &mut BTreeMap<u32, u32>,
) {
    if *stream_failed || !read_to(tokens, b"#") {
        return;
    }
    if tokens.next().is_none() {
        *stream_failed = true;
        return;
    }
    let count = read_formatted_long(tokens, stream_failed);
    if count <= 0 {
        return;
    }
    for _ in 0..count {
        let source = read_formatted_long(tokens, stream_failed) as u32;
        let target = read_formatted_long(tokens, stream_failed) as u32;
        insert_first_wins(destination, source, target);
    }
}

fn read_formatted_long<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    stream_failed: &mut bool,
) -> i32 {
    if *stream_failed {
        return 0;
    }
    let Some(token) = tokens.next() else {
        *stream_failed = true;
        return 0;
    };
    let Some(value) = std::str::from_utf8(token)
        .ok()
        .and_then(|token| token.parse::<i32>().ok())
    else {
        *stream_failed = true;
        return 0;
    };
    value
}

fn write_map(
    destination: &mut Vec<u8>,
    values: &BTreeMap<u32, u32>,
    section: EquipmentComposeSection,
) -> Result<(), EquipmentComposeSerializeError> {
    let count = i32::try_from(values.len()).map_err(|_| EquipmentComposeSerializeError {
        section,
        count: values.len(),
    })?;
    destination.extend_from_slice(&count.to_le_bytes());
    for (&source, &target) in values {
        destination.extend_from_slice(&source.to_le_bytes());
        destination.extend_from_slice(&target.to_le_bytes());
    }
    Ok(())
}
