//! Таблица level/hit/experience Miracle.
//!
//! Контракт World loader/serializer и Game decoder подтверждён точными
//! `worldserver.exe + worldserver.pdb` и `gameserver.exe + GameServer.pdb`;
//! исходный owner `setup/hitlevelsetup.cpp`.
//!
//! Отсутствие
//! файла явно ставит `AL=0`, любой открытый файл после token-scan — `AL=1`,
//! в том числе файл без единого `*`. Поэтому пустая таблица является успешным
//! состоянием.
//! Прежний vector полностью очищается до открытия файла. Value-owner сохраняет
//! этот clear-first
//! state transition и при ошибке файла остаётся пустым.
//!
//! Каждая запись — ровно три consecutive little-endian `u32`; wire состоит
//! из signed 32-битного count и `count × 0x0C` байт. `Vec` заменяет старый
//! static vector, `std::fs` — `CRFile`, а общий исходный `read_to` сохраняет
//! whitespace token-scan, точный `*` и терминатор `<end>`. Повреждённое число
//! исходно могло протащить неинициализированные stack-байты; Rust вместо этого
//! возвращает typed format error, оставляя только уже полностью прочитанные
//! записи. Это устраняет внутренний UB и не назначает ему wire-семантику.

use std::error::Error;
use std::fmt;
use std::path::Path;

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
        destination.extend_from_slice(&count.to_le_bytes());
        for entry in &self.entries {
            destination.extend_from_slice(&entry.level.to_le_bytes());
            destination.extend_from_slice(&entry.hit.to_le_bytes());
            destination.extend_from_slice(&entry.experience.to_le_bytes());
        }
        Ok(())
    }

    /// Очищает vector до count и сохраняет только полные records.
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

impl Error for HitLevelFormatError {}

#[derive(Debug)]
pub(crate) enum HitLevelFileLoadError {
    Io(std::io::Error),
    Format(HitLevelFormatError),
}

impl fmt::Display for HitLevelFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for HitLevelFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HitLevelSerializeError {
    pub(crate) count: usize,
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

impl Error for HitLevelSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HitLevelDecodeError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
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

impl Error for HitLevelDecodeError {}

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
    Ok(i32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_u32(source: &[u8], cursor: &mut usize) -> Result<u32, HitLevelDecodeError> {
    Ok(u32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], HitLevelDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(HitLevelDecodeError {
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes
        .try_into()
        .expect("размер HitLevel scalar уже проверен"))
}
