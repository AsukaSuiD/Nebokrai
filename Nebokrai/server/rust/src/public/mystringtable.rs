//! Byte-array adapter исходного `MyStringTable`.
//!
//! WorldServer `MyStringTable::toByteArray` —
//!; GameServer decode-вариант ниже остаётся.
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//!
//! Wire: signed 32-bit count, затем каждая пара ordered map как две
//! NUL-terminated byte-строки. Исходный метод дописывает в destination и не
//! очищает его; это сохранено. `BTreeMap`-order предоставляет базовый
//! `StringTable`, а `Vec` заменяет только MSVC vector plumbing.

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
        let count = i32::try_from(self.table.entries().len())
            .map_err(|_| self.table.entries().len())?;
        destination.extend_from_slice(&count.to_le_bytes());
        for (id, value) in self.table.entries() {
            append_c_string(destination, id);
            append_c_string(destination, value);
        }
        Ok(())
    }
}

fn append_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    destination.extend_from_slice(value);
    destination.push(0);
}
