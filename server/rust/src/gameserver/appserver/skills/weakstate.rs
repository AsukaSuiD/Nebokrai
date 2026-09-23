//! Живые операции ослабления CWeakState (0x12e).
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/weakstate.cpp/.h`.
//! Запись, прямоугольник, срок и player-формула находятся в `zone/effects/weak.rs`.
//! Здесь остаются разрешение S, visual, применение результата, restart и End.
//! Begin требует S; при NULL U не обновляет часы. AI типа 2 читает положение
//! живой цели, а смена региона завершает это состояние.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, remove_applied_state_from,
    resolve_applied_state_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
    update_applied_state_end_visual, update_property_state_visual,
    StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{WEAK_STATE_BYTES, WEAK_STATE_ID, WeakState};

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub(crate) fn begin_primary_weak_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: WeakState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    resolve_state_move_shape(game, holder_region, holder)?;
    resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
    if user.is_some() { state.start_at(now()); }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, holder_region, holder)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

pub(crate) fn update_weak_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none()
    { return false; }
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<WeakState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).cloned()
    else { return false; };
    match target.object_type {
        600 => {
            if let Some(monster) = game.find_region_mut(target_region)
                .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id))
            {
                let modifiers = monster.move_shape_mut().property_modifiers_mut();
                let (minimum_attack, maximum_attack) = state.apply_to_monster_attacks(
                    modifiers.minimum_attack, modifiers.maximum_attack,
                );
                modifiers.minimum_attack = minimum_attack;
                modifiers.maximum_attack = maximum_attack;
            }
        }
        400 => {
            let Some(properties) = game.find_player(target.id).map(|player| player.combat_properties())
            else { return false; };
            let (minimum_attack, maximum_attack) = state.apply_to_player_attacks(
                properties.minimum_attack, properties.maximum_attack,
            );
            let updated = PlayerCombatProperties {
                minimum_attack, maximum_attack, ..properties
            };
            if let Some(player) = game.find_player_mut(target.id) {
                player.update_state_combat_properties(|_| updated);
            }
        }
        _ => {}
    }
    true
}

pub(crate) fn restart_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none() {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(shape) = resolve_state_move_shape(game, region_id, holder) else { return false };
    let Some(state) = shape.applied_state::<WeakState>(key) else { return false };
    match state.state_type() {
        1 => {
            if !state.expired(now()) {
                return false;
            }
        }
        2 => {
            let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
            else { return false; };
            let Some(target) = resolve_state_move_shape(game, target_region, target)
            else { return false; };
            let x = target.shape().get_tile_x().unwrap_or(i32::MIN);
            let y = target.shape().get_tile_y().unwrap_or(i32::MIN);
            if state.contains(x, y) {
                return false;
            }
        }
        _ => {}
    }
    end_weak_state(game, region_id, holder, key)
}

pub(crate) fn set_weak_state_region(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) {
    let Some(state_type) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key))
        .map(|state| state.state_type())
    else { return; };
    if state_type == 2 {
        let _ = end_weak_state(game, region_id, holder, key);
    } else if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        let _ = shape.set_applied_state_sufferer_region(key, region_id);
    }
}

pub(crate) fn end_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(
        game, region_id, holder, key, StatePropertyTarget::Sufferer,
    );
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(
        game, region_id, holder, key, (target_region, target), WEAK_STATE_BYTES,
    )
}
