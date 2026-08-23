//! Ограничения регионов `CRegionSetup` из WorldServer, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Wire — signed count и ordered 12-байтные records: region ID, minimum level
//! и required contribution. Map key задаёт signed order, но отдельно не идёт.
//! `BTreeMap<i32, _>` заменяет MSVC tree. Семантика return у file loader не
//! подтверждена, поэтому этот owner предоставляет только wire.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionSetupEntry {
    pub(crate) id: i32,
    pub(crate) can_enter_level: i32,
    pub(crate) required_contribute: i32,
}

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
 /// Owner очищается до чтения; при некорректной записи сохраняется уже
 /// прочитанный префикс, а неизвестные значения остаются пустыми.
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
