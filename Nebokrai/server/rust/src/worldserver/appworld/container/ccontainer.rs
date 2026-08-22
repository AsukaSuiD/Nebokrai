//! Владелец базового контейнера исторического `WorldServer`.
//!
//! Статус constructor/destructor RVA `0x000E0DB0/0x000E0AE0` и
//! `AddListener` RVA `0x000E0DF0` — `IMPLEMENTED`; virtual find/remove ниже
//! остаются `UNKNOWN` (исследовательский декомпилят хранится локально) до конкретного storage-owner-а. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:22,34,237`.
//!
//! Original vector хранит не владеющие `CContainerListener*`, отвергает null
//! и повторный pointer, а destructor/`Release` освобождает только сам vector.
//! Rust сохраняет эту семантику через стандартные `Arc/Weak`: контейнер не
//! продлевает жизнь listener-а, duplicate определяется по identity control
//! block-а, а dangling pointer не возникает. `Mutex` делает listener-owner
//! законно переносимым вместе с загруженным `CPlayer` между DB и game
//! потоками; порядок callbacks от этого не меняется. Compiler/STL capacity и
//! ручной `delete` не переносятся.

use std::sync::{Arc, Mutex, Weak};

use crate::worldserver::appworld::listener::ccontainerlistener::CContainerListener;

pub(crate) type SharedContainerListener = Arc<Mutex<dyn CContainerListener>>;

/// Безопасное owning-состояние исходного `CContainer`, не копия его ABI.
pub(crate) struct CContainerState {
    listeners: Vec<Weak<Mutex<dyn CContainerListener>>>,
}

impl CContainerState {
    /// Создаёт exact пустой listener-vector constructor-а `0x004E0DB0`.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    /// Регистрирует non-null listener один раз по identity исходного pointer-а.
    pub(crate) fn add_listener(&mut self, listener: Option<&SharedContainerListener>) -> i32 {
        let Some(listener) = listener else {
            return 0;
        };
        let weak = Arc::downgrade(listener);
        if self.listeners.iter().any(|current| current.ptr_eq(&weak)) {
            return 0;
        }
        self.listeners.push(weak);
        1
    }

    /// Освобождает только non-owning registry, как folded base `Release`.
    pub(crate) fn release(&mut self) {
        self.listeners.clear();
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp

// ============================================================================
// FUNCTION: CWorldRegion::tagWeatherTime::tagOption::~tagOption
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:197
// RVA: 0x0003EAD0
// ADDRESS: 0043ead0
// PROTOTYPE: void __thiscall ~tagOption(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainer::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:162
// RVA: 0x000E0A00
// ADDRESS: 004e0a00
// PROTOTYPE: CBaseObject * __thiscall Find(long param_1, CGUID * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:119
// RVA: 0x000E0A10
// ADDRESS: 004e0a10
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:225
// RVA: 0x000E0A20
// ADDRESS: 004e0a20
// PROTOTYPE: CBaseObject * __thiscall Remove(long param_1, CGUID * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainer::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:145
// RVA: 0x000E0A40
// ADDRESS: 004e0a40
// PROTOTYPE: CBaseObject * __thiscall Find(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:209
// RVA: 0x000E0A60
// ADDRESS: 004e0a60
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainer::~CContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:34
// RVA: 0x000E0AE0
// ADDRESS: 004e0ae0
// PROTOTYPE: void __thiscall ~CContainer(void)
//
// IMPLEMENTED обычным `Drop` для vector `Weak`; listener-ы не принадлежат
// контейнеру и не уничтожаются вместе с ним.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainer::CContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:22
// RVA: 0x000E0DB0
// ADDRESS: 004e0db0
// PROTOTYPE: undefined __thiscall CContainer(void)
//
// IMPLEMENTED выше как `CContainerState::with_constructor_defaults`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainer::AddListener
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp:237
// RVA: 0x000E0DF0
// ADDRESS: 004e0df0
// PROTOTYPE: int __thiscall AddListener(CContainerListener * param_1)
//
// IMPLEMENTED выше как `add_listener`; `Weak::ptr_eq` заменяет raw pointer
// identity, сохраняя ответы `0/1` и порядок vector-а.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00535540
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp
// RVA: 0x00135540
// ADDRESS: 00535540
// PROTOTYPE: undefined Unwind@00535540()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0053554b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\ccontainer.cpp
// RVA: 0x0013554B
// ADDRESS: 0053554b
// PROTOTYPE: undefined Unwind@0053554b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
