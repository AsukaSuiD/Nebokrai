//! Живые callbacks CGodBlessState/CGodBlessState2.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/skills/godblessstate{,2}.cpp/.h.
//! Данные, срок, codec и числовые правила находятся в Zone effects.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    resolve_applied_state_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
    update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
pub(crate) use nebokrai_zone::effects::{GOD_BLESS_STATE_BYTES, GOD_BLESS_STATE_ID, GodBlessState};
pub(crate) fn update_god_bless_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<GodBlessState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        let (minimum_gain, maximum_gain, element_gain) = state.monster_gains();
        if let Some(monster) = game.find_region_mut(target_region)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.minimum_attack = modifiers.minimum_attack.wrapping_add(minimum_gain);
            modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(maximum_gain);
            modifiers.element_modify = modifiers.element_modify.wrapping_add(element_gain);
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                (properties.minimum_attack, properties.maximum_attack, properties.element_modify) =
                    state.player_gains(
                        properties.minimum_attack, properties.maximum_attack, properties.element_modify,
                    );
                properties
            });
        }
    }
    true
}

pub(crate) fn restart_god_bless_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key)).copied()
        else { return false };
    if state.skill_id() != GOD_BLESS_STATE_ID {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_god_bless_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_god_bless_state(game, region_id, holder, key)
}

pub(crate) fn end_god_bless_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key)).copied()
        else { return false };
    if state.skill_id() == GOD_BLESS_STATE_ID {
        crate::gameserver::appserver::states::state::update_applied_state_end_visual(
            game, region_id, holder, key, StatePropertyTarget::Sufferer,
        );
    }
    let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) else { return false };
    shape.mark_applied_state_ended(key);
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return false };
    crate::gameserver::appserver::states::state::remove_applied_state_from(
        game, region_id, holder, key, (target_region, target), GOD_BLESS_STATE_BYTES,
    )
}
