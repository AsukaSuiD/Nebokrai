//! Живой путь CCallosityState/CCallosityState2 (0x75/0x7d) в Zone.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/callositystate.cpp/.h` и
//! `callositystate2.cpp/.h`; машинно методы пары почти полностью folded
//! (9 методов, Restart-fold с CPromotionState `0x1FD450`). Данные, срок,
//! запись и формула — `effects/callosity.rs`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/callositystate.rs`; тела перенесены
//! буквально порцией №6c «self/zone-касты» (разведка — запись аудита
//! «Zone skills: машинная разведка battlefairy-навыков (порция №6)»,
//! 26 сентября 2026).
//!
//! Здесь замена первого состояния семейства, участники, visual, restart/End
//! и вызов общего пересчёта. Begin берёт часы до публикации visual;
//! UpdateProperty выполняется после попытки Begin независимо от результата.
//!
//! Объявленные швы переноса (не расхождения): hub `selfcast::SelfCastGame`
//! реализован у прежнего владельца; обвязка арены (`end_and_destroy_state_at`,
//! `begin_base_applied_state`, `begin_applied_state_visual`,
//! `update_property_state_visual`, `update_applied_state_end_visual`,
//! `resolve_applied_state_sufferer`, `remove_applied_state_from`) остаётся у
//! прежнего `states/state.rs` и объявлена одноимёнными методами трейта.

use crate::app::game_message::CMessage;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::callosity::{CALLOSITY_2_SKILL_ID, CALLOSITY_SKILL_ID};
use super::selfcast::{
    SelfCastGame, SelfCastMoveShape, SelfCastPlayer, SelfCastPropertyTarget,
    selfcast_storage_participant,
};
use super::state::StateKey;

pub use crate::effects::{CALLOSITY_STATE_BYTES, CallosityFamilyState};

pub fn replace_callosity_state<Game: SelfCastGame>(
    game: &mut Game, source: (i32, ShapeIdentity),
    create: impl FnOnce() -> CallosityFamilyState, now: &mut dyn FnMut() -> u32,
) -> bool {
    let previous = game.resolve_state_move_shape(source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| {
            matches!(state.state_id(), CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID)
        }));
    if let Some((position, _)) = previous {
        let _ = game.end_and_destroy_state_at(source.0, source.1, position);
    }
    let mut state = create();
    let begun = (|| {
        game.resolve_state_move_shape(source.0, source.1)?;
        state.start_at(now());
        let user = selfcast_storage_participant(game, source)?;
        let sufferer = selfcast_storage_participant(game, source)?;
        if game.resolve_state_move_shape(sufferer.0, sufferer.1).is_some() {
            let mut message = CMessage::new(0x000b_fe03);
            message.add_long(sufferer.1.object_type);
            message.add_long(sufferer.1.id);
            message.add_ulong(state.skill_id());
            message.add_long(state.client_state_time(&mut *now));
            message.add_ulong(0);
            game.send_move_shape_around(sufferer.0, sufferer.1, &message);
        }
        let record = state.encoded_for_install();
        let shape = game.resolve_state_move_shape_mut(source.0, source.1)?;
        let key = shape.append_applied_state_record(state, &record);
        shape.mark_applied_state_begun(key);
        shape.set_applied_state_user(key, Some(user));
        shape.set_applied_state_sufferer(key, Some(sufferer));
        shape.begin_applied_state_visual(key, 1);
        shape.update_applied_state_visual_base(key);
        Some(())
    })().is_some();
    game.update_move_shape_properties(source.0, source.1);
    begun
}

pub fn end_callosity_state_key<Game: SelfCastGame>(
    game: &mut Game, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).is_none()
    { return false; }
    game.update_applied_state_end_visual(region_id, holder, key, SelfCastPropertyTarget::Sufferer);
    let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    game.remove_applied_state_from(region_id, holder, key, target, CALLOSITY_STATE_BYTES)
}

pub fn update_callosity_state_properties<Game: SelfCastGame>(
    game: &mut Game, region_id: i32, holder: ShapeIdentity, key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    let _ = game.update_property_state_visual::<CallosityFamilyState>(
        region_id, holder, key, SelfCastPropertyTarget::Sufferer, now,
        |state, now| state.client_state_time(now) as u32,
    );
    if target.object_type == PLAYER_TYPE {
        let Some(state) = game.resolve_state_move_shape(region_id, holder)
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

pub fn restart_callosity_state<Game: SelfCastGame>(
    game: &mut Game, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).is_none()
    { return false; }
    let Some(sufferer) = selfcast_storage_participant(game, (region_id, holder)) else { return false; };
    if !game.begin_base_applied_state(region_id, holder, key) { return false; }
    if let Some(shape) = game.resolve_state_move_shape_mut(region_id, holder) {
        shape.set_applied_state_sufferer(key, Some(sufferer));
    }
    if game.begin_applied_state_visual(region_id, holder, key, 1) {
        game.update_property_state_visual::<CallosityFamilyState>(
            region_id, holder, key, SelfCastPropertyTarget::Sufferer, now,
            |state, now| state.client_state_time(now) as u32,
        );
    }
    true
}

pub fn update_callosity_state<Game: SelfCastGame>(
    game: &mut Game, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    if !game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_callosity_state_key(game, region_id, holder, key)
}
