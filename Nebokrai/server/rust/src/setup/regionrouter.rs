//! Межрегиональная таблица маршрутизации исторического Miracle.
//!
//! Статус World `CRegionRouter::AddToByteArray` RVA `0x000B24D0`:
//! `IMPLEMENTED`; loader, route search, singleton и Game decoder ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp:81`.
//!
//! Wire: signed region count, затем для каждого ordered map node шесть `i32`
//! (`region_id`, вход X/Y, выход X/Y, out-range), signed next count и ordered
//! 12-байтные `next_region_id + X + Y`. Map key отдельно не передаётся: EXE
//! пишет ID из value; это различие сохранено. Параметр `sendSelf` точным
//! serializer-ом не читался. `BTreeMap` и owned values заменяют MSVC tree и
//! raw struct-copy, не меняя signed ordering или compact layout.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRoutePoint {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionNextNode {
    pub(crate) next_region_id: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRouterNode {
    pub(crate) region_id: i32,
    pub(crate) entry: RegionRoutePoint,
    pub(crate) exit: RegionRoutePoint,
    pub(crate) exit_range: i32,
    pub(crate) next: BTreeMap<i32, RegionNextNode>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRouter {
    nodes: BTreeMap<i32, RegionRouterNode>,
}

impl RegionRouter {
    pub(crate) fn insert_node(
        &mut self,
        map_region_id: i32,
        node: RegionRouterNode,
    ) -> Option<RegionRouterNode> {
        self.nodes.insert(map_region_id, node)
    }

    pub(crate) fn clear(&mut self) {
        self.nodes.clear();
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), RegionRouterSerializeError> {
        write_count(destination, self.nodes.len(), None)?;
        for node in self.nodes.values() {
            destination.extend_from_slice(&node.region_id.to_le_bytes());
            destination.extend_from_slice(&node.entry.x.to_le_bytes());
            destination.extend_from_slice(&node.entry.y.to_le_bytes());
            destination.extend_from_slice(&node.exit.x.to_le_bytes());
            destination.extend_from_slice(&node.exit.y.to_le_bytes());
            destination.extend_from_slice(&node.exit_range.to_le_bytes());
            write_count(destination, node.next.len(), Some(node.region_id))?;
            for next in node.next.values() {
                destination.extend_from_slice(&next.next_region_id.to_le_bytes());
                destination.extend_from_slice(&next.x.to_le_bytes());
                destination.extend_from_slice(&next.y.to_le_bytes());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionRouterSerializeError {
    pub(crate) region_id: Option<i32>,
    pub(crate) count: usize,
}

impl fmt::Display for RegionRouterSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.region_id {
            Some(region_id) => write!(
                formatter,
                "region {region_id} содержит {} переходов вне signed 32-битного диапазона",
                self.count
            ),
            None => write!(
                formatter,
                "RegionRouter содержит {} регионов вне signed 32-битного диапазона",
                self.count
            ),
        }
    }
}

impl Error for RegionRouterSerializeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    region_id: Option<i32>,
) -> Result<(), RegionRouterSerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| RegionRouterSerializeError { region_id, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация loader-а, route search,
// singleton и Game decoder-а, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\regionrouter.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp

// ============================================================================
// FUNCTION: CRegionRouter::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionrouter.h:70
// RVA: 0x0001E420
// ADDRESS: 0041e420
// PROTOTYPE: CRegionRouter * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionRouter::stRouterNode::GetNextPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp:10
// RVA: 0x000A4C30
// ADDRESS: 004a4c30
// PROTOTYPE: void __thiscall GetNextPoint(long param_1, long * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionRouter::GetRegionInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp:320
// RVA: 0x000A4C70
// ADDRESS: 004a4c70
// PROTOTYPE: bool __thiscall GetRegionInfo(long param_1, long * param_2, long * param_3, long * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionRouter::IsConectRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp:279
// RVA: 0x000A4D80
// ADDRESS: 004a4d80
// PROTOTYPE: bool __thiscall IsConectRegion(long param_1, long param_2, long * param_3, long * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionRouter::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp:105
// RVA: 0x000A5750
// ADDRESS: 004a5750
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\regionrouter.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp

// ============================================================================
// FUNCTION: CRegionRouter::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionrouter.h:70
// RVA: 0x000300D0
// ADDRESS: 004300d0
// PROTOTYPE: CRegionRouter * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionRouter::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp:81
// RVA: 0x000B24D0
// ADDRESS: 004b24d0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionRouter::LoadRouterSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp:30
// RVA: 0x000B3E10
// ADDRESS: 004b3e10
// PROTOTYPE: void __thiscall LoadRouterSetup(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegionRouter::ChageRegionRouter
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\regionrouter.cpp:141
// RVA: 0x000B4350
// ADDRESS: 004b4350
// PROTOTYPE: void __thiscall ChageRegionRouter(long param_1, long param_2, vector<std::pair<long,tagPOINT>,std::allocator<std::pair<long,tagPOINT>_>_> * param_3, long param_4, long param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
