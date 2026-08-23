//! Базовый derived lifecycle `CGoodsContainer` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cgoodscontainer.cpp`. Constructor,
//! destructor и `Release` RVA `0x001DB840/0x001DB860/0x001DB8C0` сохраняют
//! owner type/id и mode в нуле, причём `Release` также освобождает listener
//! vector base `CContainer`. `Set/GetOwner` и `Set/GetContainerMode` перенесены
//! буквально. PDB подтверждает, что ошибочно названный декомпилятором адрес
//! `0x001DB8F0` — именно `CGoodsContainer::SetContainerMode`.
//!
//! Default `Add(CBaseObject*)` и `Clear` являются намеренными no-op virtual
//! slots (`xor eax,eax; ret 0xC` и `ret 4`). Rust derived owners не вызывают
//! искусственную base-заглушку: они реализуют typed storage напрямую. Четыре
//! `Find/Remove` overload-а были только переходами в `CContainer` thunks и
//! аналогично поглощены typed API concrete container-ов. Сложные stack merge/
//! split `Add(position, CGoods*)` и `Remove(position, amount)` ниже остаются
//! RAW до materialization `CGoods` и listener callbacks.

use super::ccontainer::CContainer;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum GoodsContainerMode {
    #[default]
    Normal,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGoodsContainer {
    base: CContainer,
    owner_type: i32,
    owner_id: i32,
    mode: GoodsContainerMode,
}

impl CGoodsContainer {
    pub(crate) const fn new() -> Self {
        Self {
            base: CContainer::new(),
            owner_type: 0,
            owner_id: 0,
            mode: GoodsContainerMode::Normal,
        }
    }

    pub(crate) const fn base(&self) -> &CContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CContainer {
        &mut self.base
    }

    pub(crate) const fn owner_type(&self) -> i32 {
        self.owner_type
    }

    pub(crate) const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.owner_type = owner_type;
        self.owner_id = owner_id;
    }

    pub(crate) const fn container_mode(&self) -> GoodsContainerMode {
        self.mode
    }

    pub(crate) const fn set_container_mode(&mut self, mode: GoodsContainerMode) {
        self.mode = mode;
    }

    pub(crate) fn release(&mut self) {
        self.owner_type = 0;
        self.owner_id = 0;
        self.mode = GoodsContainerMode::Normal;
        self.base.release();
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cgoodscontainer.cpp

// ============================================================================
// FUNCTION: CGoodsShadowContainer::tagGoodsShadow::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cgoodscontainer.cpp
// RVA: 0x000DF630
// ADDRESS: 004df630
// PROTOTYPE: tagGoodsShadow * __thiscall operator=(tagGoodsShadow * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cgoodscontainer.cpp:63
// RVA: 0x001DB970
// ADDRESS: 005db970
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cgoodscontainer.cpp:186
// RVA: 0x001DBB60
// ADDRESS: 005dbb60
// PROTOTYPE: CBaseObject * __thiscall Remove(ulong param_1, ulong param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
