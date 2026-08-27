//! Каноническое состояние `CNaturalState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает постоянное
//! состояние `0xdc`, нулевые клиентские время и дополнительные данные,
//! сообщения начала/окончания `0xBFE03/0xBFE04` и насыщение сопротивления
//! стихиям до `i32::MAX`. Владение состоянием остаётся у
//! `CanonicalStateStorage`; сохранённый псевдокод ниже описывает остальной
//! непереведённый корпус.

use super::natural::NATURAL_SKILL_ID;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NaturalState {
    element_resistance_gain: u16,
}

impl NaturalState {
    pub(crate) const fn new(element_resistance_gain: u16) -> Self {
        Self {
            element_resistance_gain,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        NATURAL_SKILL_ID
    }
    pub(crate) const fn element_resistance_gain(self) -> u16 {
        self.element_resistance_gain
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\naturalstate.cpp

// ============================================================================
// FUNCTION: CNaturalState::CNaturalState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\naturalstate.cpp:24
// RVA: 0x001F3740
// ADDRESS: 005f3740
// PROTOTYPE: undefined __thiscall CNaturalState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
