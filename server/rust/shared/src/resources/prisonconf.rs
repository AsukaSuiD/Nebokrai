//! Тюремная конфигурация `PrisonConf` из WorldServer/GameServer.
//! Контракт подтверждён точными `Nworldserver.exe + WorldServer.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/setup/prisonconf.h/.cpp`.
//!
//! Loader очищает signed-byte map, но при ошибке открытия сохраняет прежний
//! PK threshold; до первой загрузки он остаётся `None`. Direction читается как
//! `char`, поэтому token `-1` даёт ASCII `'-'`, а не `0xFF`.
//!
//! Wire пишет threshold, signed count и десятибайтные records без padding.
//! Duplicate country заменяет значение; `BTreeMap<i8, _>` сохраняет порядок.
//! Game decoder сначала очищает map, затем публикует threshold и только полные
//! records. Safe short-buffer сохраняет этот подтверждённый partial state;
//! безразмерному C++ pointer с неизвестным UB Rust побочных эффектов не задаёт.
//! `GetParam` использует изменяющий map::operator[]: в GameServer
//! `prison_check` вызывает его по 0x004D243A, тело находится по 0x004D0B60.
//! Отсутствующий signed-byte ключ создаёт PrisonParam с нулевыми region/x/y/d:
//! 0x004D0BA1..0x004D0BC8 обнуляют три DWORD значения перед вставкой,
//! а 0x00431C42..0x00431C5D копируют пару в узел без изменения этих полей.
//! BTreeMap::entry().or_default() сохраняет вставку и последующее wire-наличие
//! записи; существующая страна не перезаписывается. Неизвестный до загрузки
//! PK-порог остаётся Option и не заменяется предполагаемым нулём.
//! Установленный экземпляр и его потребители остаются у владельца роли.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::protocol::LegacyReader;
use crate::resources::read_to_marker as read_to;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PrisonParam {
    pub region: i32,
    pub x: i16,
    pub y: i16,
    pub direction: i8,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PrisonConf {
    pk_value_enter: Option<i32>,
    prison_params: BTreeMap<i8, PrisonParam>,
}

impl PrisonConf {
    pub fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, PrisonConfFileLoadError> {
        self.prison_params.clear();
        let source = std::fs::read(path).map_err(PrisonConfFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(PrisonConfFileLoadError::Format)
    }

    pub fn load_from_bytes(&mut self, source: &[u8]) -> Result<usize, PrisonConfFormatError> {
        self.prison_params.clear();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        let _ignored_name = next_token(&mut tokens, "имя PK-порога")?;
        self.pk_value_enter = Some(read_i32(&mut tokens, "PK-порог входа")?);

        let mut applied = 0;
        while read_to(&mut tokens, b"#") {
            let country = read_i32(&mut tokens, "ID страны")? as i8;
            let region = read_i32(&mut tokens, "регион тюрьмы")?;
            let x = read_i16(&mut tokens, "координата X")?;
            let y = read_i16(&mut tokens, "координата Y")?;
            let direction_token = next_token(&mut tokens, "направление")?;
            let direction =
                direction_token
                    .first()
                    .copied()
                    .ok_or(PrisonConfFormatError::UnexpectedEnd {
                        field: "направление",
                    })? as i8;
            self.prison_params.insert(
                country,
                PrisonParam {
                    region,
                    x,
                    y,
                    direction,
                },
            );
            applied += 1;
        }
        Ok(applied)
    }

    pub const fn pk_value_enter(&self) -> Option<i32> {
        self.pk_value_enter
    }

    pub fn get_param(&mut self, country: i8) -> &PrisonParam {
        self.prison_params.entry(country).or_default()
    }

    pub fn prison_params(&self) -> &BTreeMap<i8, PrisonParam> {
        &self.prison_params
    }

    pub fn clear_prison_params(&mut self) {
        self.prison_params.clear();
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PrisonConfSerializeError> {
        let pk_value_enter = self
            .pk_value_enter
            .ok_or(PrisonConfSerializeError::PkValueUnavailable)?;
        let count = i32::try_from(self.prison_params.len()).map_err(|_| {
            PrisonConfSerializeError::EntryCount {
                count: self.prison_params.len(),
            }
        })?;
        destination.extend_from_slice(&pk_value_enter.to_le_bytes());
        destination.extend_from_slice(&count.to_le_bytes());
        for (&country, param) in &self.prison_params {
            destination.push(country as u8);
            destination.extend_from_slice(&param.region.to_le_bytes());
            destination.extend_from_slice(&param.x.to_le_bytes());
            destination.extend_from_slice(&param.y.to_le_bytes());
            destination.push(param.direction as u8);
        }
        Ok(())
    }

    /// Воспроизводит `PrisonConf::DecordFromByteArray` GameServer.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<usize, PrisonConfDecodeError> {
        self.prison_params.clear();
        self.pk_value_enter = Some(read_wire_i32(source, cursor)?);
        let count = read_wire_i32(source, cursor)?;
        for _ in 0..count.max(0) {
            let country = read_wire_i8(source, cursor)?;
            let region = read_wire_i32(source, cursor)?;
            let x = read_wire_i16(source, cursor)?;
            let y = read_wire_i16(source, cursor)?;
            let direction = read_wire_i8(source, cursor)?;
            self.prison_params.insert(
                country,
                PrisonParam {
                    region,
                    x,
                    y,
                    direction,
                },
            );
        }
        Ok(self.prison_params.len())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrisonConfFormatError {
    UnexpectedEnd { field: &'static str },
    InvalidLong { field: &'static str, token: Vec<u8> },
}

impl fmt::Display for PrisonConfFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { field } => write!(formatter, "отсутствует поле {field}"),
            Self::InvalidLong { field, token } => write!(
                formatter,
                "поле {field} не является подходящим signed long: {}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for PrisonConfFormatError {}

#[derive(Debug)]
pub enum PrisonConfFileLoadError {
    Io(std::io::Error),
    Format(PrisonConfFormatError),
}

impl fmt::Display for PrisonConfFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for PrisonConfFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrisonConfSerializeError {
    PkValueUnavailable,
    EntryCount { count: usize },
}

impl fmt::Display for PrisonConfSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PkValueUnavailable => formatter.write_str("PK-порог PrisonConf ещё не загружен"),
            Self::EntryCount { count } => write!(
                formatter,
                "PrisonConf содержит {count} записей вне signed 32-битного диапазона"
            ),
        }
    }
}

impl Error for PrisonConfSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrisonConfDecodeError {
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
}

impl fmt::Display for PrisonConfDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "PrisonConf snapshot обрывается на {}: нужно {}, доступно {}",
            self.offset, self.needed, self.available
        )
    }
}

impl Error for PrisonConfDecodeError {}

fn next_token<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<&'source [u8], PrisonConfFormatError> {
    tokens
        .next()
        .ok_or(PrisonConfFormatError::UnexpectedEnd { field })
}

