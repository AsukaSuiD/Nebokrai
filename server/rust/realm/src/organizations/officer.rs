//! Четыре officer-поля country owner-а исторического `WorldServer`,
//! перенесённые в Realm `organizations/`.
//!
//! `CKing::CKing` и `CMinister::CMinister` оба сначала
//! создают `CCountryIdentity`, затем обнуляют ровно четыре bytes по `+0x20`;
//! это `id_type`, `quest_switch`, `appointed` и `salary_received`. Источник
//! контракта — `worldserver.exe` и `worldserver.pdb`.
//!
//! `CCountry` использует эти значения через отдельный snapshot owner; данный
//! тип сохраняет только подтверждённый lifecycle identity/officer и не вводит
//! aliasing старой C++ inheritance.

use super::countryidentity::CCountryIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct COfficer {
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
    pub const fn with_constructor_defaults() -> Self {
        Self {
            identity: CCountryIdentity::with_constructor_defaults(),
            id_type: 0,
            quest_switch: false,
            appointed: false,
            salary_received: false,
        }
    }

    pub const fn identity(&self) -> &CCountryIdentity {
        &self.identity
    }

    pub fn identity_mut(&mut self) -> &mut CCountryIdentity {
        &mut self.identity
    }

    pub const fn id_type(&self) -> u8 {
        self.id_type
    }

    pub const fn set_id_type(&mut self, id_type: u8) {
        self.id_type = id_type;
    }

    pub const fn quest_switch(&self) -> bool {
        self.quest_switch
    }

    pub const fn set_quest_switch(&mut self, enabled: bool) {
        self.quest_switch = enabled;
    }

    pub const fn appointed(&self) -> bool {
        self.appointed
    }

    pub const fn set_appointed(&mut self, appointed: bool) {
        self.appointed = appointed;
    }

    pub const fn salary_received(&self) -> bool {
        self.salary_received
    }

    pub const fn set_salary_received(&mut self, salary_received: bool) {
        self.salary_received = salary_received;
    }
}
