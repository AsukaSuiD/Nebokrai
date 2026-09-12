//! CRageBreakState (0x6e), gameserver.exe/GameServer.pdb,
//! appserver/skills/ragebreakstate.cpp.
//!
//! Общие с Fury payload, codec и формула находятся в furystate.rs; запись
//! и визуальный ресурс принадлежат единственному ключу общей арены.
//! Объектный Begin требует S, читает часы только при U и создаёт loop1
//! без Update. End обновляет visual, заново разрешает actual S и удаляет
//! именно этот экземпляр у него, не подставляя держателя вместо цели.
//! Fury вызывает отдельный timer-only Restart: keep/gain/visual неизменны.
//! DB restart вызывает Begin(NULL, holder), сохраняет часы/User и заменяет
//! visual. SetRegion меняет только регион User.
//! ThunderSlash расходует первый ID6E без проверки RTTI/ended: End и затем
//! destructor свежего остатка той же позиции. Расход не добавляет чтения
//! часов или UpdateProperty сверх callbacks самого завершения.

use super::furystate::{
    AttackGainState, FURY_STATE_BYTES, begin_primary_attack_gain_state,
    update_attack_gain_state_properties,
};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state, end_and_destroy_state_at,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    update_applied_state_end_visual,
};
use crate::gameserver::gameserver::game::CGame;

pub(crate) const RAGE_BREAK_STATE_ID: u32 = 0x6e;
pub(crate) const RAGE_BREAK_STATE_BYTES: usize = FURY_STATE_BYTES;
pub(crate) type RageBreakState = AttackGainState<RAGE_BREAK_STATE_ID>;

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub(crate) fn begin_primary_rage_break_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: RageBreakState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    sufferer?;
    begin_primary_attack_gain_state(game, holder_region, holder, user, sufferer, state, now)
}

/// Результат означает наличие первого ID, а не успешное удаление его End.
pub(super) fn consume_rage_break_state(game: &mut CGame, source: (i32, ShapeIdentity)) -> bool {
    let Some((position, _)) = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == RAGE_BREAK_STATE_ID))
    else { return false; };
    let _ = end_and_destroy_state_at(game, source.0, source.1, position);
    true
}

pub(crate) fn end_rage_break_state_key(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(sufferer) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, sufferer, RAGE_BREAK_STATE_BYTES)
}

pub(crate) fn update_rage_break_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
    key: StateKey, now: &mut dyn FnMut() -> u32,
) -> bool {
    update_attack_gain_state_properties::<RAGE_BREAK_STATE_ID>(game, region_id, holder, key, now)
}

pub(crate) fn restart_rage_break_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key)).is_none()
    { return false; }
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_rage_break_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_rage_break_state_key(game, region_id, holder, key)
}
