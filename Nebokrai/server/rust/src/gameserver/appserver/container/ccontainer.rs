//! Базовый lifecycle `CContainer` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, SHA-256 EXE
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, PDB
//! `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`.
//! Исходный owner `server/gameserver/appserver/container/ccontainer.cpp`.
//!
//! Constructor/destructor RVA `0x000DF570/0x000DF2A0` владеют только ordered
//! vector listener-ов. `AddListener` RVA `0x000DF5B0` отклоняет null и duplicate,
//! `RemoveListener` RVA `0x000DF250` удаляет первое совпадение с сохранением
//! порядка. `Vec` и RAII заменяют MSVC allocation/memmove без изменения этих
//! контрактов. Старый pointer identity выражен непрозрачным ненулевым handle:
//! container не получает владение самим listener-ом.
//!
//! `Find/Remove` RVA `0x000DF1A0..0x000DF200` были только virtual forwarding
//! thunks: overload с type игнорировал type, overload с object извлекал его
//! `m_guExID`, null возвращал null. В Rust эти переходы принадлежат typed API
//! конкретного derived container-а; отдельное фиктивное base-хранилище не
//! вводится. `tagPreviousContainer` RVA `0x000DF1D0` сохранён буквально.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ContainerListenerHandle(usize);

impl ContainerListenerHandle {
    /// Null pointer исходного API не образует listener identity.
    pub(crate) const fn from_legacy_identity(identity: usize) -> Option<Self> {
        if identity == 0 {
            None
        } else {
            Some(Self(identity))
        }
    }

    pub(crate) const fn legacy_identity(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PreviousContainer {
    pub(crate) container_type: i32,
    pub(crate) container_id: i32,
    pub(crate) container_extend_id: i32,
    pub(crate) goods_position: u32,
}

/// Общая часть всех concrete container-ов; object storage остаётся у derived
/// owner-а, как и в исходной virtual иерархии.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CContainer {
    listeners: Vec<ContainerListenerHandle>,
}

impl CContainer {
    pub(crate) const fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub(crate) fn listeners(&self) -> &[ContainerListenerHandle] {
        &self.listeners
    }

    pub(crate) fn add_listener(&mut self, listener: Option<ContainerListenerHandle>) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        if self.listeners.contains(&listener) {
            return false;
        }
        self.listeners.push(listener);
        true
    }

    pub(crate) fn remove_listener(&mut self, listener: Option<ContainerListenerHandle>) -> bool {
        let Some(listener) = listener else {
            return false;
        };
        let Some(index) = self.listeners.iter().position(|entry| *entry == listener) else {
            return false;
        };
        self.listeners.remove(index);
        true
    }
}
