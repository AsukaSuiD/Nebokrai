//! Базовая country-identity исторического `WorldServer`.
//!
//! — часть контракта owner-а. Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.
//!
//! Constructor устанавливает только signed ID `0` и пустое имя. `Vec<u8>`
//! заменяет MSVC `std::string` и его destructor; Rust layout не объявляется
//! копией прежнего ABI. Практический C++ reference подтверждает C-string
//! границу setter-а: embedded NUL завершает значимые bytes имени.

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

    pub(crate) fn set_name(&mut self, name: &[u8]) {
        let prefix = name.split(|byte| *byte == 0).next().unwrap_or_default();
        self.name.clear();
        self.name.extend_from_slice(prefix);
    }
}
