//! Настройки country contribution исторического Miracle.
//!
//! Контракт World `LoadContributeSetup` и `AddToByteArray`
//!:; Game decoder ниже остаётся.
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! Исходный владелец PDB:
//!
//! Loader очищает только item-vector: одиннадцать process-global signed
//! параметров сохраняются при ошибке открытия и обновляются позиционно по мере
//! успешного formatted extraction. Имена параметров читаются, но не
//! проверяются. Rust сохраняет этот state transition; malformed значение даёт
//! typed error после уже применённого prefix-а, а не протаскивает дальше
//! failbit и не добавляет запись с неинициализированными числами.
//!
//! После параметров каждая найденная `#`-запись имеет форму
//! `lo hi name count`. Wire повторяет одиннадцать `i32`, signed item count и
//! для каждого item `u32 lo, u32 hi, u32 count, name\0`. Пустой item-vector
//! допустим — очищенный C++ reference ошибочно требовал хотя бы одну запись.
//! Legacy name хранится byte-exact; `std::fs`, `Vec` и `Drop` заменяют только
//! CRFile/STL plumbing.

use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::public::readwrite::read_to;

const PARAMETER_COUNT: usize = 11;

/// Один исходный `tagContributeItem` без STL layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ContributeItem {
    pub(crate) low_value: u32,
    pub(crate) high_value: u32,
    pub(crate) count: u32,
    pub(crate) name: Vec<u8>,
}

/// Safe owner одиннадцати static long и item-vector-а.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CContributeSetup {
    parameters: [i32; PARAMETER_COUNT],
    items: Vec<ContributeItem>,
}

impl CContributeSetup {
 /// Очищает только item-vector, сохраняя positional scalar prefix.
    pub(crate) fn clear_items(&mut self) {
        self.items.clear();
    }

 /// Очищает items до открытия, но не трогает scalar prefix.
    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, ContributeSetupFileLoadError> {
        self.clear_items();
        let source = std::fs::read(path).map_err(ContributeSetupFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(ContributeSetupFileLoadError::Format)
    }

 /// Применяет оригинал positional scalar updates и затем `#`-items.
    pub(crate) fn load_from_bytes(
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

 /// Дописывает оригинал positional wire World owner-а.
    pub(crate) fn add_to_byte_array(
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
}

/// Safe formatted-extraction boundary с сохранённым partial prefix-state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ContributeSetupFormatError {
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
pub(crate) enum ContributeSetupFileLoadError {
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
pub(crate) enum ContributeSetupSerializeError {
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

// оставшихся call-site деталей, а не как Rust-реализация.
