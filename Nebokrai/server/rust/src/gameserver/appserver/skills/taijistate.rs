//! Каноническая достигнутая часть `CTaiJiState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает постоянное
//! состояние `0x12d` без собственного визуального эффекта. Для игрока
//! `OnUpdateProperties` использует младшие 16 бит параметра и насыщает
//! сопротивление стихиям до `i32::MAX`. Ветка монстра остаётся сохранённым
//! псевдокодом до появления настоящего исполнителя навыка монстра.

use super::taiji::TAIJI_SKILL_ID;
use crate::gameserver::appserver::player::PlayerCombatProperties;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TaiJiState {
    element_resistance_gain: i32,
}

impl TaiJiState {
    pub(crate) const fn new(element_resistance_gain: i32) -> Self {
        Self { element_resistance_gain }
    }

    pub(crate) const fn skill_id(self) -> u32 { TAIJI_SKILL_ID }
    pub(crate) const fn player_element_resistance_gain(self) -> u16 {
        self.element_resistance_gain as u16
    }

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.element_resistance = properties
            .element_resistance
            .wrapping_add(u32::from(self.player_element_resistance_gain()))
            .min(i32::MAX as u32);
        properties
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранены конструктор по умолчанию и смешанная player/monster-функция свойств; player-ветвь подключена.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taijistate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taijistate.h

// ============================================================================
// FUNCTION: CTaiJiState::CTaiJiState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taijistate.cpp:24
// RVA: 0x00200FD0
// ADDRESS: 00600fd0
// PROTOTYPE: undefined __thiscall CTaiJiState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CTaiJiState::OnUpdateProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taijistate.cpp:37
// RVA: 0x002010F0
// ADDRESS: 006010f0
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: GameServer
