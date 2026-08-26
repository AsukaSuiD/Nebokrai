//! Byte-array adapter исходного `MyStringTable`.
//!
//! WorldServer `toByteArray` и GameServer `fromByteArray` RVA `0x0002A910`
//! подтверждают общий wire: signed 32-bit count, затем каждая ordered-map пара
//! как две NUL-terminated byte-строки. Точные пары:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb` и
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`; исходный owner
//! `public/mystringtable.cpp` с объявлением в `public/stringtable.h`.
//!
//! Encoder дописывает в destination и не очищает его. Decoder не очищает
//! таблицу, принимает non-positive count как пустой prefix и публикует пары по
//! мере чтения; duplicate ID заменяет прежнее значение. Безграничное чтение
//! C-строк из EXE заменено проверкой конца slice с сохранением уже применённого
//! prefix. `BTreeMap`-order предоставляет базовый `StringTable`, а `Vec`
//! заменяет только MSVC vector/string plumbing.

use std::error::Error;
use std::fmt;

use crate::gameserver::appserver::legacycodec::LegacyReader;

use super::stringtable::StringTable;

#[derive(Default)]
pub(crate) struct MyStringTable {
    table: StringTable,
}

impl MyStringTable {
    pub(crate) const fn new() -> Self {
        Self {
            table: StringTable::new(),
        }
    }

    pub(crate) const fn table(&self) -> &StringTable {
        &self.table
    }

    pub(crate) const fn table_mut(&mut self) -> &mut StringTable {
        &mut self.table
    }

    /// Дописывает оригинал World string-table wire в существующий buffer.
    pub(crate) fn to_byte_array(&self, destination: &mut Vec<u8>) -> Result<(), usize> {
        let count =
            i32::try_from(self.table.entries().len()).map_err(|_| self.table.entries().len())?;
        destination.extend_from_slice(&count.to_le_bytes());
        for (id, value) in self.table.entries() {
            append_c_string(destination, id);
            append_c_string(destination, value);
        }
        Ok(())
    }

    /// Декодирует Game string-table payload и возвращает exact consumed length.
    pub(crate) fn from_byte_array(
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
        tracing::trace!(
            declared_entries,
            decoded_entries,
            replaced_entries,
            unique_entries,
            consumed = cursor,
            "таблица строк декодирована"
        );
        Ok(MyStringTableDecodeOutcome {
            empty: unique_entries == 0,
            consumed: cursor,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MyStringTableDecodeOutcome {
    pub(crate) empty: bool,
    pub(crate) consumed: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MyStringTableDecodeError {
    pub(crate) field: &'static str,
    pub(crate) entry_index: Option<i32>,
    pub(crate) offset: usize,
    pub(crate) available: usize,
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
