//! Наложение, обновление и снятие ядовитого тумана у живой фигуры Game.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/poisonfogstate.cpp` и `poisonfogstate.h`.
//! Данные, запись и расчёт потерь находятся в Zone; здесь остаются участники,
//! visual, поиск фигуры и запись рассчитанных свойств.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, remove_applied_state_from,
    resolve_applied_state_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
    update_applied_state_end_visual, update_property_state_visual,
    StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{POISON_FOG_STATE_BYTES, POISON_FOG_STATE_ID, PoisonFogState};

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub(crate) fn begin_primary_poison_fog_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: PoisonFogState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    resolve_state_move_shape(game, holder_region, holder)?;
    resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
    if user.is_some() { state.begin_at(now()); }
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


pub(crate) fn update_poison_fog_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    let Some((target_region, target)) = resolve_applied_state_sufferer(
        game, region_id, holder, key,
    ) else { return false };
    let target_level = match target.object_type {
        400 => {
            let Some(player) = game.find_player(target.id) else { return false };
            player.level()
        }
        600 => {
            let Some(monster) = game.find_region(target_region)
                .and_then(|region| region.base().find_monster_by_id(target.id))
            else { return false };
            let Some(properties) = monster.base_property_key()
                .and_then(|name| game.find_monster_property_by_origin_name(name))
            else { return false };
            properties.level as u8
        }
        500 | 1100 | 1200 => 1,
        _ => return false,
    };
    let _ = update_property_state_visual::<PoisonFogState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key))
    else { return false };
    match target.object_type {
        400 => {
            let Some(properties) = game.find_player(target.id).map(|player| player.combat_properties())
            else { return false };
            let (defense, element_resistance) = state.player_properties(
                target_level, properties.defense, properties.element_resistance,
            );
            let mut updated = properties;
            updated.defense = defense;
            updated.element_resistance = element_resistance;
            let Some(player) = game.find_player_mut(target.id) else { return false };
            player.update_state_combat_properties(|_| updated);
        }
        600 => {
            let (defense, resistance) = state.monster_losses(target_level);
            let Some(shape) = resolve_state_move_shape_mut(game, target_region, target)
            else { return false };
            let modifiers = shape.property_modifiers_mut();
            modifiers.defense = modifiers.defense.wrapping_sub(defense as i32);
            modifiers.element_resistance = modifiers.element_resistance
                .wrapping_sub(resistance as i32);
        }
        _ => {}
    }
    true
}

pub(crate) fn restart_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(
        game, region_id, holder, key,
    ) { return false }
    let _ = begin_applied_state_visual(
        game, region_id, holder, key, 1,
    );
    true
}

pub(crate) fn update_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key))
        .is_some_and(|state| state.expired(now_ms));
    if !expired { return false }
    end_poison_fog_state(game, region_id, holder, key)
}

pub(crate) fn end_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    update_applied_state_end_visual(
        game, region_id, holder, key, StatePropertyTarget::Sufferer,
    );
    let Some((target_region, target)) = resolve_applied_state_sufferer(
        game, region_id, holder, key,
    ) else { return false };
    remove_applied_state_from(
        game, region_id, holder, key, (target_region, target), POISON_FOG_STATE_BYTES,
    )
}

// Для координатного Begin (0x00607E30) и typed Begin (0x00607EC0)
// вызывающие цепочки не установлены; основной путь использует объектный Begin.
