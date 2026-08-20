//! Владелец общего goods-container исторического `WorldServer`.
//!
//! Статус positional `CGoodsContainer::Add` RVA `0x000E07F0` —
//! `IMPLEMENTED`; остальной корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная
//! пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
//! EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:61`.
//!
//! Восстановленная ветка работает только с уже найденным товаром позиции:
//! сначала сравнивает unsigned base-properties index, затем signed particular
//! attribute `GAP_PARTICULAR_ATTRIBUTE/1`, после чего требует max-stack больше
//! `1`. Вычитание свободного места и сложение amount остаются 32-битными
//! wrapping-операциями исходного `unsigned long`. Успех уничтожает incoming
//! товар через Rust ownership, эквивалентно `CGoodsFactory::GarbageCollect`, и
//! возвращает `None`; false возвращает тот же `Box` вызывающему как `Some`.
//!
//! Exact EXE `0x004E081F..0x004E084C` исправляет две ошибки raw-декомпилята:
//! receiver второго base-index/addon вызова загружается из `pObject`, а не из
//! numeric position. Listener loop после успеха сохранён как доказанный no-op:
//! достигнутый embedded listener amount-owner-а имеет оба слота
//! `mov eax,1; ret 0xC`, а других override-ов в exact World PDB нет. После
//! ответа reverse прекращён.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsbaseproperties::GAP_PARTICULAR_ATTRIBUTE;
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;

/// Выполняет exact stacking-ветку для уже занятой позиции.
pub(crate) fn add_to_occupied_position(
    existing: &mut CGoods,
    incoming: Box<CGoods>,
    registry: &GoodsBasePropertiesRegistry,
) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
    let existing_index = existing
        .get_base_properties_index()
        .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
    let incoming_index = incoming
        .get_base_properties_index()
        .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
    if existing_index != incoming_index
        || existing.get_addon_property_value(GAP_PARTICULAR_ATTRIBUTE, 1)
            != incoming.get_addon_property_value(GAP_PARTICULAR_ATTRIBUTE, 1)
    {
        return Ok(Some(incoming));
    }

    let max_stack = existing.get_max_stack_number(registry)?;
    if max_stack <= 1 || incoming.get_amount() > max_stack.wrapping_sub(existing.get_amount()) {
        return Ok(Some(incoming));
    }

    existing.set_amount(existing.get_amount().wrapping_add(incoming.get_amount()));
    drop(incoming);
    Ok(None)
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp

// ============================================================================
// FUNCTION: CGoodsContainer::CGoodsContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:19
// RVA: 0x000E05A0
// ADDRESS: 004e05a0
// PROTOTYPE: undefined __thiscall CGoodsContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::~CGoodsContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:35
// RVA: 0x000E05C0
// ADDRESS: 004e05c0
// PROTOTYPE: void __thiscall ~CGoodsContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:53
// RVA: 0x000E05E0
// ADDRESS: 004e05e0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:143
// RVA: 0x000E05F0
// ADDRESS: 004e05f0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:151
// RVA: 0x000E0600
// ADDRESS: 004e0600
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::SetOwner
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:230
// RVA: 0x000E0610
// ADDRESS: 004e0610
// PROTOTYPE: void __thiscall SetOwner(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:238
// RVA: 0x000E0630
// ADDRESS: 004e0630
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:243
// RVA: 0x000E0640
// ADDRESS: 004e0640
// PROTOTYPE: CBaseObject * __thiscall Remove(long param_1, CGUID * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:248
// RVA: 0x000E0650
// ADDRESS: 004e0650
// PROTOTYPE: CBaseObject * __thiscall Find(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:253
// RVA: 0x000E0660
// ADDRESS: 004e0660
// PROTOTYPE: CBaseObject * __thiscall Find(long param_1, CGUID * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::AddFromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:112
// RVA: 0x000E0690
// ADDRESS: 004e0690
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Add
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:61
// RVA: 0x000E07F0
// ADDRESS: 004e07f0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, void * param_3)
//
// IMPLEMENTED выше как `add_to_occupied_position`; concrete container
// сохраняет lookup позиции, а World listener callbacks доказанно no-op.
//

// ============================================================================
// FUNCTION: CGoodsContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:191
// RVA: 0x000E0910
// ADDRESS: 004e0910
// PROTOTYPE: CBaseObject * __thiscall Remove(ulong param_1, ulong param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@005355c0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp
// RVA: 0x001355C0
// ADDRESS: 005355c0
// PROTOTYPE: undefined Unwind@005355c0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@005355cb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp
// RVA: 0x001355CB
// ADDRESS: 005355cb
// PROTOTYPE: undefined Unwind@005355cb()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
