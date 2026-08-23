//! Две таблицы преобразования экипировки исторического Miracle.
//!
//! World `EquipmentComposeList::LoadList/AddToByteArray`
//! —; Game decoder и lookup queries ниже
//! остаются. Точная пара:
//! Исходный владелец PDB:
//!
//! Wire состоит из двух последовательных ordered map: для каждой сначала
//! signed count, затем пары `u32 source + u32 target`. Loader и Game decoder
//! использовали `map::insert`, поэтому duplicate source сохраняет первое
//! значение. Исходная функция всегда возвращала `false` даже после успешной
//! записи, но reconnect caller игнорировал результат; Rust сообщает только
//! реальную ошибку представимости count и не переносит бессодержательный flag.
//! `BTreeMap` заменяет MSVC tree без изменения unsigned key-order.
//! `LoadList` очищает обе таблицы до попытки чтения, ищет два точных маркера
//! `#`, пропускает следующий label и читает signed count с парами signed
//! `long`, сохраняя их 32-битный шаблон как unsigned key/value. Exact
//! возвращает `0` только при ошибке открытия и `1`
//! после любого открытого stream, даже если секции неполны; это legacy-
//! различие между доступностью ресурса и полнотой данных сохранено.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::readwrite::read_to;

/// Safe owner исходных static `m_mapList1` и `m_mapList2`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EquipmentComposeList {
    first: BTreeMap<u32, u32>,
    second: BTreeMap<u32, u32>,
}

impl EquipmentComposeList {
 /// Загружает две ordered map из уже выбранного resource backend-а.
    pub(crate) fn load_list(&mut self, source: Option<&[u8]>) -> bool {
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

    pub(crate) fn insert_first(&mut self, source: u32, target: u32) -> bool {
        insert_first_wins(&mut self.first, source, target)
    }

    pub(crate) fn insert_second(&mut self, source: u32, target: u32) -> bool {
        insert_first_wins(&mut self.second, source, target)
    }

    pub(crate) fn clear(&mut self) {
        self.first.clear();
        self.second.clear();
    }

 /// Дописывает оригинал `map1 + map2` wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), EquipmentComposeSerializeError> {
        write_map(destination, &self.first, EquipmentComposeSection::First)?;
        write_map(destination, &self.second, EquipmentComposeSection::Second)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentComposeSection {
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
pub(crate) struct EquipmentComposeSerializeError {
    pub(crate) section: EquipmentComposeSection,
    pub(crate) count: usize,
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

fn insert_first_wins(values: &mut BTreeMap<u32, u32>, source: u32, target: u32) -> bool {
    if values.contains_key(&source) {
        return false;
    }
    values.insert(source, target);
    true
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

// и lookup queries, а не как Rust-реализация.
