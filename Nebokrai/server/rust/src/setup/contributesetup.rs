//! Настройки country contribution исторического Miracle.
//!
//! Статус World `LoadContributeSetup` RVA `0x000937D0` и `AddToByteArray` RVA
//! `0x00092EE0`: `IMPLEMENTED`; Game decoder ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\contributesetup.cpp:43,103`.
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
    /// Очищает items до открытия, но не трогает scalar prefix.
    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, ContributeSetupFileLoadError> {
        self.items.clear();
        let source = std::fs::read(path).map_err(ContributeSetupFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(ContributeSetupFileLoadError::Format)
    }

    /// Применяет exact positional scalar updates и затем `#`-items.
    pub(crate) fn load_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<usize, ContributeSetupFormatError> {
        self.items.clear();
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

    /// Дописывает exact positional wire World owner-а.
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

// Сырой C++ ниже сохранён как локальная документация Game decoder-а и
// оставшихся call-site деталей, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\contributesetup.cpp

// ============================================================================
// FUNCTION: CContributeSetup::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\contributesetup.cpp:142
// RVA: 0x000EB490
// ADDRESS: 004eb490
// PROTOTYPE: bool __cdecl DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\contributesetup.cpp

// ============================================================================
// FUNCTION: CContributeSetup::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\contributesetup.cpp:103
// RVA: 0x00092EE0
// ADDRESS: 00492ee0
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContributeSetup::LoadContributeSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\contributesetup.cpp:43
// RVA: 0x000937D0
// ADDRESS: 004937d0
// PROTOTYPE: int __cdecl LoadContributeSetup(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
