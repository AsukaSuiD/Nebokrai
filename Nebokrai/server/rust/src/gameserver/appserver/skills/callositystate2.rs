//! Каноническое состояние второй закалки `CCallosityState2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `callositystate2.cpp`. Состояние `0x7d` прибавляет коэффициент `CCH` с
//! переполнением. Подтверждённая странность сохранена: `time_to_keep` не
//! обслуживается отдельным `AI`, а клиентское время и дополнительные данные
//! равны нулю. Единственным владельцем взаимно исключающей пары остаётся
//! `CanonicalStateStorage`.

use super::callosity2::CALLOSITY_2_SKILL_ID;
use crate::gameserver::appserver::player::PlayerCombatProperties;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallosityState2 {
    blast_factor: u16,
    time_to_keep: i32,
}

impl CallosityState2 {
    pub(crate) const fn new(blast_factor: u16, time_to_keep: i32) -> Self {
        Self {
            blast_factor,
            time_to_keep,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 { CALLOSITY_2_SKILL_ID }
    pub(crate) const fn blast_factor(self) -> u16 { self.blast_factor }
    pub(crate) const fn time_to_keep(self) -> i32 { self.time_to_keep }
    pub(crate) const fn client_state_time(self) -> i32 { 0 }
    pub(crate) const fn additional_data(self) -> u32 { 0 }

    pub(crate) const fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.cch = properties.cch.wrapping_add(self.blast_factor);
        properties
    }
}
