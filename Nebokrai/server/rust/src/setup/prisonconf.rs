//! Конфигурация тюрьмы исторического Miracle.
//!
//! Контракт World `load_conf` и `AddToByteArray`
//!:; Game decoder и singleton plumbing не входят в этот owner и остаются
//! Точная пара:
//! Исходный владелец PDB:
//!
//! Loader сначала очищает signed-char ordered map, но сохраняет прежний
//! `_pk_val_enter` при ошибке открытия. Конструктор EXE его не инициализировал,
//! поэтому safe owner выражает ещё не загруженное значение через `Option`, а
//! serializer возвращает typed boundary вместо выдуманного нуля.
//!
//! Каждая `#`-запись читает signed ID, region, два signed short и одно поле
//! `char`. Реальный `prisonconf.ini` содержит направление `-1`, однако
//! formatted extraction в `char` берёт только первый символ: observable wire
//! содержит ASCII `'-'` (`0x2D`), не числовой `0xFF`. Этот quirk сохранён
//! явно. ID сужается до младшего signed byte; duplicate key заменяется.
//!
//! Wire: `i32 pk_threshold`, signed count, затем ordered записи
//! `i8 country + i32 region + i16 x + i16 y + i8 direction` — десять байт без
//! трёх padding-байт исходного 12-байтного `PrisonParam`. `BTreeMap<i8, _>`,
//! `std::fs` и `Drop` заменяют только MSVC tree/CRFile plumbing.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::public::readwrite::read_to;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PrisonParam {
    pub(crate) region: i32,
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) direction: i8,
}

/// Safe owner исходного singleton state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PrisonConf {
    pk_value_enter: Option<i32>,
    prison_params: BTreeMap<i8, PrisonParam>,
}

impl PrisonConf {
 /// Очищает map до открытия, не назначая отсутствующий constructor scalar.
    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, PrisonConfFileLoadError> {
        self.prison_params.clear();
        let source = std::fs::read(path).map_err(PrisonConfFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(PrisonConfFileLoadError::Format)
    }

    pub(crate) fn load_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<usize, PrisonConfFormatError> {
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

    pub(crate) fn get_param(&self, country: i8) -> Option<&PrisonParam> {
        self.prison_params.get(&country)
    }

 /// Missing-file ветвь loader-а очищает только map, сохраняя прежний scalar.
    pub(crate) fn clear_prison_params(&mut self) {
        self.prison_params.clear();
    }

 /// Дописывает оригинал compact wire без C++ struct padding.
    pub(crate) fn add_to_byte_array(
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PrisonConfFormatError {
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
pub(crate) enum PrisonConfFileLoadError {
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
pub(crate) enum PrisonConfSerializeError {
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

// singleton и оставшихся call-site деталей, а не как Rust-реализация.
