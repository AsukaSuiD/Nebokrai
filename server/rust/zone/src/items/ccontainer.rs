//! Базовый lifecycle `CContainer` исторического GameServer: ordered vector
//! listener-ов с контрактами add/remove/duplicate и нулевым domain-смыслом
//! destructor-а.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` (идентификаторы —
//! docs/reconstruction/gameserver-npc-and-regions.md#идентификаторы-сборки).
//! Исходный owner `server/gameserver/appserver/container/ccontainer.cpp`.
//!
//! Constructor/destructor RVA `0x000DF570/0x000DF2A0` владеют только ordered
//! vector listener-ов. `AddListener` RVA `0x000DF5B0` отклоняет null и duplicate,
//! `RemoveListener` RVA `0x000DF250` удаляет первое совпадение с сохранением
//! порядка. `Vec` и RAII заменяют MSVC allocation/memmove без изменения этих
//! контрактов. Старый pointer identity выражен непрозрачным ненулевым handle:
//! container не получает владение самим listener-ом.
//! `IndexSet` заменяет ручные `contains + push` и поиск позиции при удалении,
//! сохраняя уникальность, порядок вставки и сдвиг последующих обработчиков.
//!
//! `Find/Remove` RVA `0x000DF1A0..0x000DF200` были только virtual forwarding
//! thunks: overload с type игнорировал type, overload с object извлекал его
//! `m_guExID`, null возвращал null. В Rust эти переходы принадлежат typed API
//! конкретного derived container-а; отдельное фиктивное base-хранилище не
//! вводится. `tagPreviousContainer` RVA `0x000DF1D0` сохранён буквально.

use indexmap::IndexSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContainerListenerHandle(usize);

type OrderedListenerSet = IndexSet<ContainerListenerHandle, BuildHasherDefault<DefaultHasher>>;

impl ContainerListenerHandle {
    /// Null pointer исходного API не образует listener identity.
    pub const fn from_legacy_identity(identity: usize) -> Option<Self> {
        if identity == 0 {
            None
        } else {
            Some(Self(identity))
        }
    }

    pub const fn legacy_identity(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PreviousContainer {
    pub container_type: i32,
    pub container_id: i32,
    pub container_extend_id: i32,
    pub goods_position: u32,
}

/// Общая часть всех concrete container-ов; object storage остаётся у derived
/// owner-а, как и в исходной virtual иерархии.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CContainer {
    listeners: OrderedListenerSet,
}

impl CContainer {
    pub const fn new() -> Self {
        Self {
            listeners: IndexSet::with_hasher(BuildHasherDefault::new()),
        }
    }

    pub fn listener_snapshot(&self) -> Vec<ContainerListenerHandle> {
        self.listeners.iter().copied().collect()
    }

    pub fn release(&mut self) {
        self.listeners.clear();
    }

    pub fn add_listener(&mut self, listener: Option<ContainerListenerHandle>) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        self.listeners.insert(listener)
    }

    pub fn remove_listener(&mut self, listener: Option<ContainerListenerHandle>) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        self.listeners.shift_remove(&listener)
    }
}
