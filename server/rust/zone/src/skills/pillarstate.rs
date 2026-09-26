//! Живое переключение, visual и End защитной стойки CPillarState в Zone.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/pillarstate.cpp/.h`. Машинные якоря:
//! Begin состояния `0x1F4B60`; данные, срок и сохраняемая запись —
//! `effects/pillar.rs`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/pillarstate.rs`; тела перенесены
//! буквально порцией №6c «self/zone-касты» (разведка — запись аудита
//! «Zone skills: машинная разведка battlefairy-навыков (порция №6)»,
//! 26 сентября 2026).
//!
//! Повторный каст снимает первый ID74; Begin запрещает движение после visual.
//! End ищет фактического Sufferer и снимает его запрет движения перед удалением.
//!
//! Объявленные швы переноса (не расхождения): hub `selfcast::SelfCastGame`
//! реализован у прежнего владельца; обвязка арены (`begin_base_applied_state`,
//! `begin_applied_state_visual`, `update_property_state_visual`,
//! `update_applied_state_end_visual`, `resolve_applied_state_sufferer`,
//! `remove_applied_state_from`, `end_and_destroy_state_at`) остаётся у
//! прежнего `states/state.rs` и объявлена одноимёнными методами трейта.

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::regions::ShapeIdentity;

use super::selfcast::{SelfCastGame, SelfCastMoveShape, SelfCastPropertyTarget};
use super::state::StateKey;

pub use crate::effects::{PILLAR_STATE_BYTES, PILLAR_STATE_ID, PillarState};

pub fn toggle_pillar_state<Game: SelfCastGame>(
    game: &mut Game, source: (i32, ShapeIdentity),
    create: impl FnOnce(&Game) -> Option<PillarState>, now: &mut dyn FnMut() -> u32,
) -> bool {
    let previous = game.resolve_state_move_shape(source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == PILLAR_STATE_ID));
    if let Some((position, _)) = previous {
        return game.end_and_destroy_state_at(source.0, source.1, position);
    }
    let Some(mut state) = create(game) else { return false; };
    let begun = (|| {
        game.resolve_state_move_shape(source.0, source.1)?;
        state.begin_at(now());
        let shape = game.resolve_state_move_shape(source.0, source.1)?.shape();
        let participant = (shape.get_region_id(), ShapeIdentity {
            ex_id: CGuid::GUID_INVALID, ..shape.identity()
        });
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(shape.identity().object_type);
        message.add_long(shape.identity().id);
        message.add_ulong(PILLAR_STATE_ID);
        message.add_long(state.client_time(&mut *now));
        message.add_ulong(0);
        game.send_move_shape_around(participant.0, participant.1, &message);
        // Объектный Begin запрещает движение исходному param_2 после visual.
        let shape = game.resolve_state_move_shape_mut(source.0, source.1)?;
        shape.set_moveable(false);
        let record = state.encoded_for_install();
        let key = shape.append_applied_state_record(state, &record);
        shape.mark_applied_state_begun(key);
        shape.set_applied_state_user(key, Some(participant));
        shape.set_applied_state_sufferer(key, Some(participant));
        Some(())
    })().is_some();
    game.update_move_shape_properties(source.0, source.1);
    begun
}

pub fn restart_pillar_state<Game: SelfCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key)).is_none() { return false; }
    if !game.begin_base_applied_state(region_id, holder, key) {
        return false;
    }
    if game.begin_applied_state_visual(region_id, holder, key, 1) {
        game.update_property_state_visual::<PillarState>(
            region_id, holder, key, SelfCastPropertyTarget::Sufferer, now,
            |state, now| state.client_time(now) as u32,
        );
    }
    if let Some(shape) = game.resolve_state_move_shape_mut(region_id, holder) {
        shape.set_moveable(false);
    }
    true
}

pub fn update_pillar_state<Game: SelfCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_pillar_state(game, region_id, holder, key)
}

pub fn end_pillar_state<Game: SelfCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key)).is_none() { return false; }
    game.update_applied_state_end_visual(region_id, holder, key, SelfCastPropertyTarget::Sufferer);
    let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key) else { return false; };
    if let Some(shape) = game.resolve_state_move_shape_mut(target.0, target.1) {
        shape.set_moveable(true);
    }
    game.remove_applied_state_from(region_id, holder, key, target, PILLAR_STATE_BYTES)
}
