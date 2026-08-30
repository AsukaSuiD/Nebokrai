//! Каноническая достигнутая часть `CEnlargeMaxMpState`.
//!
//! Для игрока состояние `602` складывает текущий максимум MP и знаковый
//! параметр как `u32` с переполнением, после чего ограничивает результат
//! значением `i32::MAX`. Собственного визуального сообщения и таймера нет.
//! Исходный owner PDB — `skills/enlargemaxmpstate.cpp/.h`, точная пара
//! GameServer. Constructor RVA `0x001E21D0` задаёт skill ID `0x25A` и нулевой
//! gain; `Default` выражает этот контракт без временного CState/STL noise.

use super::enlargemaxmp::ENLARGE_MAX_MP_SKILL_ID;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EnlargeMaxMpState {
    gain: i32,
}

impl EnlargeMaxMpState {
    pub(crate) const fn new(gain: i32) -> Self {
        Self { gain }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        ENLARGE_MAX_MP_SKILL_ID
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
