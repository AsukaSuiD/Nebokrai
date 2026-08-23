//! Номинальный minister owner исторического `WorldServer`.
//!
//! действует. Constructor создаёт country identity и обнуляет те же четыре
//! officer bytes; destructor не добавляет наблюдаемого эффекта поверх
//! `COfficer`. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Rust сохраняет отдельный nominal type и composition вместо C++ vtable.

use super::officer::COfficer;

/// Номинальный minister с полным действующим officer-prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CMinister {
    officer: COfficer,
}

impl Default for CMinister {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl CMinister {
 /// Повторяет нулевой constructor-state minister-а.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            officer: COfficer::with_constructor_defaults(),
        }
    }

    pub(crate) const fn officer(&self) -> &COfficer {
        &self.officer
    }

    pub(crate) fn officer_mut(&mut self) -> &mut COfficer {
        &mut self.officer
    }
}
