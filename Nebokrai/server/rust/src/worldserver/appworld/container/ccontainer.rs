//! Владелец базового контейнера исторического `WorldServer`.
//!
//! Owner реализует listener registration, virtual GUID forwarders и cleanup;
//! источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.
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

pub(crate) struct CContainerState {
    listeners: Vec<Weak<Mutex<dyn CContainerListener>>>,
}

impl CContainerState {
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

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
/// null; generic result сохраняет это без оригинал pointer-а.
pub(crate) const fn remove_base_by_guid<Removed>(_ex_id: &CGuid) -> Option<Removed> {
    None
}
