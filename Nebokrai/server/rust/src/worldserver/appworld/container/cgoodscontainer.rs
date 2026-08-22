//! Владелец общего goods-container исторического `WorldServer`.
//!
//! Статус constructor/destructor/Clear/Release/SetOwner RVA
//! `0x000E05A0/0x000E05C0/0x000E05F0/0x000E0600/0x000E0610` и positional
//! `Add` RVA `0x000E07F0` — `IMPLEMENTED`; остальной корпус ниже остаётся
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
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
//!
//! Exact ASM base-state подтверждает owner type/ID по `+0x14/+0x18`.
//! `Clear` — folded пустой `ret 4`, а `Release` сначала обнуляет owner и затем
//! освобождает только listener-vector `CContainer`. Rust выражает inheritance
//! композицией двух safe state-owner-ов; embedded listeners конкретных goods-
//! контейнеров пока не регистрируются, поскольку оба их callback-а доказанно
//! сведены линкером к no-op `0x004DBD10`.
//! GUID-object и typed-GUID forwarder-ы ниже делегируют concrete storage через
//! общий `ContainerGuidStorage`: null object сохраняет null, а type scalar,
//! как в EXE, не читается. Это заменяет только erased `CBaseObject*` и vtable,
//! не меняя identity либо порядок lookup/remove у concrete контейнеров.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsbaseproperties::GAP_PARTICULAR_ATTRIBUTE;
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::ccontainer::{CContainerState, SharedContainerListener};

/// Достигнутое base-состояние исходного `CGoodsContainer`, не копия ABI.
pub(crate) struct CGoodsContainerState {
    container_base: CContainerState,
    owner_type: i32,
    owner_id: i32,
}

impl CGoodsContainerState {
    /// Создаёт exact base defaults constructor-а `0x004E05A0`.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            container_base: CContainerState::with_constructor_defaults(),
            owner_type: 0,
            owner_id: 0,
        }
    }

    /// Base `Clear` доказанно не меняет ни owner, ни listeners.
    pub(crate) const fn clear(&mut self) {}

    /// Сбрасывает owner до очистки non-owning listener registry.
    pub(crate) fn release(&mut self) {
        self.owner_type = 0;
        self.owner_id = 0;
        self.container_base.release();
    }

    /// Сохраняет два signed owner scalar без дополнительных эффектов.
    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.owner_type = owner_type;
        self.owner_id = owner_id;
    }

    pub(crate) const fn owner_type(&self) -> i32 {
        self.owner_type
    }

    pub(crate) const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub(crate) fn add_listener(&mut self, listener: Option<&SharedContainerListener>) -> i32 {
        self.container_base.add_listener(listener)
    }
}

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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:19
// RVA: 0x000E05A0
// ADDRESS: 004e05a0
// PROTOTYPE: undefined __thiscall CGoodsContainer(void)
//
// IMPLEMENTED выше как `CGoodsContainerState::with_constructor_defaults`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::~CGoodsContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:35
// RVA: 0x000E05C0
// ADDRESS: 004e05c0
// PROTOTYPE: void __thiscall ~CGoodsContainer(void)
//
// IMPLEMENTED обычным `Drop`; exact ASM обнуляет owner перед base destructor,
// что не имеет внешнего эффекта для уничтожаемого Rust value.
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:143
// RVA: 0x000E05F0
// ADDRESS: 004e05f0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Exact target `0x004B4CC0` состоит из `ret 4`; IMPLEMENTED как `clear`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:151
// RVA: 0x000E0600
// ADDRESS: 004e0600
// PROTOTYPE: void __thiscall Release(void)
//
// IMPLEMENTED выше; exact tail target `0x0043EAD0` очищает vector storage, а
// не weather-state, ошибочно приписанный folded телу декомпилятором.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::SetOwner
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:230
// RVA: 0x000E0610
// ADDRESS: 004e0610
// PROTOTYPE: void __thiscall SetOwner(long param_1, long param_2)
//
// IMPLEMENTED выше как `set_owner`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Remove
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:238
// RVA: 0x000E0630
// ADDRESS: 004e0630
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2)
//
// IMPLEMENTED_OWNER: `remove_by_object_guid` из `ccontainer.rs` у concrete
// `ContainerGuidStorage`; `None` сохраняет исходный null gate.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Remove
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:243
// RVA: 0x000E0640
// ADDRESS: 004e0640
// PROTOTYPE: CBaseObject * __thiscall Remove(long param_1, CGUID * param_2, void * param_3)
//
// IMPLEMENTED_OWNER: `remove_by_typed_guid` из `ccontainer.rs` делегирует
// GUID concrete storage-owner-у и, как exact virtual forwarder, не читает
// type scalar либо context.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Find
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:248
// RVA: 0x000E0650
// ADDRESS: 004e0650
// PROTOTYPE: CBaseObject * __thiscall Find(CBaseObject * param_1)
//
// IMPLEMENTED_OWNER: `find_by_object_guid` из `ccontainer.rs` сохраняет null
// gate и передаёт embedded GUID concrete storage-owner-у.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Find
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cgoodscontainer.cpp:253
// RVA: 0x000E0660
// ADDRESS: 004e0660
// PROTOTYPE: CBaseObject * __thiscall Find(long param_1, CGUID * param_2)
//
// IMPLEMENTED_OWNER: `find_by_typed_guid` из `ccontainer.rs` игнорирует type
// scalar и делает ровно concrete GUID lookup.
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
