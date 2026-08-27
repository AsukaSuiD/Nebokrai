//! Упорядоченная защитная ветвь `CFightDefense::PreDefense`.
//!
//! Исходный `m_vStates` применяет щиты в порядке вставки для каждой части
//! атаки. Типизированный enum сохраняет этот порядок без RTTI и не превращает
//! состояния в универсальную систему эффектов.

use super::lifeshieldstate::LifeShieldState;
use super::machineshieldstate::MachineShieldState;
use super::manashieldstate::ManaShieldState;
use crate::gameserver::appserver::states::attackpower::AttackPower;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DefenseShieldState {
    Life(LifeShieldState),
    Machine(MachineShieldState),
    Mana(ManaShieldState),
}

impl DefenseShieldState {
    pub(crate) const fn skill_id(self) -> u32 {
        match self {
            Self::Life(state) => state.skill_id(),
            Self::Machine(state) => state.skill_id(),
            Self::Mana(state) => state.skill_id(),
        }
    }

    pub(crate) fn absorb_damage(
        &mut self,
        skill_id: u32,
        damage_factor: f32,
        player_mana: u32,
        war_soul_mana: Option<i32>,
        power: &mut AttackPower,
    ) {
        if (530..=545).contains(&skill_id) && skill_id != 544 {
            return;
        }
        match self {
            Self::Life(state) => state.absorb_damage(damage_factor, war_soul_mana, power),
            Self::Machine(state) => state.absorb_damage(damage_factor, player_mana, power),
            Self::Mana(state) => {
                state.absorb_damage(damage_factor, player_mana, power);
            }
        }
    }

    pub(crate) const fn expired(
        self,
        now_ms: u32,
        player_mana: u32,
        dead: bool,
        war_soul_mana: Option<i32>,
    ) -> bool {
        match self {
            Self::Life(state) => state.expired(now_ms, player_mana, dead, war_soul_mana),
            Self::Machine(state) => state.expired(now_ms, player_mana, dead),
            Self::Mana(state) => state.expired(now_ms, player_mana, dead),
        }
    }
}
