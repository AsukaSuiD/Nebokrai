//! Номинальный minister owner исторического `WorldServer`.
//!
//! Constructor создаёт country identity и обнуляет четыре
//! officer bytes; destructor не добавляет наблюдаемого эффекта поверх
//! `COfficer`. Источник контракта — `worldserver.exe` и `worldserver.pdb`.
//!
//! Rust сохраняет отдельный nominal type и composition вместо C++ vtable.

use super::officer::COfficer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CMinister {
    officer: COfficer,
}

impl Default for CMinister {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl CMinister {
    pub const fn with_constructor_defaults() -> Self {
        Self {
            officer: COfficer::with_constructor_defaults(),
        }
    }

    pub const fn officer(&self) -> &COfficer {
        &self.officer
    }

    pub fn officer_mut(&mut self) -> &mut COfficer {
        &mut self.officer
    }
}
