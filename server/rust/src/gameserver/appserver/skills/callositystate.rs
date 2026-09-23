//! Живой путь CCallosityState/CCallosityState2 (0x75/0x7d).
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/callositystate.cpp/.h` и `callositystate2.cpp/.h`.
//! Данные, срок, запись и формула находятся в `zone/effects/callosity.rs`.
//! Здесь остаются замена первого состояния семейства, участники, visual,
//! restart/End и вызов общего пересчёта. Begin берёт часы до публикации visual;
//! UpdateProperty выполняется после попытки Begin независимо от результата.

use super::callosity::CALLOSITY_SKILL_ID;
use super::callosity2::CALLOSITY_2_SKILL_ID;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    end_and_destroy_state_at, remove_applied_state_from,
    resolve_applied_state_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
    update_applied_state_end_visual, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{CALLOSITY_STATE_BYTES, CallosityFamilyState};

fn participant(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..shape.identity()
    }))
}

pub(crate) fn replace_callosity_state(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce() -> CallosityFamilyState, now: &mut dyn FnMut() -> u32,
) -> bool {
    let previous = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| {
            matches!(state.state_id(), CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID)
        }));
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, source.0, source.1, position);
    }
    let mut state = create();
    let begun = (|| {
        resolve_state_move_shape(game, source.0, source.1)?;
        state.start_at(now());
        let user = participant(game, source)?;
        let sufferer = participant(game, source)?;
        if resolve_state_move_shape(game, sufferer.0, sufferer.1).is_some() {
            let mut message = CMessage::new(0x000b_fe03);
            message.add_long(sufferer.1.object_type);
            message.add_long(sufferer.1.id);
            message.add_ulong(state.skill_id());
            message.add_long(state.client_state_time(&mut *now));
            message.add_ulong(0);
            let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
        }
        let record = state.encoded_for_install();
        let shape = resolve_state_move_shape_mut(game, source.0, source.1)?;
        let key = shape.append_applied_state_record(state, &record);
        shape.mark_applied_state_begun(key);
        shape.set_applied_state_user(key, Some(user));
        shape.set_applied_state_sufferer(key, Some(sufferer));
        shape.begin_applied_state_visual(key, 1);
        shape.update_applied_state_visual_base(key);
        Some(())
    })().is_some();
    let _ = game.update_move_shape_properties(source.0, source.1);
    begun
}

pub(crate) fn end_callosity_state_key(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, CALLOSITY_STATE_BYTES)
}

pub(crate) fn update_callosity_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<CallosityFamilyState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_state_time(now) as u32,
    );
    if target.object_type == 400 {
        let Some(state) = resolve_state_move_shape(game, region_id, holder)
            .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).copied()
        else { return false; };
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                properties.blast_attack = state.apply_to_blast_attack(properties.blast_attack);
                properties
            });
        }
    }
    true
}

pub(crate) fn restart_callosity_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).is_none()
    { return false; }
    let Some(sufferer) = participant(game, (region_id, holder)) else { return false; };
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_applied_state_sufferer(key, Some(sufferer));
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<CallosityFamilyState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.client_state_time(now) as u32,
        );
    }
    true
}

pub(crate) fn update_callosity_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_callosity_state_key(game, region_id, holder, key)
}
