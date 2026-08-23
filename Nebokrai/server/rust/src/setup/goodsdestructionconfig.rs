//! Уничтожение предметов `CGoodsDestroySetup` из WorldServer, подтверждённое
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Wire пишет `u32 enabled`, signed counts, ordered `u16` goods types и
//! original-name C-строки; повторы значимы. Loader очищает оба vectors, но при
//! ошибке открытия сохраняет enabled. После `#` идут `* label u16`, после
//! первого `<end>` — независимые `+ name` records. Файл без `#` успешен с
//! пустыми vectors и прежним enabled.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;

use crate::public::readwrite::read_to;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsDestroySetup {
    enabled: bool,
    goods_types: Vec<u16>,
    original_names: Vec<Vec<u8>>,
}

impl GoodsDestroySetup {
    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(crate) fn push_goods_type(&mut self, goods_type: u16) {
        self.goods_types.push(goods_type);
    }

    pub(crate) fn push_original_name(&mut self, original_name: Vec<u8>) {
        self.original_names.push(original_name);
    }

    pub(crate) fn clear_lists(&mut self) {
        self.goods_types.clear();
        self.original_names.clear();
    }

    pub(crate) fn load_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<GoodsDestroyLoadReport, GoodsDestroyFormatError> {
        self.clear_lists();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        if !read_to(&mut tokens, b"#") {
            return Ok(GoodsDestroyLoadReport::default());
        }

        let _label = next_token(&mut tokens, "метка enabled")?;
        self.enabled = read_legacy_bool(&mut tokens, "enabled")?;
        let mut report = GoodsDestroyLoadReport::default();
        while read_to(&mut tokens, b"*") {
            let _label = next_token(&mut tokens, "метка типа предмета")?;
            self.goods_types
                .push(read_u16(&mut tokens, "тип уничтожаемого предмета")?);
            report.goods_types += 1;
        }
        while read_to(&mut tokens, b"+") {
            self.original_names
                .push(next_token(&mut tokens, "исходное имя предмета")?.to_vec());
            report.original_names += 1;
        }
        Ok(report)
    }

    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<GoodsDestroyLoadReport, GoodsDestroyFileLoadError> {
        self.clear_lists();
        let source = std::fs::read(path).map_err(GoodsDestroyFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(GoodsDestroyFileLoadError::Format)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), GoodsDestroySerializeError> {
        destination.extend_from_slice(&u32::from(self.enabled).to_le_bytes());
        write_count(
            destination,
            self.goods_types.len(),
            GoodsDestroyList::GoodsTypes,
        )?;
        for &goods_type in &self.goods_types {
            destination.extend_from_slice(&goods_type.to_le_bytes());
        }

        write_count(
            destination,
            self.original_names.len(),
            GoodsDestroyList::OriginalNames,
        )?;
        for (name_index, original_name) in self.original_names.iter().enumerate() {
            if original_name.contains(&0) {
                return Err(GoodsDestroySerializeError::NameContainsNul { name_index });
            }
            destination.extend_from_slice(original_name);
            destination.push(0);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsDestroyLoadReport {
    pub(crate) goods_types: usize,
    pub(crate) original_names: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GoodsDestroyFormatError {
    UnexpectedEnd { field: &'static str },
    InvalidBoolean { token: Vec<u8> },
    InvalidGoodsType { token: Vec<u8> },
}

impl fmt::Display for GoodsDestroyFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { field } => write!(formatter, "отсутствует поле {field}"),
            Self::InvalidBoolean { token } => write!(
                formatter,
                "enabled не является legacy numeric bool: {}",
                String::from_utf8_lossy(token)
            ),
            Self::InvalidGoodsType { token } => write!(
                formatter,
                "тип предмета не является unsigned short: {}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for GoodsDestroyFormatError {}

#[derive(Debug)]
pub(crate) enum GoodsDestroyFileLoadError {
    Io(io::Error),
    Format(GoodsDestroyFormatError),
}

impl fmt::Display for GoodsDestroyFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for GoodsDestroyFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

fn next_token<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<&'source [u8], GoodsDestroyFormatError> {
    tokens
        .next()
        .ok_or(GoodsDestroyFormatError::UnexpectedEnd { field })
}

fn read_legacy_bool<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<bool, GoodsDestroyFormatError> {
    let token = next_token(tokens, field)?;
    match token {
        b"0" => Ok(false),
        b"1" => Ok(true),
        _ => Err(GoodsDestroyFormatError::InvalidBoolean {
            token: token.to_vec(),
        }),
    }
}

fn read_u16<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<u16, GoodsDestroyFormatError> {
    let token = next_token(tokens, field)?;
    let text = std::str::from_utf8(token).map_err(|_| GoodsDestroyFormatError::InvalidGoodsType {
        token: token.to_vec(),
    })?;
    text.parse::<u16>()
        .map_err(|_| GoodsDestroyFormatError::InvalidGoodsType {
            token: token.to_vec(),
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsDestroyList {
    GoodsTypes,
    OriginalNames,
}

impl fmt::Display for GoodsDestroyList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::GoodsTypes => "типов предметов",
            Self::OriginalNames => "исходных имён предметов",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsDestroySerializeError {
    CountOutOfRange {
        list: GoodsDestroyList,
        count: usize,
    },
    NameContainsNul {
        name_index: usize,
    },
}

impl fmt::Display for GoodsDestroySerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { list, count } => write!(
                formatter,
                "GoodsDestroy содержит {count} {list} вне signed 32-битного диапазона"
            ),
            Self::NameContainsNul { name_index } => write!(
                formatter,
                "исходное имя GoodsDestroy #{name_index} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for GoodsDestroySerializeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    list: GoodsDestroyList,
) -> Result<(), GoodsDestroySerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| GoodsDestroySerializeError::CountOutOfRange { list, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

// Game decoder-а и queries, а не как Rust-реализация.
