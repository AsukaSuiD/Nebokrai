//! Каноническая достигнутая часть `CEnlargeMaxHpState`.
//!
//! Для игрока состояние `601` складывает текущий максимум HP и знаковый
//! параметр как `u32` с переполнением, после чего ограничивает результат
//! значением `i32::MAX`. Собственного визуального сообщения и таймера нет.
//! Исходный owner PDB — `skills/enlargemaxhpstate.cpp/.h`, точная пара
//! GameServer. Constructor RVA `0x001E2350` задаёт skill ID `0x259` и нулевой
//! gain; `Default` выражает этот контракт без временного CState/STL noise.

use super::enlargemaxhp::ENLARGE_MAX_HP_SKILL_ID;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EnlargeMaxHpState {
    gain: i32,
}

impl EnlargeMaxHpState {
    pub(crate) const fn new(gain: i32) -> Self {
        Self { gain }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        ENLARGE_MAX_HP_SKILL_ID
    }

    pub(crate) const fn apply(self, value: u32) -> u32 {
        let result = value.wrapping_add(self.gain as u32);
        if result > i32::MAX as u32 {
            i32::MAX as u32
        } else {
            result
        }
    }
}
