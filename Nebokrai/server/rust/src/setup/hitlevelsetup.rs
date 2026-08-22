//! Таблица level/hit/experience исторического Miracle.
//!
//! Статус World `LoadHitLevelSetup` RVA `0x000979C0` и
//! `AddToByteArray` RVA `0x00097550`: `IMPLEMENTED`; Game decoder ниже
//! остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\hitlevelsetup.cpp:22,51`.
//!
//! Exact EXE `0x004979C0..0x00497B2A` подтвердил спорный return: отсутствие
//! файла явно ставит `AL=0`, любой открытый файл после token-scan — `AL=1`,
//! в том числе файл без единого `*`. Поэтому пустая таблица является успешным
//! состоянием и не получает ограничения из более нового C++ reference.
//! RAW также ошибочно показывал ранний return после освобождения прежнего
//! vector: инструкции `0x004979F3..0x004979FE` сходятся на общей очистке трёх
//! указателей и продолжают открытие. Value-owner сохраняет этот clear-first
//! state transition и при ошибке файла остаётся пустым.
//!
//! Каждая запись — ровно три consecutive little-endian `u32`; wire состоит
//! из signed 32-битного count и `count × 0x0C` байт. `Vec` заменяет старый
//! static vector, `std::fs` — `CRFile`, а общий доказанный `read_to` сохраняет
//! whitespace token-scan, точный `*` и терминатор `<end>`. Повреждённое число
//! исходно могло протащить неинициализированные stack-байты; Rust вместо этого
//! возвращает typed format error, оставляя только уже полностью прочитанные
//! записи. Это устраняет внутренний UB и не назначает ему wire-семантику.

use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::public::readwrite::read_to;

/// Точный 12-байтовый элемент `tagHitLevel`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct HitLevelEntry {
    pub(crate) level: u32,
    pub(crate) hit: u32,
    pub(crate) experience: u32,
}

/// Value-owner вместо двух process-global vector-ов World/Game вариантов.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CHitLevelSetup {
    entries: Vec<HitLevelEntry>,
}

impl CHitLevelSetup {
    /// Возвращает текущий ordered набор без раскрытия mutable global state.
    pub(crate) fn entries(&self) -> &[HitLevelEntry] {
        &self.entries
    }

    /// Очищает owner на той же позиции, что и exact loader перед `rfOpen`.
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    /// Очищает прежний owner, читает файл стандартной библиотекой и парсит его.
    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, HitLevelFileLoadError> {
        self.clear();
        let source = std::fs::read(path).map_err(HitLevelFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(HitLevelFileLoadError::Format)
    }

    /// Повторяет `ReadTo("*")` и три formatted unsigned-long extraction-а.
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

    /// Дописывает exact `count + raw records` в существующий buffer.
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
}

/// Ошибка безопасного parser-а вместо formatted extraction из плохого input.
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

/// Ошибка технического file-owner-а с отдельным exact format-source.
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

/// Невозможный в исходном 32-битном vector count.
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

// Остальной сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\hitlevelsetup.cpp

// ============================================================================
// FUNCTION: CHitLevelSetup::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\hitlevelsetup.cpp:62
// RVA: 0x000C5CB0
// ADDRESS: 004c5cb0
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\hitlevelsetup.cpp

// ============================================================================
// FUNCTION: CHitLevelSetup::AddToByteArray
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\hitlevelsetup.cpp:51
// RVA: 0x00097550
// ADDRESS: 00497550
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// IMPLEMENTED_OWNER: `CHitLevelSetup::add_to_byte_array` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHitLevelSetup::LoadHitLevelSetup
// STATUS: IMPLEMENTED_OWNER / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\hitlevelsetup.cpp:22
// RVA: 0x000979C0
// ADDRESS: 004979c0
// PROTOTYPE: bool __cdecl LoadHitLevelSetup(char * param_1)
//
// IMPLEMENTED_OWNER: `load_from_bytes` вместе с owned `CGame` reload path.
// Exact `0x004979E9..0x00497A25` очищает vector до `rfOpen`, а
// `0x00497B17..0x00497B2A` возвращает false только для отсутствующего файла.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
