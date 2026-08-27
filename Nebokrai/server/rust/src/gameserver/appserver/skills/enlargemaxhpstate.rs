//! Каноническая достигнутая часть `CEnlargeMaxHpState`.
//!
//! Для игрока состояние `601` складывает текущий максимум HP и знаковый
//! параметр как `u32` с переполнением, после чего ограничивает результат
//! значением `i32::MAX`. Собственного визуального сообщения и таймера нет.

use super::enlargemaxhp::ENLARGE_MAX_HP_SKILL_ID;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EnlargeMaxHpState {
    gain: i32,
}

impl EnlargeMaxHpState {
    pub(crate) const fn new(gain: i32) -> Self { Self { gain } }
    pub(crate) const fn skill_id(self) -> u32 { ENLARGE_MAX_HP_SKILL_ID }
    pub(crate) const fn apply(self, value: u32) -> u32 {
        let result = value.wrapping_add(self.gain as u32);
        if result > i32::MAX as u32 { i32::MAX as u32 } else { result }
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxhpstate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxhpstate.h

// ============================================================================
// FUNCTION: CEnlargeMaxHpState::CEnlargeMaxHpState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxhpstate.cpp:24
// RVA: 0x001E2350
// ADDRESS: 005e2350
// PROTOTYPE: undefined __thiscall CEnlargeMaxHpState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
