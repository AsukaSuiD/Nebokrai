//! Byte-array adapter исходного `MyStringTable`.
//!
//! WorldServer `MyStringTable::toByteArray` RVA `0x00055940` —
//! `IMPLEMENTED`; GameServer decode-вариант ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! исходный owner `e:\svn\fengyun_russia_dev\public\mystringtable.cpp:14`.
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

    /// Дописывает exact World string-table wire в существующий buffer.
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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\mystringtable.cpp

// ============================================================================
// FUNCTION: MyStringTable::_GetStringFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\mystringtable.cpp:209
// RVA: 0x0002A880
// ADDRESS: 0042a880
// PROTOTYPE: char * __thiscall _GetStringFromByteArray(uchar * param_1, long * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: MyStringTable::fromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\mystringtable.cpp:27
// RVA: 0x0002A910
// ADDRESS: 0042a910
// PROTOTYPE: long __thiscall fromByteArray(uchar * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\mystringtable.cpp

// ============================================================================
// FUNCTION: MyStringTable::toByteArray
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\mystringtable.cpp:14
// RVA: 0x00055940
// ADDRESS: 00455940
// PROTOTYPE: void __thiscall toByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// `0x00455940..0x00455A2A` дописывает count и ordered C-string пары без clear.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
