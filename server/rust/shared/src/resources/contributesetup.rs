//! Общее country contribution `CContributeSetup` World/Game в Shared resources.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner
//! `setup/contributesetup.cpp`.
//!
//! Loader очищает только items; одиннадцать scalars при ошибке открытия
//! сохраняются и обновляются позиционно. Malformed value оставляет уже
//! прочитанный префикс. `#` record имеет `lo hi name count`.
//!
//! Wire пишет одиннадцать `i32`, signed item count и records
//! `u32 lo/hi/count + name\0`. Пустой item vector допустим.

use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::protocol::LegacyReader;
use crate::resources::read_to_marker as read_to;

const PARAMETER_COUNT: usize = 11;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributeItem {
    pub low_value: u32,
    pub high_value: u32,
    pub count: u32,
    pub name: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CContributeSetup {
    parameters: [i32; PARAMETER_COUNT],
    items: Vec<ContributeItem>,
}

impl CContributeSetup {
    pub const fn combat_levels(&self) -> (i32, i32) {
        (self.parameters[0], self.parameters[1])
    }

    pub const fn over_level_penalties(&self) -> (i32, i32) {
        (self.parameters[2], self.parameters[3])
    }

    pub const fn contribution_base_parameters(&self) -> (i32, i32, i32, i32) {
        (
            self.parameters[4],
            self.parameters[5],
            self.parameters[6],
            self.parameters[7],
        )
    }

    pub const fn country_city_modifiers(&self) -> (i32, i32, i32) {
        (self.parameters[8], self.parameters[9], self.parameters[10])
    }

    pub fn item_for_value(&self, value: u32) -> Option<&ContributeItem> {
        self.items
            .iter()
            .find(|item| item.low_value < value && value < item.high_value)
    }

    pub fn clear_items(&mut self) {
        self.items.clear();
    }

    pub fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, ContributeSetupFileLoadError> {
        self.clear_items();
        let source = std::fs::read(path).map_err(ContributeSetupFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(ContributeSetupFileLoadError::Format)
    }

    pub fn load_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<usize, ContributeSetupFormatError> {
        self.clear_items();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());

        for parameter_index in 0..PARAMETER_COUNT {
            let _ignored_name = next_token(&mut tokens, "имя параметра")?;
            self.parameters[parameter_index] = read_i32(&mut tokens, "значение параметра")?;
        }

        let mut applied = 0;
        while read_to(&mut tokens, b"#") {
            let low_value = read_u32(&mut tokens, "нижняя граница contribution")?;
            let high_value = read_u32(&mut tokens, "верхняя граница contribution")?;
            let name = next_token(&mut tokens, "имя contribution item")?.to_vec();
            let count = read_u32(&mut tokens, "число contribution item")?;
            self.items.push(ContributeItem {
                low_value,
                high_value,
                count,
                name,
            });
            applied += 1;
        }
        Ok(applied)
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), ContributeSetupSerializeError> {
        for value in self.parameters {
            destination.extend_from_slice(&value.to_le_bytes());
        }
        let item_count = i32::try_from(self.items.len()).map_err(|_| {
            ContributeSetupSerializeError::ItemCount {
                count: self.items.len(),
            }
        })?;
        destination.extend_from_slice(&item_count.to_le_bytes());
        for item in &self.items {
            if item.name.contains(&0) {
                return Err(ContributeSetupSerializeError::NameContainsNul);
            }
            destination.extend_from_slice(&item.low_value.to_le_bytes());
            destination.extend_from_slice(&item.high_value.to_le_bytes());
            destination.extend_from_slice(&item.count.to_le_bytes());
            destination.extend_from_slice(&item.name);
            destination.push(0);
        }
        Ok(())
    }

    /// Очищает items до первого scalar, но перезаписывает одиннадцать
    /// parameters позиционно: при обрыве suffix сохраняет старые values.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<usize, ContributeSetupDecodeError> {
        self.items.clear();
        for parameter in &mut self.parameters {
            *parameter = read_wire_i32(source, cursor)?;
        }
        let count = read_wire_i32(source, cursor)?;
        for _ in 0..count.max(0) {
            let low_value = read_wire_u32(source, cursor)?;
            let high_value = read_wire_u32(source, cursor)?;
            let item_count = read_wire_u32(source, cursor)?;
            let name = read_wire_c_string(source, cursor)?;
            self.items.push(ContributeItem {
                low_value,
                high_value,
                count: item_count,
                name,
            });
        }
        Ok(self.items.len())
    }

    pub fn items(&self) -> &[ContributeItem] {
        &self.items
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributeSetupFormatError {
    UnexpectedEnd { field: &'static str },
    InvalidLong { field: &'static str, token: Vec<u8> },
}

impl fmt::Display for ContributeSetupFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { field } => write!(formatter, "отсутствует поле {field}"),
            Self::InvalidLong { field, token } => write!(
                formatter,
                "поле {field} не является 32-битным long: {}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for ContributeSetupFormatError {}

#[derive(Debug)]
pub enum ContributeSetupFileLoadError {
    Io(std::io::Error),
    Format(ContributeSetupFormatError),
}

impl fmt::Display for ContributeSetupFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for ContributeSetupFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContributeSetupSerializeError {
    ItemCount { count: usize },
    NameContainsNul,
}

impl fmt::Display for ContributeSetupSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemCount { count } => write!(
                formatter,
                "contribution setup содержит {count} items вне signed 32-битного диапазона"
            ),
            Self::NameContainsNul => {
                formatter.write_str("имя contribution item содержит внутренний NUL")
            }
        }
    }
}

impl Error for ContributeSetupSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContributeSetupDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    MissingStringTerminator {
        offset: usize,
    },
}

impl fmt::Display for ContributeSetupDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "ContributeSetup snapshot обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::MissingStringTerminator { offset } => write!(
                formatter,
                "ContributeSetup item name с {offset} не завершено нулём"
            ),
        }
    }
}

