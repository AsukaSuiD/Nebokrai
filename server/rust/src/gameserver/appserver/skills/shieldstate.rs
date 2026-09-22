//! Защитная ветвь CFightDefense::PreDefense: gameserver.exe + GameServer.pdb,
//! appserver/skills/{life,mana,machine}shieldstate.cpp, promotionstate.cpp и moveshape.cpp.
//! Общая арена SlotMap сохраняет identity, порядок щитов и пропуски m_vStates.
//! Чистый PreDefense временно выделяет payload в keyed batch и возвращает его
//! до callbacks. End выполняет эффект, удаляет прежний ключ и обновляет свойства;
//! Life сначала создаёт Cure, Machine/Mana отправляют End-пакет, Promotion — нет.
//! AI Life/Mana/Machine проверяет deadline, signed life и смерть перед ресурсами.
//! Их ресурсная фаза требует CPlayer: чужой unchecked MP/layout не имитируется.
//! Первичное наложение само по себе допускает полный CMoveShape.
//! Promotion проверяет только собственный срок.
//! Первичный Life/Mana/Machine Begin читает базовые часы при U, отправляет visual и
//! добавляет состояние в арену; UpdateProperty остаётся caller-у. Restart после
//! Unserialize использует Begin(NULL, holder) и сохраняет уже прочитанный старт.
//! Object Begin требует S; visual loop1 общий, у Promotion loop0 завершается сразу.

use super::lifeshieldstate::{add_life_shield_cure, LifeShieldState};
use super::machineshieldstate::MachineShieldState;
use super::manashieldstate::ManaShieldState;
use super::promotionstate::PromotionState;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackPower;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut, update_applied_state_end_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

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

/// Первичный object Begin щитов Life/Mana/Machine: S guard → base clock при U →
/// visual loop1 → пакет → append. DB-record технический и часов не читает.
pub(crate) fn begin_primary_self_shield_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: DefenseShieldState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    if matches!(state, DefenseShieldState::Promotion(_)) { return None; }
    resolve_state_move_shape(game, holder_region, holder)?;
    resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
    if user.is_some() {
        let started = now();
        match &mut state {
            DefenseShieldState::Life(state) => state.begin_at(started),
            DefenseShieldState::Mana(state) => state.begin_at(started),
            DefenseShieldState::Machine(state) => state.begin_at(started),
            _ => unreachable!("первичный owner проверен до Begin"),
        }
    }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let message = shield_begin_message(sufferer.1, state, now);
    let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
    let record = match state {
        DefenseShieldState::Life(state) => state.encoded_for_install().to_vec(),
        DefenseShieldState::Mana(state) => state.encoded_for_install().to_vec(),
        DefenseShieldState::Machine(state) => state.encoded_for_install().to_vec(),
        _ => unreachable!("первичный owner проверен до Begin"),
    };
    let shape = resolve_state_move_shape_mut(game, holder_region, holder)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.begin_applied_state_visual(key, 1);
    shape.update_applied_state_visual_base(key);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

fn shield_begin_message(
    sufferer: ShapeIdentity, state: DefenseShieldState, now: &mut dyn FnMut() -> u32,
) -> CMessage {
    let mut message = CMessage::new(super::manashieldstate::MANA_SHIELD_STATE_BEGIN_MESSAGE);
    message.add_long(sufferer.object_type);
    message.add_long(sufferer.id);
    message.add_long(state.skill_id() as i32);
    let (time, life) = match state {
        DefenseShieldState::Life(state) => (state.client_time(&mut *now), state.life()),
        DefenseShieldState::Machine(state) => (state.client_time(&mut *now), state.life()),
        DefenseShieldState::Mana(state) => (state.client_time(&mut *now), state.life()),
        DefenseShieldState::Promotion(state) => (state.client_time(&mut *now), 0),
    };
    message.add_long(time);
    message.add_long(life);
    message
}

pub(crate) fn restart_defense_shield_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.defense_shield(key)).copied()
    else { return false };
    if !crate::gameserver::appserver::states::state::begin_base_applied_state(
        game, region_id, holder, key,
    ) { return false }
    let loop_value = if matches!(state, DefenseShieldState::Promotion(_)) { 0 } else { 1 };
    if crate::gameserver::appserver::states::state::begin_applied_state_visual(
        game, region_id, holder, key, loop_value,
    ) {
        let message = shield_begin_message(holder, state, now);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = crate::gameserver::appserver::states::state::update_applied_state_visual_base(
            game, region_id, holder, key,
        );
    }
    true
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
    let bytes = match state {
        DefenseShieldState::Life(state) => {
            add_life_shield_cure(game, region_id, holder, state, key);
            super::lifeshieldstate::LIFE_SHIELD_STATE_BYTES
        }
        DefenseShieldState::Machine(_) => super::machineshieldstate::MACHINE_SHIELD_STATE_BYTES,
        DefenseShieldState::Mana(_) => super::manashieldstate::MANA_SHIELD_STATE_BYTES,
        DefenseShieldState::Promotion(_) => {
            let removed = resolve_state_move_shape_mut(game, region_id, holder)
                .and_then(|shape| shape.remove_defense_shield_key(key)).is_some();
            if removed { let _ = game.update_move_shape_properties(region_id, holder); }
            return removed;
        }
    };
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(sufferer) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return false; };
    remove_applied_state_from(game, region_id, holder, key, sufferer, bytes)
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
