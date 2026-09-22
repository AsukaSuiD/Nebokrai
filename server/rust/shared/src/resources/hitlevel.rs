//! Общий формат таблицы HitLevel для World и Game.
//! Исходный владелец: `server/setup/hitlevelsetup.cpp/.h`.
//! Совпадающие EXE/PDB: World `LoadHitLevelSetup` VA 0x004979C0,
//! `AddToByteArray` VA 0x00497550; Game `DecordFromByteArray` VA 0x004C5CB0.
//! World очищает таблицу до открытия файла, Game — до чтения снимка.
//! Формат и безопасные границы повреждённого ввода описаны в
//! `docs/architecture/resources-and-configuration.md#таблица-hitlevel`.

use std::fmt;

use super::marker::read_to_marker;
use crate::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HitLevelEntry {
    pub level: u32,
    pub hit: u32,
    pub experience: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CHitLevelSetup {
    entries: Vec<HitLevelEntry>,
}

impl CHitLevelSetup {
    pub fn entries(&self) -> &[HitLevelEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn load_from_bytes(&mut self, source: &[u8]) -> Result<usize, HitLevelFormatError> {
        self.clear();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        while read_to_marker(&mut tokens, b"*") {
            let level = read_u32(&mut tokens, "level")?;
            let hit = read_u32(&mut tokens, "hit")?;
            let experience = read_u32(&mut tokens, "experience")?;
            self.entries.push(HitLevelEntry {
                level,
                hit,
                experience,
            });
        }
        Ok(self.entries.len())
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), HitLevelSerializeError> {
        let count = i32::try_from(self.entries.len()).map_err(|_| HitLevelSerializeError {
            count: self.entries.len(),
        })?;
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(count);
        for entry in &self.entries {
            writer.write_u32(entry.level);
            writer.write_u32(entry.hit);
            writer.write_u32(entry.experience);
        }
        Ok(())
    }

    /// Очищает вектор до чтения счётчика и сохраняет только полные записи.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<usize, HitLevelDecodeError> {
        self.entries.clear();
        let count = read_wire_i32(source, cursor)?;
        for _ in 0..count.max(0) {
            self.entries.push(HitLevelEntry {
                level: read_wire_u32(source, cursor)?,
                hit: read_wire_u32(source, cursor)?,
                experience: read_wire_u32(source, cursor)?,
            });
        }
        Ok(self.entries.len())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HitLevelFormatError {
    UnexpectedEnd { field: &'static str },
    InvalidUnsignedLong { field: &'static str, token: Vec<u8> },
}

impl fmt::Display for HitLevelFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { field } => {
                write!(formatter, "после маркера отсутствует поле {field}")
            }
            Self::InvalidUnsignedLong { field, token } => write!(
                formatter,
                "поле {field} не является unsigned long: {}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl std::error::Error for HitLevelFormatError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HitLevelSerializeError {
    pub count: usize,
}

impl fmt::Display for HitLevelSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HitLevelSetup содержит {} записей вне signed 32-битного диапазона",
            self.count
        )
    }
}

impl std::error::Error for HitLevelSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HitLevelDecodeError {
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
}

impl fmt::Display for HitLevelDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HitLevel snapshot обрывается на {}: нужно {}, доступно {}",
            self.offset, self.needed, self.available
        )
    }
}

impl std::error::Error for HitLevelDecodeError {}

fn read_u32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<u32, HitLevelFormatError> {
    let token = tokens
        .next()
        .ok_or(HitLevelFormatError::UnexpectedEnd { field })?;
    let text =
        std::str::from_utf8(token).map_err(|_| HitLevelFormatError::InvalidUnsignedLong {
            field,
            token: token.to_vec(),
        })?;
    text.parse::<u32>()
        .map_err(|_| HitLevelFormatError::InvalidUnsignedLong {
            field,
            token: token.to_vec(),
        })
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, HitLevelDecodeError> {
    read_wire(source, cursor, |reader| reader.read_i32())
}

fn read_wire_u32(source: &[u8], cursor: &mut usize) -> Result<u32, HitLevelDecodeError> {
    read_wire(source, cursor, |reader| reader.read_u32())
}

fn read_wire<Value>(
    source: &[u8],
    cursor: &mut usize,
    read: impl FnOnce(&mut LegacyReader<'_>) -> Result<Value, LegacyReadBlock>,
) -> Result<Value, HitLevelDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(map_read_block)?;
    let value = read(&mut reader).map_err(map_read_block)?;
    *cursor = reader.position();
    Ok(value)
}

fn map_read_block(block: LegacyReadBlock) -> HitLevelDecodeError {
    HitLevelDecodeError {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