impl Error for ContributeSetupDecodeError {}

fn next_token<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<&'source [u8], ContributeSetupFormatError> {
    tokens
        .next()
        .ok_or(ContributeSetupFormatError::UnexpectedEnd { field })
}

fn read_i32<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<i32, ContributeSetupFormatError> {
    let token = next_token(tokens, field)?;
    parse_text(token, field)?
        .parse::<i32>()
        .map_err(|_| invalid_long(field, token))
}

fn read_u32<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<u32, ContributeSetupFormatError> {
    let token = next_token(tokens, field)?;
    let text = parse_text(token, field)?;
    if let Some(magnitude) = text.strip_prefix('-') {
        let magnitude = magnitude
            .parse::<u32>()
            .map_err(|_| invalid_long(field, token))?;
        return Ok(0_u32.wrapping_sub(magnitude));
    }
    text.strip_prefix('+')
        .unwrap_or(text)
        .parse::<u32>()
        .map_err(|_| invalid_long(field, token))
}

fn parse_text<'source>(
    token: &'source [u8],
    field: &'static str,
) -> Result<&'source str, ContributeSetupFormatError> {
    std::str::from_utf8(token).map_err(|_| invalid_long(field, token))
}

fn invalid_long(field: &'static str, token: &[u8]) -> ContributeSetupFormatError {
    ContributeSetupFormatError::InvalidLong {
        field,
        token: token.to_vec(),
    }
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, ContributeSetupDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(map_read_block)
}

fn read_wire_u32(source: &[u8], cursor: &mut usize) -> Result<u32, ContributeSetupDecodeError> {
    LegacyReader::read_u32_from(source, cursor).map_err(map_read_block)
}

fn map_read_block(
    block: crate::protocol::LegacyReadBlock,
) -> ContributeSetupDecodeError {
    ContributeSetupDecodeError::UnexpectedEnd {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}

fn read_wire_c_string(
    source: &[u8],
    cursor: &mut usize,
) -> Result<Vec<u8>, ContributeSetupDecodeError> {
    let offset = *cursor;
    if offset > source.len() {
        return Err(ContributeSetupDecodeError::UnexpectedEnd {
            offset,
            needed: 1,
            available: 0,
        });
    }
    LegacyReader::read_c_string_from(source, cursor, source.len() - offset)
        .map(|value| value.to_vec())
        .map_err(|_| ContributeSetupDecodeError::MissingStringTerminator { offset })
}
