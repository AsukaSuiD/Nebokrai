//! Две таблицы преобразования экипировки исторического Miracle.
//!
//! Статус World `EquipmentComposeList::AddToByteArray` RVA `0x0008A750`:
//! `IMPLEMENTED`; text loader, Game decoder и lookup queries ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp:76`.
//!
//! Wire состоит из двух последовательных ordered map: для каждой сначала
//! signed count, затем пары `u32 source + u32 target`. Loader и Game decoder
//! использовали `map::insert`, поэтому duplicate source сохраняет первое
//! значение. Исходная функция всегда возвращала `false` даже после успешной
//! записи, но reconnect caller игнорировал результат; Rust сообщает только
//! реальную ошибку представимости count и не переносит бессодержательный flag.
//! `BTreeMap` заменяет MSVC tree без изменения unsigned key-order.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Safe owner исходных static `m_mapList1` и `m_mapList2`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EquipmentComposeList {
    first: BTreeMap<u32, u32>,
    second: BTreeMap<u32, u32>,
}

impl EquipmentComposeList {
    pub(crate) fn insert_first(&mut self, source: u32, target: u32) -> bool {
        insert_first_wins(&mut self.first, source, target)
    }

    pub(crate) fn insert_second(&mut self, source: u32, target: u32) -> bool {
        insert_first_wins(&mut self.second, source, target)
    }

    pub(crate) fn clear(&mut self) {
        self.first.clear();
        self.second.clear();
    }

    /// Дописывает exact `map1 + map2` wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), EquipmentComposeSerializeError> {
        write_map(destination, &self.first, EquipmentComposeSection::First)?;
        write_map(destination, &self.second, EquipmentComposeSection::Second)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentComposeSection {
    First,
    Second,
}

impl fmt::Display for EquipmentComposeSection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::First => "первая equipment-compose таблица",
            Self::Second => "вторая equipment-compose таблица",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentComposeSerializeError {
    pub(crate) section: EquipmentComposeSection,
    pub(crate) count: usize,
}

impl fmt::Display for EquipmentComposeSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} содержит {} записей вне signed 32-битного диапазона",
            self.section, self.count
        )
    }
}

impl Error for EquipmentComposeSerializeError {}

fn insert_first_wins(values: &mut BTreeMap<u32, u32>, source: u32, target: u32) -> bool {
    if values.contains_key(&source) {
        return false;
    }
    values.insert(source, target);
    true
}

fn write_map(
    destination: &mut Vec<u8>,
    values: &BTreeMap<u32, u32>,
    section: EquipmentComposeSection,
) -> Result<(), EquipmentComposeSerializeError> {
    let count = i32::try_from(values.len()).map_err(|_| EquipmentComposeSerializeError {
        section,
        count: values.len(),
    })?;
    destination.extend_from_slice(&count.to_le_bytes());
    for (&source, &target) in values {
        destination.extend_from_slice(&source.to_le_bytes());
        destination.extend_from_slice(&target.to_le_bytes());
    }
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация loader-а, Game decoder-а
// и lookup queries, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp

// ============================================================================
// FUNCTION: EquipmentComposeList::GetFirstCompose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp:137
// RVA: 0x001CA760
// ADDRESS: 005ca760
// PROTOTYPE: ulong __cdecl GetFirstCompose(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: EquipmentComposeList::GetSecondCompose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp:150
// RVA: 0x001CA790
// ADDRESS: 005ca790
// PROTOTYPE: ulong __cdecl GetSecondCompose(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: EquipmentComposeList::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp:102
// RVA: 0x001CA7C0
// ADDRESS: 005ca7c0
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp

// ============================================================================
// FUNCTION: EquipmentComposeList::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp:76
// RVA: 0x0008A750
// ADDRESS: 0048a750
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: EquipmentComposeList::LoadList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\equipmentcomposelist.cpp:17
// RVA: 0x0008A860
// ADDRESS: 0048a860
// PROTOTYPE: int __cdecl LoadList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
