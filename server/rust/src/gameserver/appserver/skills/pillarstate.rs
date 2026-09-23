//! Живое переключение, visual и End защитной стойки CPillarState.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/pillarstate.cpp/.h;
//! данные, срок и сохраняемая запись находятся в Zone effects.
//! Повторный каст снимает первый ID74; Begin запрещает движение после visual.
//! End ищет фактического Sufferer и снимает его запрет движения перед удалением.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    end_and_destroy_state_at, remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut, update_applied_state_end_visual,
    update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;
pub(crate) use nebokrai_zone::effects::{PILLAR_STATE_BYTES, PILLAR_STATE_ID, PillarState};
pub(crate) fn toggle_pillar_state(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce(&CGame) -> Option<PillarState>, now: &mut dyn FnMut() -> u32,
) -> bool {
    let previous = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == PILLAR_STATE_ID));
    if let Some((position, _)) = previous {
        return end_and_destroy_state_at(game, source.0, source.1, position).is_some();
    }
    let Some(mut state) = create(game) else { return false; };
    let begun = (|| {
        resolve_state_move_shape(game, source.0, source.1)?;
        state.begin_at(now());
        let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
        let participant = (shape.get_region_id(), ShapeIdentity {
            ex_id: CGuid::GUID_INVALID, ..shape.identity()
        });
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(shape.identity().object_type);
        message.add_long(shape.identity().id);
        message.add_ulong(PILLAR_STATE_ID);
        message.add_long(state.client_time(&mut *now));
        message.add_ulong(0);
        let _ = game.send_move_shape_around(participant.0, participant.1, &message);
        // Объектный Begin запрещает движение исходному param_2 после visual.
        let shape = resolve_state_move_shape_mut(game, source.0, source.1)?;
        shape.set_moveable(false);
        let record = state.encoded_for_install();
        let key = shape.append_applied_state_record(state, &record);
        shape.mark_applied_state_begun(key);
        shape.set_applied_state_user(key, Some(participant));
        shape.set_applied_state_sufferer(key, Some(participant));
        Some(())
    })().is_some();
    let _ = game.update_move_shape_properties(source.0, source.1);
    begun
}

pub(crate) fn restart_pillar_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key)).is_none() { return false; }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<PillarState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.client_time(now) as u32,
        );
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_moveable(false);
    }
    true
}

pub(crate) fn update_pillar_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_pillar_state(game, region_id, holder, key)
}

pub(crate) fn end_pillar_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key)).is_none() { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return false; };
    if let Some(shape) = resolve_state_move_shape_mut(game, target.0, target.1) {
        shape.set_moveable(true);
    }
    remove_applied_state_from(game, region_id, holder, key, target, PILLAR_STATE_BYTES)
}
