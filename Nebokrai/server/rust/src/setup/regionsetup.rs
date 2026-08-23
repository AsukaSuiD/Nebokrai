//! Ограничения входа в регионы исторического Miracle.
//!
//! Контракт World `CRegionSetup::AddToByteArray`:
//!; loader и Game decoder не входят в этот owner и остаются.
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! Исходный owner PDB:
//!
//! Оригинал и Game decoder подтверждают wire: signed 32-битный count, затем
//! ordered records ровно по 12 little-endian bytes: `id`, минимальный уровень
//! входа и требуемый вклад. `BTreeMap<i32, _>` заменяет `std::map<long, _>` и
//! сохраняет signed key-order; отдельный map key в wire не передаётся. Typed
//! поля исключают зависимость от C++ layout/padding, а невозможный для старого
//! 32-битного контейнера count возвращается как ошибка до изменения buffer-а.
//! Точный смысл legacy return у `LoadRegionSetup` ещё не подтверждён машинно,
//! поэтому загрузка файла здесь намеренно не выдаётся за действуетную.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Точный 12-байтовый `CRegionSetup::tagRegionSetup`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionSetupEntry {
    pub(crate) id: i32,
    pub(crate) can_enter_level: i32,
    pub(crate) required_contribute: i32,
}

/// Value-owner вместо process-global `s_mapRegionSetup`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CRegionSetup {
    entries: BTreeMap<i32, RegionSetupEntry>,
}

impl CRegionSetup {
    pub(crate) fn insert(&mut self, entry: RegionSetupEntry) -> Option<RegionSetupEntry> {
        self.entries.insert(entry.id, entry)
    }

    pub(crate) fn entries(&self) -> &BTreeMap<i32, RegionSetupEntry> {
        &self.entries
    }

 /// Читает оригинал World grammar: поиск маркера `#`, затем три signed long.
 /// Owner очищается до чтения; при malformed записи сохраняется уже
 /// подтверждённый prefix, но неизвестные значения не материализуются.
    pub(crate) fn load_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<usize, RegionSetupLoadError> {
        self.entries.clear();
        let tokens: Vec<&[u8]> = source
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty())
            .collect();
        let mut next = 0;
        let mut loaded = 0;
        while let Some(relative) = tokens[next..].iter().position(|token| *token == b"#") {
            next += relative + 1;
            let read = |offset: usize, field| {
                let token = tokens.get(next + offset).ok_or(RegionSetupLoadError::Missing {
                    record: loaded,
                    field,
                })?;
                std::str::from_utf8(token)
                    .ok()
                    .and_then(|text| text.parse::<i32>().ok())
                    .ok_or_else(|| RegionSetupLoadError::Invalid {
                        record: loaded,
                        field,
                        token: token.to_vec(),
                    })
            };
            let entry = RegionSetupEntry {
                id: read(0, "ID")?,
                can_enter_level: read(1, "уровень")?,
                required_contribute: read(2, "вклад")?,
            };
            self.entries.insert(entry.id, entry);
            loaded += 1;
            next += 3;
        }
        Ok(loaded)
    }

 /// Дописывает оригинал `count + ordered 12-byte records` в существующий buffer.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), RegionSetupSerializeError> {
        let count = i32::try_from(self.entries.len()).map_err(|_| RegionSetupSerializeError {
            count: self.entries.len(),
        })?;
        destination.extend_from_slice(&count.to_le_bytes());
        for entry in self.entries.values() {
            destination.extend_from_slice(&entry.id.to_le_bytes());
            destination.extend_from_slice(&entry.can_enter_level.to_le_bytes());
            destination.extend_from_slice(&entry.required_contribute.to_le_bytes());
        }
        Ok(())
    }
}

/// Невозможный в исходном 32-битном `std::map` размер.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionSetupSerializeError {
    pub(crate) count: usize,
}

impl fmt::Display for RegionSetupSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "RegionSetup содержит {} записей вне signed 32-битного диапазона",
            self.count
        )
    }
}

impl Error for RegionSetupSerializeError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionSetupLoadError {
    Missing {
        record: usize,
        field: &'static str,
    },
    Invalid {
        record: usize,
        field: &'static str,
        token: Vec<u8>,
    },
}

impl fmt::Display for RegionSetupLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { record, field } => {
                write!(formatter, "RegionSetup: запись {record}, отсутствует поле {field}")
            }
            Self::Invalid {
                record,
                field,
                token,
            } => write!(
                formatter,
                "RegionSetup: запись {record}, поле {field} содержит нецелое значение {:?}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for RegionSetupLoadError {}
