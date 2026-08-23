//! Четыре officer-поля country owner-а исторического `WorldServer`.
//!
//! `CKing::CKing` и `CMinister::CMinister` оба сначала
//! создают `CCountryIdentity`, затем обнуляют ровно четыре bytes по `+0x20`;
//! это `id_type`, `quest_switch`, `appointed` и `salary_received`. Источник
//! контракта — WorldServer EXE/PDB.
//!
//! `CCountry` использует эти значения через отдельный snapshot owner; данный
//! тип сохраняет только подтверждённый lifecycle identity/officer и не вводит
//! aliasing старой C++ inheritance.

use super::countryidentity::CCountryIdentity;

/// Безопасная композиция identity и четырёх byte-полей officer-а.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct COfficer {
    identity: CCountryIdentity,
    id_type: u8,
    quest_switch: bool,
    appointed: bool,
    salary_received: bool,
}

impl Default for COfficer {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl COfficer {
 /// Повторяет reached prefix `CCountryIdentity` и четыре нулевых bytes.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            identity: CCountryIdentity::with_constructor_defaults(),
            id_type: 0,
            quest_switch: false,
            appointed: false,
            salary_received: false,
        }
    }

    pub(crate) const fn identity(&self) -> &CCountryIdentity {
        &self.identity
    }

    pub(crate) fn identity_mut(&mut self) -> &mut CCountryIdentity {
        &mut self.identity
    }

    pub(crate) const fn id_type(&self) -> u8 {
        self.id_type
    }

    pub(crate) const fn set_id_type(&mut self, id_type: u8) {
        self.id_type = id_type;
    }

    pub(crate) const fn quest_switch(&self) -> bool {
        self.quest_switch
    }

    pub(crate) const fn set_quest_switch(&mut self, enabled: bool) {
        self.quest_switch = enabled;
    }

    pub(crate) const fn appointed(&self) -> bool {
        self.appointed
    }

    pub(crate) const fn set_appointed(&mut self, appointed: bool) {
        self.appointed = appointed;
    }

    pub(crate) const fn salary_received(&self) -> bool {
        self.salary_received
    }

    pub(crate) const fn set_salary_received(&mut self, salary_received: bool) {
        self.salary_received = salary_received;
    }
}
