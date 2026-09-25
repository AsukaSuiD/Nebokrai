//! Живое наложение временной Agility2 (0x81).
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/agilitystate2.cpp/.h` и `agility2.cpp`.
//! Данные, запись и расчёт срока принадлежат `zone/effects/agility.rs`.
//! Здесь остаются замена первого ID81, участники, visual, restart и End.
//! Begin читает часы и публикует BFE03 до append; общий пересчёт выполняется
//! после попытки Begin независимо от результата.
//! MATCH по разведке порции №4: visual Begin временной Agility2 — loop0
//! (константа zone `skills/selfstate.rs`), BFE03/BFE04 со свежим клиентским
//! остатком и extra=0.

use super::agility2::AGILITY_2_SKILL_ID;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    end_and_destroy_state_at, remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
    update_player_state_properties, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;
use nebokrai_zone::skills::AGILITY_2_VISUAL_LOOP;

pub(crate) use nebokrai_zone::effects::{AGILITY_STATE_2_BYTES, AgilityState2};

fn participant(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..shape.identity()
    }))
}

pub(crate) fn replace_agility_state_2(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce() -> AgilityState2, now: &mut dyn FnMut() -> u32,
) -> bool {
    let previous = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == AGILITY_2_SKILL_ID));
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
            message.add_long(state.client_time(&mut *now));
            message.add_ulong(0);
            let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
        }
        let record = state.encoded_for_install();
        let shape = resolve_state_move_shape_mut(game, source.0, source.1)?;
        let key = shape.append_applied_state_record(state, &record);
        shape.mark_applied_state_begun(key);
        shape.set_applied_state_user(key, Some(user));
        shape.set_applied_state_sufferer(key, Some(sufferer));
        shape.begin_applied_state_visual(key, AGILITY_2_VISUAL_LOOP);
        shape.update_applied_state_visual_base(key);
        Some(())
    })().is_some();
    let _ = game.update_move_shape_properties(source.0, source.1);
    begun
}

pub(crate) fn update_agility_state_2_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    update_player_state_properties::<AgilityState2>(game, region_id, holder, key, |state, player| {
        player.update_state_combat_properties(|mut properties| {
            properties.full_miss = state.apply_to_full_miss(properties.full_miss);
            properties
        });
    })
}

pub(crate) fn restart_agility_state_2(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AgilityState2>(key)).is_none()
    { return false; }
    let Some(sufferer) = participant(game, (region_id, holder)) else { return false; };
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_applied_state_sufferer(key, Some(sufferer));
    }
    if begin_applied_state_visual(game, region_id, holder, key, AGILITY_2_VISUAL_LOOP) {
        update_property_state_visual::<AgilityState2>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.client_time(now) as u32,
        );
    }
    true
}

pub(crate) fn update_agility_state_2(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AgilityState2>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_agility_state_2(game, region_id, holder, key)
}

pub(crate) fn end_agility_state_2(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AgilityState2>(key)).is_none()
    { return false; }
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, AGILITY_STATE_2_BYTES)
}
