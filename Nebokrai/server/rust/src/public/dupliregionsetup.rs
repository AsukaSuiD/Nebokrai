//! Список дублирующих регионов исторического Miracle.
//!
//! Статус World `CDupliRegionSetup::AddToByteArray` RVA `0x000506B0` и
//! `GetRandomRegion` RVA `0x00050A00`: `IMPLEMENTED`; loader и Game decoder ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp:42`.
//!
//! Exact World/Game serializers подтверждают wire: signed 32-битный count и
//! insertion-order records по восемь little-endian bytes (`region_id`,
//! `duplicate_region_id`). `Vec` заменяет старый `std::list`, поскольку
//! наблюдаемый контракт требует только порядка и размера. Typed поля исключают
//! C++ layout/padding, а переполнение count возвращается до изменения buffer-а.
//! `GetRandomRegion` сначала кладёт исходный signed ID во временный vector,
//! затем дописывает все его duplicate ID в list-order и ровно один раз вызывает
//! общий `random(count)`. `Vec` и переданный caller-ом RNG adapter заменяют
//! только STL/process-global plumbing. Контракт adapter-а исходный: при
//! положительном bound он возвращает индекс `0..bound`. Поведение повреждённого
//! ini пока не доказано и не маскируется удобной новой семантикой.

use std::error::Error;
use std::fmt;

/// Точный восьмибайтовый `CDupliRegionSetup::tagDupliRegion`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DupliRegionEntry {
    pub(crate) region_id: i32,
    pub(crate) duplicate_region_id: i32,
}

/// Value-owner вместо process-local `std::list`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CDupliRegionSetup {
    entries: Vec<DupliRegionEntry>,
}

impl CDupliRegionSetup {
    pub(crate) fn push(&mut self, entry: DupliRegionEntry) {
        self.entries.push(entry);
    }

    pub(crate) fn entries(&self) -> &[DupliRegionEntry] {
        &self.entries
    }

    /// Выбирает исходный region либо один из его duplicate в точном list-order.
    pub(crate) fn get_random_region(
        &self,
        region_id: i32,
        mut random: impl FnMut(i32) -> i32,
    ) -> i32 {
        let mut candidates = Vec::with_capacity(1 + self.entries.len());
        candidates.push(region_id);
        candidates.extend(
            self.entries
                .iter()
                .filter(|entry| entry.region_id == region_id)
                .map(|entry| entry.duplicate_region_id),
        );
        let bound = i32::try_from(candidates.len())
            .expect("32-битный legacy list не превышает i32::MAX записей");
        let selected = random(bound);
        candidates[usize::try_from(selected)
            .expect("legacy random(count) возвращает неотрицательный индекс")]
    }

    /// Дописывает exact `count + insertion-order 8-byte records`.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), DupliRegionSerializeError> {
        let count = i32::try_from(self.entries.len()).map_err(|_| DupliRegionSerializeError {
            count: self.entries.len(),
        })?;
        destination.extend_from_slice(&count.to_le_bytes());
        for entry in &self.entries {
            destination.extend_from_slice(&entry.region_id.to_le_bytes());
            destination.extend_from_slice(&entry.duplicate_region_id.to_le_bytes());
        }
        Ok(())
    }
}

/// Невозможный в исходном 32-битном `std::list` размер.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DupliRegionSerializeError {
    pub(crate) count: usize,
}

impl fmt::Display for DupliRegionSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "DupliRegionSetup содержит {} записей вне signed 32-битного диапазона",
            self.count
        )
    }
}

impl Error for DupliRegionSerializeError {}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp

// ============================================================================
// FUNCTION: CDupliRegionSetup::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp:42
// RVA: 0x000238B0
// ADDRESS: 004238b0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDupliRegionSetup::CDupliRegionSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp:5
// RVA: 0x000238F0
// ADDRESS: 004238f0
// PROTOTYPE: undefined __thiscall CDupliRegionSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDupliRegionSetup::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp:54
// RVA: 0x00023C20
// ADDRESS: 00423c20
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp

// ============================================================================
// FUNCTION: CDupliRegionSetup::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp:42
// RVA: 0x000506B0
// ADDRESS: 004506b0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDupliRegionSetup::CDupliRegionSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp:5
// RVA: 0x000506F0
// ADDRESS: 004506f0
// PROTOTYPE: undefined __thiscall CDupliRegionSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDupliRegionSetup::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp:15
// RVA: 0x00050820
// ADDRESS: 00450820
// PROTOTYPE: bool __thiscall Load(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDupliRegionSetup::GetRandomRegion
// STATUS: IMPLEMENTED / VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\dupliregionsetup.cpp:79
// RVA: 0x00050A00
// ADDRESS: 00450a00
// PROTOTYPE: long __thiscall GetRandomRegion(long param_1)
//
// IMPLEMENTED_OWNER: `CDupliRegionSetup::get_random_region` выше. Exact
// `0x00450A1E..0x00450ACC` подтверждает начальный source ID, list-order append,
// один `random(vector.size())`, сохранение выбранного значения до удаления
// временного vector и signed 32-bit return.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
