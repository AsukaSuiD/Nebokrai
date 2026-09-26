//! Живой Begin, End и пересчёт CFuryState и CRageBreakState.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/furystate.cpp/.h
//! и ragebreakstate.cpp/.h. Данные и числовые правила — в Zone effects;
//! общие живые callbacks AttackGain-семьи (ICF-пара Serialize/
//! OnUpdateProperties/Unserialize/AI) перенесены буквально в
//! `nebokrai_zone::skills::ragebreakstate` порцией T4; здесь — обёртки
//! Fury-ветки и делегации с прежними сигнатурами.

use crate::gameserver::appserver::moveshape::{AppliedState, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, end_base_applied_state, resolve_state_move_shape,
    update_applied_state_end_visual,
};
use crate::gameserver::gameserver::game::CGame;
pub(crate) use nebokrai_zone::effects::{
    ATTACK_GAIN_STATE_BYTES as FURY_STATE_BYTES, AttackGainState, FURY_STATE_SKILL_ID, FuryState,
};

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub(crate) fn begin_primary_fury_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: FuryState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    user?;
    begin_primary_attack_gain_state(game, holder_region, holder, user, sufferer, state, now)
}

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub(super) fn begin_primary_attack_gain_state<const ID: u32>(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: AttackGainState<ID>,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey>
where
    AttackGainState<ID>: AppliedState,
{
    nebokrai_zone::skills::ragebreakstate::begin_primary_attack_gain_state(
        game, holder_region, holder, user, sufferer, state, now,
    )
}

pub(crate) fn update_fury_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
    key: StateKey, now: &mut dyn FnMut() -> u32,
) -> bool {
    update_attack_gain_state_properties::<FURY_STATE_SKILL_ID>(game, region_id, holder, key, now)
}

pub(super) fn update_attack_gain_state_properties<const ID: u32>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool
where
    AttackGainState<ID>: AppliedState,
{
    nebokrai_zone::skills::ragebreakstate::update_attack_gain_state_properties::<CGame, ID>(
        game, region_id, holder, key, now,
    )
}

pub(crate) fn restart_fury_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    false
}

pub(crate) fn update_fury_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<FuryState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_fury_state(game, region_id, holder, key)
}

pub(crate) fn end_fury_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<FuryState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    end_base_applied_state(game, region_id, holder, key, FURY_STATE_BYTES)
}
