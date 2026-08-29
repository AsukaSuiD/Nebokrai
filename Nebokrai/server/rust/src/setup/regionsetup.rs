//! Ограничения регионов `CRegionSetup` из WorldServer/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner `setup/regionsetup.cpp`.
//!
//! Двоичный формат содержит знаковый счётчик и упорядоченные 12-байтные записи:
//! ID региона, минимальный уровень и требуемый вклад. Ключ карты задаёт знаковый
//! порядок, но отдельно не сериализуется. `BTreeMap<i32, _>` заменяет дерево
//! MSVC. Семантика результата загрузчика файла не подтверждена, поэтому этот
//! владелец предоставляет только двоичный формат.

use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionSetupEntry {
    pub(crate) id: i32,
    pub(crate) can_enter_level: i32,
    pub(crate) required_contribute: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CRegionSetup {
    entries: BTreeMap<i32, RegionSetupEntry>,
}

impl CRegionSetup {
    pub(crate) fn insert(&mut self, entry: RegionSetupEntry) -> Option<RegionSetupEntry> {
        self.entries.insert(entry.id, entry)
    }

    pub(crate) fn entries(&self) -> &BTreeMap<i32, RegionSetupEntry> {
        &self.entries
    }

    pub(crate) fn get(&self, region_id: i32) -> Option<RegionSetupEntry> {
        self.entries.get(&region_id).copied()
    }

    /// Читает оригинал World grammar: поиск маркера `#`, затем три signed long.
    /// Owner очищается до чтения; при некорректной записи сохраняется уже
    /// прочитанный префикс, а неизвестные значения остаются пустыми.
    pub(crate) fn load_from_bytes(&mut self, source: &[u8]) -> Result<usize, RegionSetupLoadError> {
        self.entries.clear();
        let tokens: Vec<&[u8]> = source
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty())
            .collect();
        let mut next = 0;
        let mut loaded = 0;
        while let Some(relative) = tokens[next..].iter().position(|token| *token == b"#") {
            next += relative + 1;
            let read = |offset: usize, field| {
                let token = tokens
                    .get(next + offset)
                    .ok_or(RegionSetupLoadError::Missing {
                        record: loaded,
                        field,
                    })?;
                std::str::from_utf8(token)
                    .ok()
                    .and_then(|text| text.parse::<i32>().ok())
                    .ok_or_else(|| RegionSetupLoadError::Invalid {
                        record: loaded,
                        field,
                        token: token.to_vec(),
                    })
            };
            let entry = RegionSetupEntry {
                id: read(0, "ID")?,
                can_enter_level: read(1, "уровень")?,
                required_contribute: read(2, "вклад")?,
            };
            self.entries.insert(entry.id, entry);
            loaded += 1;
            next += 3;
        }
        Ok(loaded)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), RegionSetupSerializeError> {
        let count = i32::try_from(self.entries.len()).map_err(|_| RegionSetupSerializeError {
            count: self.entries.len(),
        })?;
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(count);
        for entry in self.entries.values() {
            writer.write_i32(entry.id);
            writer.write_i32(entry.can_enter_level);
            writer.write_i32(entry.required_contribute);
        }
        Ok(())
    }

    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<usize, RegionSetupDecodeError> {
        self.entries.clear();
        let count = read_wire_i32(source, cursor)?;
        for _ in 0..count.max(0) {
            let entry = RegionSetupEntry {
                id: read_wire_i32(source, cursor)?,
                can_enter_level: read_wire_i32(source, cursor)?,
                required_contribute: read_wire_i32(source, cursor)?,
            };
            self.entries.insert(entry.id, entry);
        }
        Ok(self.entries.len())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("RegionSetup содержит {count} записей вне signed 32-битного диапазона")]
pub(crate) struct RegionSetupSerializeError {
    pub(crate) count: usize,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("RegionSetup snapshot обрывается на {offset}: нужно {needed}, доступно {available}")]
pub(crate) struct RegionSetupDecodeError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionSetupLoadError {
    Missing {
        record: usize,
        field: &'static str,
    },
    Invalid {
        record: usize,
        field: &'static str,
        token: Vec<u8>,
    },
}

impl fmt::Display for RegionSetupLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { record, field } => {
                write!(
                    formatter,
                    "RegionSetup: запись {record}, отсутствует поле {field}"
                )
            }
            Self::Invalid {
                record,
                field,
                token,
            } => write!(
                formatter,
                "RegionSetup: запись {record}, поле {field} содержит нецелое значение {:?}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl std::error::Error for RegionSetupLoadError {}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, RegionSetupDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(map_read_block)?;
    let value = reader.read_i32().map_err(map_read_block)?;
    *cursor = reader.position();
    Ok(value)
}

fn map_read_block(block: LegacyReadBlock) -> RegionSetupDecodeError {
    RegionSetupDecodeError {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
