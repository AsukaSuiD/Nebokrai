//! Каноническое состояние `CNaturalState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает постоянное
//! состояние `0xdc`, нулевые клиентские время и дополнительные данные,
//! сообщения начала/окончания `0xBFE03/0xBFE04` и насыщение сопротивления
//! стихиям до `i32::MAX`. Владение состоянием остаётся у
//! `CanonicalStateStorage`. Constructor RVA `0x001F3740` задаёт ID `0xDC` и
//! нулевой gain; это буквально выражено `Default`.

use super::natural::NATURAL_SKILL_ID;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
