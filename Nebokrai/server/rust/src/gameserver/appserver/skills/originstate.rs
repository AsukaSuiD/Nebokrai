//! Каноническая достигнутая player-часть `COriginState`.
//!
//! Игрок получает знаково расширенные младшие 16 бит параметра через
//! wrapping-сложение с `element_modify`. Состояние `304` не имеет собственного
//! таймера или визуального сообщения.

use super::origin::ORIGIN_SKILL_ID;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OriginState {
    element_modify_gain: i32,
}

impl OriginState {
    pub(crate) const fn new(element_modify_gain: i32) -> Self { Self { element_modify_gain } }
    pub(crate) const fn skill_id(self) -> u32 { ORIGIN_SKILL_ID }
    pub(crate) const fn apply_to_player(self, value: i32) -> i32 {
        value.wrapping_add((self.element_modify_gain as i16) as i32)
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранены конструктор по умолчанию и смешанная player/monster-функция свойств; player-ветвь подключена.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\originstate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\originstate.h

// ============================================================================
// FUNCTION: COriginState::COriginState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\originstate.cpp:24
// RVA: 0x00201210
// ADDRESS: 00601210
// PROTOTYPE: undefined __thiscall COriginState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: COriginState::OnUpdateProperties
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `OriginState::apply_to_player`; monster-ветвь остаётся RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\originstate.cpp:37
// RVA: 0x002012D0
// ADDRESS: 006012d0
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: GameServer
