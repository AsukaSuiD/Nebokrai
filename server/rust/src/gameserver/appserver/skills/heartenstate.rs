//! Применение и снятие Hearten у живой фигуры Game.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/heartenstate.cpp` и `heartenstate.h`.
//! Данные, срок и формула прибавки находятся в `zone/effects/hearten.rs`;
//! участники, visual и запись результата игроку остаются здесь.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut, update_applied_state_end_visual,
    update_player_state_properties, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{HEARTEN_STATE_BYTES, HeartenState};

pub(crate) fn begin_primary_hearten_state(
    game: &mut CGame,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: (i32, ShapeIdentity),
    mut state: HeartenState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    if user.is_some() { state.begin_at(now()); }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, sufferer.0, sufferer.1)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    // Loop1 создаёт общий каталог. Между Begin и append нет внешнего callback;
    // первый UpdateVisualEffect принадлежит последующему UpdateProperty.
    Some(key)
}

pub(crate) fn update_hearten_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
    key: StateKey, now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, sufferer)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<HeartenState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    if sufferer.object_type != 400 { return true; }
    update_player_state_properties::<HeartenState>(game, region_id, holder, key, |state, player| {
        player.update_state_combat_properties(|mut properties| {
            properties.maximum_hp = state.apply(properties.maximum_hp);
            properties
        });
    })
}

pub(crate) fn restart_hearten_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HeartenState>(key)).is_none()
    { return false; }
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_hearten_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HeartenState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_hearten_state(game, region_id, holder, key)
}

pub(crate) fn end_hearten_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HeartenState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(sufferer) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, sufferer, HEARTEN_STATE_BYTES)
}
