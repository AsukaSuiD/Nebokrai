//! Живые операции состояния ослабления CWeakState (0x12E).
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/weakstate.cpp/.h`. Запись, прямоугольник, срок и
//! player/monster-формулы находятся в `effects/weak.rs`; здесь — разрешение
//! участников S/U, visual, применение результата, restart, ветви AI и End.
//! Арена и визуализация остаются швами `ZonalCastGame`
//! (`skills/zonalcast.rs`).
//! Begin требует S; при NULL U не обновляет часы. AI типа 2 читает
//! положение живой цели, а смена региона завершает это состояние.
//! Запись Player-свойств и монстровых модификаторов цели — прежние операции
//! владельца; отсутствие цели не прерывает верхнее обновление.

use nebokrai_shared::values::CGuid;

use crate::combat::PlayerCombatProperties;
use crate::effects::{WeakState, WEAK_STATE_BYTES};
use crate::regions::ShapeIdentity;

use super::state::StateKey;
use super::zonalcast::{ZonalCastGame, ZonalCastMoveShape, ZonalCastPlayer, ZonalCastPropertyTarget};

/// Объектный Begin первичного экземпляра: holder и S обязаны разрешиться,
/// участники перечитываются живыми (ex_id сброшен), запись арены —
/// `encoded_for_install` прежней кодировки.
pub fn begin_primary_weak_state<Game: ZonalCastGame>(
    game: &mut Game,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: WeakState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    game.resolve_state_move_shape(holder_region, holder)?;
    game.resolve_state_move_shape(sufferer.0, sufferer.1)?;
    if user.is_some() { state.start_at(now()); }
    let participant = |(region, identity)| {
        let shape = game.resolve_state_move_shape(region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let record = state.encoded_for_install();
    let shape = game.resolve_state_move_shape_mut(holder_region, holder)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

/// Property callback: visual по sufferer, затем пересчёт живых свойств
/// цели. Монстр допускает молчаливое отсутствие записи региона, player —
/// перечитку через снимок combat properties и ту же замену.
pub fn update_weak_state_properties<Game: ZonalCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none()
    { return false; }
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    let _ = game.update_property_state_visual::<WeakState>(
        region_id, holder, key, ZonalCastPropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).cloned()
    else { return false; };
    match target.object_type {
        600 => {
            let _ = game.apply_weak_monster_attacks(target_region, target.id, &state);
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

/// Restart прежнего экземпляра: повторный base Begin и visual loop1.
pub fn restart_weak_state<Game: ZonalCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none() {
        return false;
    }
    if !game.begin_base_applied_state(region_id, holder, key) {
        return false;
    }
    let _ = game.begin_applied_state_visual(region_id, holder, key, 1);
    true
}

/// Ветви AI прежнего CWeakState: тип 1 завершается по строгому сроку,
/// тип 2 — когда живая цель покидает прямоугольник.
pub fn update_weak_state<Game: ZonalCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(shape) = game.resolve_state_move_shape(region_id, holder) else { return false };
    let Some(state) = shape.applied_state::<WeakState>(key) else { return false };
    match state.state_type() {
        1 => {
            if !state.expired(now()) {
                return false;
            }
        }
        2 => {
            let Some((target_region, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
            else { return false; };
            let Some(target) = game.resolve_state_move_shape(target_region, target)
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

/// Смена региона держателя: тип 2 завершается, остальные просто следуют за
/// sufferer-регионом.
pub fn set_weak_state_region<Game: ZonalCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) {
    let Some(state_type) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key))
        .map(|state| state.state_type())
    else { return; };
    if state_type == 2 {
        let _ = end_weak_state(game, region_id, holder, key);
    } else if let Some(shape) = game.resolve_state_move_shape_mut(region_id, holder) {
        let _ = shape.set_applied_state_sufferer_region(key, region_id);
    }
}

/// Полный End прежнего состояния: property end-visual по sufferer, затем
/// удаление записи из арены прежним байт-объёмом.
pub fn end_weak_state<Game: ZonalCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none()
    { return false; }
    let _ = game.update_applied_state_end_visual(
        region_id, holder, key, ZonalCastPropertyTarget::Sufferer,
    );
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    game.remove_applied_state_from(
        region_id, holder, key, (target_region, target), WEAK_STATE_BYTES,
    )
}
