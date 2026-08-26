//! Таблица level/hit/experience Miracle.
//!
//! Контракт World loader/serializer и Game decoder подтверждён точными
//! `worldserver.exe + worldserver.pdb` и `gameserver.exe + GameServer.pdb`;
//! исходный владелец `setup/hitlevelsetup.cpp`.
//!
//! Отсутствие
//! файла явно ставит `AL=0`, любой открытый файл после просмотра токенов — `AL=1`,
//! в том числе файл без единого `*`. Поэтому пустая таблица является успешным
//! состоянием.
//! Прежний вектор полностью очищается до открытия файла. Владелец значений сохраняет
//! этот переход с предварительной очисткой и при ошибке файла остаётся пустым.
//!
//! Каждая запись — ровно три последовательных `u32` с младшим байтом первым;
//! формат состоит из знакового 32-битного счётчика и `count × 0x0C` байт. `Vec`
//! заменяет старый статический вектор, `std::fs` — `CRFile`, а общий `read_to`
//! сохраняет просмотр токенов по пробельным символам, точный `*` и терминатор
//! `<end>`. Повреждённое число исходно могло протащить неинициализированные байты
//! стека; Rust вместо этого возвращает типизированную ошибку формата, оставляя
//! только уже полностью прочитанные записи. Это устраняет внутреннее неопределённое
//! поведение и не назначает ему семантику двоичного формата.

use std::fmt;
use std::path::Path;
use thiserror::Error;

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::public::readwrite::read_to;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct HitLevelEntry {
    pub(crate) level: u32,
    pub(crate) hit: u32,
    pub(crate) experience: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CHitLevelSetup {
    entries: Vec<HitLevelEntry>,
}

impl CHitLevelSetup {
    pub(crate) fn entries(&self) -> &[HitLevelEntry] {
        &self.entries
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, HitLevelFileLoadError> {
        self.clear();
        let source = std::fs::read(path).map_err(HitLevelFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(HitLevelFileLoadError::Format)
    }

    pub(crate) fn load_from_bytes(&mut self, source: &[u8]) -> Result<usize, HitLevelFormatError> {
        self.clear();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        while read_to(&mut tokens, b"*") {
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

    pub(crate) fn add_to_byte_array(
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
    pub(crate) fn decord_from_byte_array(
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
pub(crate) enum HitLevelFormatError {
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

#[derive(Debug, Error)]
pub(crate) enum HitLevelFileLoadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Format(#[from] HitLevelFormatError),
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("HitLevelSetup содержит {count} записей вне signed 32-битного диапазона")]
pub(crate) struct HitLevelSerializeError {
    pub(crate) count: usize,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("HitLevel snapshot обрывается на {offset}: нужно {needed}, доступно {available}")]
pub(crate) struct HitLevelDecodeError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

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
    HitLevelDecodeError { offset: block.offset, needed: block.needed, available: block.available }
}
