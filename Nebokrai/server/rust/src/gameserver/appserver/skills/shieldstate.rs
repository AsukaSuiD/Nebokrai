//! Упорядоченная защитная ветвь `CFightDefense::PreDefense`.
//!
//! Исходный `m_vStates` применяет щиты в порядке вставки для каждой части
//! атаки. Типизированный enum сохраняет этот порядок без RTTI и не превращает
//! состояния в универсальную систему эффектов.
//! Источник: `gameserver.exe` + `GameServer.pdb`, владельцы
//! `appserver/skills/{life,mana,machine}shieldstate.cpp` и `moveshape.cpp`.
//! AI (`0x5E2D90`, `0x5F34B0`) проверяет срок, прочность, смерть и MP.
//! End (`0x5E3110`, `0x5FD420`) выполняет эффект до RemoveState
//! (`0x4CDAB0`), который вызывает UpdateProperty игрока. Поэтому следующий
//! щит проверяется после полного завершения предыдущего, а LifeShield
//! создаёт Cure до удаления самого щита. Экземплярами владеет общий
//! `CMoveShape::state_entries`: SlotMap сохраняет identity, а список адресов —
//! исходный порядок и пропуски. End удаляет тот же ключ после своего эффекта.
//! Для чистого PreDefense payload временно выделяется в keyed batch и
//! возвращается в прежние ключи до любых callbacks или применения урона.
//! Vtable Life 0x0065F1EC направляет AI на 0x005E2D90; Mana 0x00660654 и
//! Machine 0x006604DC — на одно тело 0x005F34B0. До чтения ресурсов общими
//! являются deadline, signed life, GetSufferer и IsDied. Затем EXE читает
//! unchecked CPlayer layout [+0x284], а Life — ещё GetWarSoulGoods0x0042DF10.
//! Достигнутые creators и DB load этих трёх щитов принадлежат CPlayer.
//! Для прочих materialized holders выполняется доказанный общий префикс AI
//! и настоящий End, но MP/war soul не выдумываются: семантика последующего
//! unchecked player-layout вне достигнутых creators остаётся неизвестной.
//! Promotion имеет только собственный срок, без life/dead/resource gates.
//! Direct End получает тот же живой ключ без проверок AI. Life выполняет
//! AddCure с вложенным End прежнего Cure до своего visual и удаления;
//! Machine/Mana выполняют visual→Remove, Promotion не создаёт End-пакет.
//! Player-wrapper и generic AI входят в один End с опубликованным holder.

use super::lifeshieldstate::{finish_life_shield_state_for_holder, LifeShieldState};
use super::machineshieldstate::MachineShieldState;
use super::manashieldstate::ManaShieldState;
use super::promotionstate::PromotionState;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackPower;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

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

pub(crate) fn update_defense_shield(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if holder.object_type == 400 {
        return expire_player_defense_shield(game, holder.id, key, now_ms);
    }
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.defense_shield(key)).copied() else { return false };
    let expired = match state {
        DefenseShieldState::Life(state) => state.lifetime_expired(now_ms),
        DefenseShieldState::Machine(state) => state.lifetime_expired(now_ms),
        DefenseShieldState::Mana(state) => state.lifetime_expired(now_ms),
        DefenseShieldState::Promotion(state) => state.expired(now_ms),
    } || (!matches!(state, DefenseShieldState::Promotion(_))
        && game.move_shape_health(region_id, holder).is_some_and(|health| health == 0));
    if !expired {
        return false;
    }
    end_defense_shield(game, region_id, holder, key)
}

pub(crate) fn end_defense_shield(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.defense_shield(key)).copied() else { return false };
    match state {
        DefenseShieldState::Life(state) => {
            finish_life_shield_state_for_holder(game, region_id, holder, state);
        }
        DefenseShieldState::Machine(_) | DefenseShieldState::Mana(_) => {
            let mut message = CMessage::new(super::manashieldstate::MANA_SHIELD_STATE_END_MESSAGE);
            message.add_long(holder.object_type);
            message.add_long(holder.id);
            message.add_long(state.skill_id() as i32);
            let _ = game.send_move_shape_around(region_id, holder, &message);
        }
        DefenseShieldState::Promotion(_) => {}
    }
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_defense_shield_key(key)).is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    removed
}

pub(crate) fn expire_player_defense_shield(
    game: &mut CGame,
    player_id: i32,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = game.find_player(player_id).is_some_and(|player| {
        player.defense_shield(key).is_some_and(|state| state.expired(
            now_ms,
            player.mana(),
            player.is_dead(),
            player.war_soul_mana(game.goods_factory()),
        ))
    });
    expired && end_player_defense_shield_key(game, player_id, key, now_ms)
}

pub(crate) fn end_player_defense_shield(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    now_ms: u32,
) -> bool {
    let Some(key) = game.find_player(player_id)
        .and_then(|player| player.defense_shield_key(skill_id))
    else {
        return false;
    };
    end_player_defense_shield_key(game, player_id, key, now_ms)
}

pub(crate) fn end_player_defense_shield_key(
    game: &mut CGame,
    player_id: i32,
    key: StateKey,
    _now_ms: u32,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false };
    let region_id = player.shape().get_region_id();
    let holder = player.shape().identity();
    end_defense_shield(game, region_id, holder, key)
}
