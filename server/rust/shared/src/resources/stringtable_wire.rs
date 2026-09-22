//! Передача таблицы по public/mystringtable.cpp и public/stringtable.h.
//! World toByteArray / Game fromByteArray (RVA 0x0002A910); очистка и журнал — у роли.
//! Формат и ограничения: docs/architecture/resources-and-configuration.md.

use std::error::Error;
use std::fmt;

use crate::protocol::LegacyReader;

use super::stringtable::StringTable;

#[derive(Default)]
pub struct MyStringTable {
    table: StringTable,
}

impl MyStringTable {
    pub const fn new() -> Self {
        Self {
            table: StringTable::new(),
        }
    }

    pub const fn table(&self) -> &StringTable {
        &self.table
    }

    pub const fn table_mut(&mut self) -> &mut StringTable {
        &mut self.table
    }

    /// Дописывает представление World в существующий буфер.
    pub fn to_byte_array(&self, destination: &mut Vec<u8>) -> Result<(), usize> {
        let count =
            i32::try_from(self.table.entries().len()).map_err(|_| self.table.entries().len())?;
        destination.extend_from_slice(&count.to_le_bytes());
        for (id, value) in self.table.entries() {
            append_c_string(destination, id);
            append_c_string(destination, value);
        }
        Ok(())
    }

    /// Применяет пары последовательно, без очистки и отката при позднем отказе.
    pub fn from_byte_array(
        &mut self,
        source: &[u8],
    ) -> Result<MyStringTableDecodeOutcome, MyStringTableDecodeError> {
        let mut cursor = 0usize;
        let declared_entries =
            LegacyReader::read_i32_from(source, &mut cursor).map_err(|block| {
                MyStringTableDecodeError {
                    field: "entry count",
                    entry_index: None,
                    offset: block.offset,
                    available: block.available,
                }
            })?;

        let mut decoded_entries = 0usize;
        let mut replaced_entries = 0usize;
        for entry_index in 0..declared_entries.max(0) {
            let id = take_c_string(source, &mut cursor, "entry ID", entry_index)?;
            let value = take_c_string(source, &mut cursor, "entry value", entry_index)?;
            if self.table.insert_owned(id, value).is_some() {
                replaced_entries += 1;
            }
            decoded_entries += 1;
        }

        let unique_entries = self.table.entries().len();
        Ok(MyStringTableDecodeOutcome {
            empty: unique_entries == 0,
            consumed: cursor,
            declared_entries,
            decoded_entries,
            replaced_entries,
            unique_entries,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MyStringTableDecodeOutcome {
    pub empty: bool,
    pub consumed: usize,
    pub declared_entries: i32,
    pub decoded_entries: usize,
    pub replaced_entries: usize,
    pub unique_entries: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MyStringTableDecodeError {
    pub field: &'static str,
    pub entry_index: Option<i32>,
    pub offset: usize,
    pub available: usize,
}

impl fmt::Display for MyStringTableDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(entry_index) = self.entry_index {
            write!(
                formatter,
                "MyStringTable payload обрывается на {} записи {} в {}: доступно {}",
                self.field, entry_index, self.offset, self.available
            )
        } else {
            write!(
                formatter,
                "MyStringTable payload обрывается на {} в {}: доступно {}",
                self.field, self.offset, self.available
            )
        }
    }
}

impl Error for MyStringTableDecodeError {}

fn take_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    entry_index: i32,
) -> Result<Vec<u8>, MyStringTableDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    if offset > source.len() {
        return Err(MyStringTableDecodeError {
            field,
            entry_index: Some(entry_index),
            offset,
            available,
        });
    }
    LegacyReader::read_c_string_from(source, cursor, available)
        .map(|value| value.to_vec())
        .map_err(|_| MyStringTableDecodeError {
            field,
            entry_index: Some(entry_index),
            offset,
            available,
        })
}

fn append_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    destination.extend_from_slice(value);
    destination.push(0);
}
