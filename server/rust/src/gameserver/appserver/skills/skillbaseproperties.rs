//! Runtime-свойства одного уровня навыка GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/skills/skillbaseproperties.cpp`. Подтверждённый
//! контракт хранит тип, target-self flag, byte-exact имя и ordered map
//! `usage -> value`; повторный usage заменяет значение, отсутствующий возвращает
//! ноль. ID и level принадлежат composite key фабрики, а не этому объекту.
//!
//! `BTreeMap` и обычное владение Rust заменяют MSVC `std::map`, allocator и
//! ручные destructor-ы. Числовые enum-значения остаются `u32`, пока общий
//! `SkillRelated.h` не будет материализован целиком.

use std::collections::BTreeMap;

pub(crate) const UNKNOWN_SKILL_TYPE: u32 = 0x7fff_ffff;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CSkillBaseProperties {
    skill_type: u32,
    is_target_self: i32,
    skill_name: Vec<u8>,
    properties: BTreeMap<u32, u32>,
}

impl CSkillBaseProperties {
    pub(crate) fn new(skill_type: u32, is_target_self: i32, skill_name: Vec<u8>) -> Self {
        Self {
            skill_type,
            is_target_self,
            skill_name,
            properties: BTreeMap::new(),
        }
    }

    pub(crate) const fn skill_type(&self) -> u32 {
        self.skill_type
    }

    pub(crate) const fn is_target_self(&self) -> i32 {
        self.is_target_self
    }

    pub(crate) fn skill_name(&self) -> &[u8] {
        &self.skill_name
    }

    pub(crate) fn query_property(&self, usage: u32) -> u32 {
        self.properties.get(&usage).copied().unwrap_or(0)
    }

    pub(crate) fn set_property(&mut self, usage: u32, value: u32) -> Option<u32> {
        self.properties.insert(usage, value)
    }

    pub(crate) const fn properties(&self) -> &BTreeMap<u32, u32> {
        &self.properties
    }
}
