//! Каноническое состояние второй закалки `CCallosityState2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `callositystate2.cpp`. Состояние `0x7d` прибавляет коэффициент `CCH` с
//! переполнением. AI/End обслуживаются общим владельцем семейства в
//! `callositystate.rs`; exact client-time использует общее тело
//! `0x005D5F30`, а дополнительные данные равны нулю. Единственным владельцем взаимно исключающей пары остаётся
//! `CanonicalStateStorage`.

use super::callosity2::CALLOSITY_2_SKILL_ID;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::states::state::{default_additional_data, timed_client_state_time};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallosityState2 {
    blast_factor: u16,
    started_at_ms: u32,
    time_to_keep: i32,
}

impl CallosityState2 {
    pub(crate) const fn new(blast_factor: u16, started_at_ms: u32, time_to_keep: i32) -> Self {
        Self {
            blast_factor,
            started_at_ms,
            time_to_keep,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 { CALLOSITY_2_SKILL_ID }
    pub(crate) const fn blast_factor(self) -> u16 { self.blast_factor }
    pub(crate) const fn started_at_ms(self) -> u32 { self.started_at_ms }
    pub(crate) const fn time_to_keep(self) -> i32 { self.time_to_keep }

    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 { timed_client_state_time(self.started_at_ms, self.time_to_keep as u32, now_milliseconds) as i32 }
    pub(crate) const fn additional_data(self) -> u32 { default_additional_data() }

    pub(crate) const fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.cch = properties.cch.wrapping_add(self.blast_factor);
        properties
    }
}
