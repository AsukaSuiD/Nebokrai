//! Ограничения входа в регионы исторического Miracle.
//!
//! Статус World `CRegionSetup::AddToByteArray` RVA `0x00097B90`:
//! `IMPLEMENTED`; loader и Game decoder ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp:50`.
//!
//! Exact EXE и Game decoder подтверждают wire: signed 32-битный count, затем
//! ordered records ровно по 12 little-endian bytes: `id`, минимальный уровень
//! входа и требуемый вклад. `BTreeMap<i32, _>` заменяет `std::map<long, _>` и
//! сохраняет signed key-order; отдельный map key в wire не передаётся. Typed
//! поля исключают зависимость от C++ layout/padding, а невозможный для старого
//! 32-битного контейнера count возвращается как ошибка до изменения buffer-а.
//! Точный смысл legacy return у `LoadRegionSetup` ещё не подтверждён машинно,
//! поэтому загрузка файла здесь намеренно не выдаётся за восстановленную.

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

    /// Дописывает exact `count + ordered 12-byte records` в существующий buffer.
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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\regionsetup.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp

// ============================================================================
// FUNCTION: CRegionSetup::GetProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.h:21
// RVA: 0x00040AB0
// ADDRESS: 00440ab0
// PROTOTYPE: tagRegionSetup * __cdecl GetProperty(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionSetup::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp:62
// RVA: 0x000F01E0
// ADDRESS: 004f01e0
// PROTOTYPE: bool __cdecl DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagContend::~tagContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C12B0
// ADDRESS: 005c12b0
// PROTOTYPE: void __thiscall ~tagContend(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::tagSynthesis::~tagSynthesis
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C12E0
// ADDRESS: 005c12e0
// PROTOTYPE: void __thiscall ~tagSynthesis(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c17cc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C17CC
// ADDRESS: 005c17cc
// PROTOTYPE: undefined Catch@005c17cc()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c19d1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C19D1
// ADDRESS: 005c19d1
// PROTOTYPE: undefined Catch@005c19d1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c1b56
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C1B56
// ADDRESS: 005c1b56
// PROTOTYPE: undefined Catch@005c1b56()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c1be2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C1BE2
// ADDRESS: 005c1be2
// PROTOTYPE: undefined Catch@005c1be2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c1df1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C1DF1
// ADDRESS: 005c1df1
// PROTOTYPE: undefined Catch@005c1df1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c244c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C244C
// ADDRESS: 005c244c
// PROTOTYPE: undefined Catch@005c244c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c24db
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C24DB
// ADDRESS: 005c24db
// PROTOTYPE: undefined Catch@005c24db()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c2662
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C2662
// ADDRESS: 005c2662
// PROTOTYPE: undefined Catch@005c2662()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c29d3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C29D3
// ADDRESS: 005c29d3
// PROTOTYPE: undefined Catch@005c29d3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c2f62
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C2F62
// ADDRESS: 005c2f62
// PROTOTYPE: undefined Catch@005c2f62()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c301c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x001C301C
// ADDRESS: 005c301c
// PROTOTYPE: undefined Catch@005c301c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//








// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp

// ============================================================================
// FUNCTION: Catch@004619d2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x000619D2
// ADDRESS: 004619d2
// PROTOTYPE: undefined Catch@004619d2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionSetup::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp:50
// RVA: 0x00097B90
// ADDRESS: 00497b90
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionSetup::LoadRegionSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp:20
// RVA: 0x00098440
// ADDRESS: 00498440
// PROTOTYPE: bool __cdecl LoadRegionSetup(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052f890
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionsetup.cpp
// RVA: 0x0012F890
// ADDRESS: 0052f890
// PROTOTYPE: undefined Unwind@0052f890()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
