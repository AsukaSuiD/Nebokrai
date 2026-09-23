//! Применение постоянных состояний Swordship к живым фигурам Game.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/swordshipstate{,2,3,4}.cpp/.h`.
//! Данные, запись и формулы игрока — в `zone/effects/swordship.rs`.
//! Callback разрешает S, затем применяет MIN перед MAX. End удаляет запись
//! через актуального U; restart Begin(NULL, S) сохраняет прежнего U.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::resolve_applied_state_sufferer;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
pub(crate) use nebokrai_zone::effects::{SWORDSHIP_STATE_BYTES, SwordshipState};

pub(crate) fn update_swordship_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) =
        resolve_applied_state_sufferer(game, region_id, holder, key)
    else {
        return false;
    };
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<SwordshipState>(key))
        .copied()
    else {
        return false;
    };
    if target.object_type == 600 {
        if let Some(monster) = game
            .find_region_mut(target_region)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id))
        {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.minimum_attack = modifiers
                .minimum_attack
                .wrapping_add(state.minimum_attack_gain());
            modifiers.maximum_attack = modifiers
                .maximum_attack
                .wrapping_add(state.maximum_attack_gain());
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                properties.minimum_attack = state.apply_player_minimum(properties.minimum_attack);
                properties.maximum_attack = state.apply_player_maximum(properties.maximum_attack);
                properties
            });
        }
    }
    true
}

pub(crate) fn restart_swordship_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<SwordshipState>(key))
        .is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
}

pub(crate) fn end_swordship_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<SwordshipState>(key))
        .is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, SWORDSHIP_STATE_BYTES)
}
