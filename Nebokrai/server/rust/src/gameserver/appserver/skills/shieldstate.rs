//! Упорядоченная защитная ветвь `CFightDefense::PreDefense`.
//!
//! Исходный `m_vStates` применяет щиты в порядке вставки для каждой части
//! атаки. Типизированный enum сохраняет этот порядок без RTTI и не превращает
//! состояния в универсальную систему эффектов.

use super::lifeshieldstate::{finish_life_shield_state, LifeShieldState};
use super::machineshieldstate::{send_machine_shield_state_visual, MachineShieldState};
use super::manashieldstate::{send_mana_shield_state_visual, ManaShieldState};
use super::promotionstate::PromotionState;
use crate::gameserver::appserver::states::attackpower::AttackPower;
use crate::gameserver::gameserver::game::CGame;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DefenseShieldState {
    Life(LifeShieldState),
    Machine(MachineShieldState),
    Mana(ManaShieldState),
    Promotion(PromotionState),
}

impl DefenseShieldState {
    pub(crate) const fn skill_id(self) -> u32 {
        match self {
            Self::Life(state) => state.skill_id(),
            Self::Machine(state) => state.skill_id(),
            Self::Mana(state) => state.skill_id(),
            Self::Promotion(state) => state.skill_id(),
        }
    }

    pub(crate) fn apply_pre_defense(
        &mut self,
        skill_id: u32,
        damage_factor: f32,
        player_resources: Option<(u32, Option<i32>)>,
        power: &mut AttackPower,
    ) {
        if (530..=545).contains(&skill_id) && skill_id != 544 {
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
                if power.kind
                    == crate::gameserver::appserver::states::attackpower::AttackPowerType::Element
                {
                    power.hp_damage = super::fightdefense::truncate_original(
                        f64::from(state.magic_attack_factor())
                            * f64::from(power.hp_damage)
                            * 0.001,
                    );
                }
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
            Self::Promotion(state) => state.expired(now_ms),
        }
    }
}

pub(crate) fn expire_player_defense_shields(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> usize {
    let war_soul_mana = game
        .find_player(player_id)
        .and_then(|player| player.war_soul_mana(game.goods_factory()));
    let expired = game
        .find_player_mut(player_id)
        .map(|player| player.take_expired_defense_shields(now_ms, war_soul_mana))
        .unwrap_or_default();
    let ended = expired.len();
    let mut ordinary_ended = false;
    for state in expired {
        match state {
            DefenseShieldState::Life(state) => {
                finish_life_shield_state(game, player_id, state, now_ms);
            }
            DefenseShieldState::Machine(state) => {
                ordinary_ended = true;
                send_machine_shield_state_visual(game, player_id, state, false, now_ms);
            }
            DefenseShieldState::Mana(state) => {
                ordinary_ended = true;
                send_mana_shield_state_visual(game, player_id, state, false, now_ms);
            }
            DefenseShieldState::Promotion(_) => {
                ordinary_ended = true;
            }
        }
    }
    if ordinary_ended {
        let _ = game.publish_player_states(player_id);
    }
    ended
}
