//! Владелец базового контейнера исторического `WorldServer`.
//!
//! Статус constructor/destructor RVA `0x000E0DB0/0x000E0AE0`, folded
//! `CWorldRegion::tagWeatherTime::tagOption` destructor RVA `0x0003EAD0` и
//! `AddListener` RVA `0x000E0DF0` и virtual GUID forwarder-ы
//! `0x000E0A00..0x000E0A60` — `IMPLEMENTED`. Точная пара:
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

use crate::public::guid::CGuid;
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

/// Concrete storage, к которому старый `CContainer` обращался virtual slot-ом
/// GUID find/remove. Rust не хранит erased `CBaseObject*` и не вводит vtable.
pub(crate) trait ContainerGuidStorage {
    type Object;
    type Removed;

    fn find_by_guid(&self, ex_id: &CGuid) -> Option<&Self::Object>;
    fn remove_by_guid(&mut self, ex_id: &CGuid) -> Option<Self::Removed>;
}

/// Повторяет `Find(long, GUID)`: scalar type не читается и GUID передаётся
/// concrete virtual owner-у без дополнительных эффектов.
pub(crate) fn find_by_typed_guid<'storage, Storage: ContainerGuidStorage>(
    storage: &'storage Storage,
    _object_type: i32,
    ex_id: &CGuid,
) -> Option<&'storage Storage::Object> {
    storage.find_by_guid(ex_id)
}

/// Повторяет `Remove(long, GUID, void*)`: scalar type не читается.
pub(crate) fn remove_by_typed_guid<Storage: ContainerGuidStorage>(
    storage: &mut Storage,
    _object_type: i32,
    ex_id: &CGuid,
) -> Option<Storage::Removed> {
    storage.remove_by_guid(ex_id)
}

/// Повторяет object-overload `Find`: null даёт null, иначе используется
/// embedded GUID объекта.
pub(crate) fn find_by_object_guid<'storage, Storage: ContainerGuidStorage>(
    storage: &'storage Storage,
    ex_id: Option<&CGuid>,
) -> Option<&'storage Storage::Object> {
    storage.find_by_guid(ex_id?)
}

/// Повторяет object-overload `Remove`: null даёт null, иначе GUID делегируется
/// concrete storage-owner-у.
pub(crate) fn remove_by_object_guid<Storage: ContainerGuidStorage>(
    storage: &mut Storage,
    ex_id: Option<&CGuid>,
) -> Option<Storage::Removed> {
    storage.remove_by_guid(ex_id?)
}

/// Базовый virtual `Remove(GUID, void*)` не имел storage и всегда возвращал
/// null; generic result сохраняет это без raw pointer-а.
pub(crate) const fn remove_base_by_guid<Removed>(_ex_id: &CGuid) -> Option<Removed> {
    None
}