fn read_i32<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<i32, PrisonConfFormatError> {
    let token = next_token(tokens, field)?;
    parse_signed(token, field)
}

fn read_i16<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<i16, PrisonConfFormatError> {
    let token = next_token(tokens, field)?;
    let value = parse_signed(token, field)?;
    i16::try_from(value).map_err(|_| invalid_long(field, token))
}

fn parse_signed(token: &[u8], field: &'static str) -> Result<i32, PrisonConfFormatError> {
    let text = std::str::from_utf8(token).map_err(|_| invalid_long(field, token))?;
    text.parse::<i32>().map_err(|_| invalid_long(field, token))
}

fn invalid_long(field: &'static str, token: &[u8]) -> PrisonConfFormatError {
    PrisonConfFormatError::InvalidLong {
        field,
        token: token.to_vec(),
    }
}

fn read_wire_i8(source: &[u8], cursor: &mut usize) -> Result<i8, PrisonConfDecodeError> {
    LegacyReader::read_i8_from(source, cursor).map_err(map_read_block)
}

fn read_wire_i16(source: &[u8], cursor: &mut usize) -> Result<i16, PrisonConfDecodeError> {
    LegacyReader::read_i16_from(source, cursor).map_err(map_read_block)
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, PrisonConfDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(map_read_block)
}

fn map_read_block(block: crate::protocol::LegacyReadBlock) -> PrisonConfDecodeError {
    PrisonConfDecodeError {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
