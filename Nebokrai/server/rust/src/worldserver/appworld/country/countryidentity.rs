//! Базовая country-identity исторического `WorldServer`.
//!
//! Статус constructor/destructor `CCountryIdentity` RVA
//! `0x000E0EC0/0x000E0E70`: `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\\svn\\fengyun_russia_dev\\server\\worldserver\\appworld\\country\\countryidentity.cpp:7,11`.
//!
//! Constructor устанавливает только signed ID `0` и пустое имя. `Vec<u8>`
//! заменяет MSVC `std::string` и его destructor; Rust layout не объявляется
//! копией прежнего ABI. Практический C++ reference подтверждает C-string
//! границу setter-а: embedded NUL завершает значимые bytes имени.

/// Безопасный value-owner исходной identity без vtable/string ABI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CCountryIdentity {
    id: i32,
    name: Vec<u8>,
}

impl Default for CCountryIdentity {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl CCountryIdentity {
    /// Повторяет подтверждённые нулевые значения constructor-а.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            id: 0,
            name: Vec::new(),
        }
    }

    pub(crate) const fn id(&self) -> i32 {
        self.id
    }

    pub(crate) fn name(&self) -> &[u8] {
        &self.name
    }

    pub(crate) const fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    /// Присваивает C-string prefix имени без неявной перекодировки.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        let prefix = name.split(|byte| *byte == 0).next().unwrap_or_default();
        self.name.clear();
        self.name.extend_from_slice(prefix);
    }
}
