//! Диспетчер защитной ветви `CFightDefense::PreDefense` в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/
//! {life,mana,machine}shieldstate.cpp, promotionstate.cpp и moveshape.cpp.
//! Правило пропуска сверено по машинному коду диспетчера:
//! `skill_id < 530 || > 545 || == 544` проходит
//! к обходу состояний, остальные ID выходят из ветви. Стихийная ветвь
//! Promotion проверяет ID `0x142` и `kind == 3`.
//! Выбор источника MP по варианту щита и порядок обхода состояний описаны
//! в [бое](../../../docs/gameplay/combat.md); живые Begin/restart/AI/End
//! остаются у переходного Game.

use crate::combat::{AttackPower, AttackPowerType};

use super::lifeshield::LifeShieldState;
use super::machineshield::MachineShieldState;
use super::manashield::ManaShieldState;
use super::promotion::PromotionState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefenseShieldState {
    Life(LifeShieldState),
    Machine(MachineShieldState),
    Mana(ManaShieldState),
    Promotion(PromotionState),
}

/// ID навыков боевого духа обходят щиты; единственное исключение — 544.
pub const fn is_pre_defense_skipped_skill(skill_id: u32) -> bool {
    skill_id >= 530 && skill_id <= 545 && skill_id != 544
}

impl DefenseShieldState {
    pub const fn skill_id(self) -> u32 {
        match self {
            Self::Life(state) => state.skill_id(),
            Self::Machine(state) => state.skill_id(),
            Self::Mana(state) => state.skill_id(),
            Self::Promotion(state) => state.skill_id(),
        }
    }

    pub fn apply_pre_defense(
        &mut self,
        skill_id: u32,
        damage_factor: f32,
        player_resources: Option<(u32, Option<i32>)>,
        power: &mut AttackPower,
    ) {
        if is_pre_defense_skipped_skill(skill_id) {
            return;
        }
        match self {
            Self::Life(state) => {
                if let Some((_, war_soul_mana)) = player_resources {
                    state.absorb_damage(damage_factor, war_soul_mana, power);
                }
            }
            Self::Machine(state) => {
                if let Some((player_mana, _)) = player_resources {
                    state.absorb_damage(damage_factor, player_mana, power);
                }
            }
            Self::Mana(state) => {
                if let Some((player_mana, _)) = player_resources {
                    state.absorb_damage(damage_factor, player_mana, power);
                }
            }
            Self::Promotion(state) => {
                if power.kind == AttackPowerType::Element {
                    power.hp_damage = state.apply_element_attack(power.hp_damage);
                }
            }
        }
    }

    pub const fn expired(
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
            Self::Promotion(state) => state.expired(now_ms),
        }
    }
}
