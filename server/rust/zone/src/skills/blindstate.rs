//! Общая блокировка движения и боя для Blind и состояний рывка в Zone;
//! BoaLock использует тот же lifecycle, но запрещает только движение.
//! Данные и 8-байтная запись семейства — `effects/blind.rs` (там же адреса
//! конструкторов и общего кодека).
//!
//! Quirks: строгий wrapping deadline действует и при нулевом сроке; End не
//! снимает запреты держателя при отсутствующей S; общие AI/End обслуживают
//! также KnockOut/SpiderWeb/Seal/Strike/KnightCut/BoaLock; OnAction не
//! объединён — Blind/KnockOut/Seal/KnightCut заканчиваются при Defense,
//! остальные ничего не делают.
//!
//! Швы: hub `selfcast::SelfCastGame`; обвязка арены — прежний
//! `states/state.rs` (одноимённые методы трейта).
//!
//! Исходные владельцы PDB: `appserver/skills/{blindstate,boalockstate}.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#blindstate--8-байтный-lifecycle-blindlock-семейства

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;
use crate::effects::{BLIND_STATE_BYTES, BlindState};
use crate::regions::ShapeIdentity;

use super::selfcast::{SelfCastGame, SelfCastMoveShape, SelfCastPropertyTarget};
use super::state::{AppliedState, StateData, StateKey, state_client_record};

pub trait BlindStatePayload: AppliedState {
    fn blind_state_id(&self) -> u32;
    fn begin_at(&mut self, now_ms: u32);
    fn remaining(&self, now: &mut dyn FnMut() -> u32) -> u32;
    fn install_record(&self) -> [u8; BLIND_STATE_BYTES];
    fn blocks_fighting(&self) -> bool { true }
}

impl<const ID: u32, const BLOCKS_FIGHTING: bool> BlindStatePayload for BlindState<ID, BLOCKS_FIGHTING>
where BlindState<ID, BLOCKS_FIGHTING>: AppliedState {
    fn blind_state_id(&self) -> u32 { ID }
    fn begin_at(&mut self, now_ms: u32) { BlindState::begin_at(self, now_ms); }
    fn remaining(&self, now: &mut dyn FnMut() -> u32) -> u32 { self.client_state_time(now) }
    fn install_record(&self) -> [u8; BLIND_STATE_BYTES] { self.encoded_for_install() }
    fn blocks_fighting(&self) -> bool { BLOCKS_FIGHTING }
}

fn has_blind_lifecycle(state: &StateData) -> bool {
    state.is_blind() || matches!(state, StateData::Rush(_) | StateData::Rush2(_) | StateData::BoaLock(_))
}

fn begin_visual_message(
    target: ShapeIdentity,
    state_id: u32,
    remaining: impl FnOnce() -> u32,
) -> CMessage {
    let mut message = CMessage::new(0x000b_fe03);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_ulong(state_id);
    message.add_ulong(remaining());
    message.add_ulong(0);
    message
}

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub fn begin_primary_blind_state<Game: SelfCastGame, T: BlindStatePayload>(
    game: &mut Game,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: T,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    begin_primary_blind_state_at(game, holder_region, holder, user, sufferer, state, None, now)
}

/// KnockOut, Mosou, KnightCut и Seal создают новый payload до поиска прежнего.
/// Полный End и destructor освобождают ту же позицию до нового Begin.
pub fn replace_primary_blind_state<Game: SelfCastGame, T: BlindStatePayload>(
    game: &mut Game, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    state: T, now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(shape) = game.resolve_state_move_shape(target.0, target.1) else { return false; };
    let previous = shape.find_state_position(|old| old.state_id() == state.blind_state_id());
    let placement = previous.and_then(|(_, key)| shape.applied_state_replacement_location(key));
    if let Some((position, _)) = previous {
        let _ = game.end_and_destroy_state_at(target.0, target.1, position);
    }
    begin_primary_blind_state_at(
        game, target.0, target.1, Some(source), Some(target), state, placement, now,
    ).is_some()
}

#[allow(clippy::too_many_arguments, reason = "место публикации задаёт конкретный caller после End старого состояния")]
pub fn begin_primary_blind_state_at<Game: SelfCastGame, T: BlindStatePayload>(
    game: &mut Game,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: T,
    placement: Option<(usize, usize)>,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    game.resolve_state_move_shape(holder_region, holder)?;
    game.resolve_state_move_shape(sufferer.0, sufferer.1)?;
    if user.is_some() { state.begin_at(now()); }
    let participant = |(region, identity)| {
        let shape = game.resolve_state_move_shape(region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let message = begin_visual_message(sufferer.1, state.blind_state_id(), || state.remaining(&mut *now));
    game.send_move_shape_around(sufferer.0, sufferer.1, &message);
    let target = game.resolve_state_move_shape_mut(sufferer.0, sufferer.1)?;
    target.set_moveable(false);
    if state.blocks_fighting() { target.set_fightable(false); }
    // После Begin(1)/Update(0) loop1 остаётся незавершённым. Общая арена
    // создаёт этот visual лишь после полного Begin, без повторного пакета.
    let record = state.install_record();
    let shape = game.resolve_state_move_shape_mut(holder_region, holder)?;
    let key = match placement {
        Some(location) => shape.insert_replacement_state_record(state, &record, location)?,
        None => shape.append_applied_state_record(state, &record),
    };
    shape.begin_applied_state_visual(key, 1);
    shape.update_applied_state_visual_base(key);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

pub fn restart_blind_state<Game: SelfCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state_data(key)).filter(|state| has_blind_lifecycle(state))
    else { return false; };
    let blocks_fighting = !matches!(state, StateData::BoaLock(_));
    if !game.begin_base_applied_state(region_id, holder, key)
        || !game.begin_applied_state_visual(region_id, holder, key, 1)
    { return false; }
    if let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key)
        && let Some(state) = game.resolve_state_move_shape(region_id, holder)
            .and_then(|shape| shape.applied_state_data(key))
    {
        let message = begin_visual_message(target.1, state.state_id(), || state_client_record(state, 0, now).time as u32);
        game.send_move_shape_around(target.0, target.1, &message);
    }
    game.update_applied_state_visual_base(region_id, holder, key);
    // Объектный Begin сохраняет свой параметр S через visual, а не ищет
    // другого участника после доставки. Для restart этим параметром был holder.
    if let Some(target) = game.resolve_state_move_shape_mut(region_id, holder) {
        target.set_moveable(false);
        if blocks_fighting { target.set_fightable(false); }
    }
    true
}

pub fn update_blind_state<Game: SelfCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state_data(key))
        .is_some_and(|state| match state {
            StateData::Blind(state) => state.expired(now_ms),
            StateData::Rush(state) => state.expired(now_ms),
            StateData::Rush2(state) => state.expired(now_ms),
            StateData::KnockOut(state) => state.expired(now_ms),
            StateData::BoaLock(state) => state.expired(now_ms),
            StateData::KnightCut(state) => state.expired(now_ms),
            StateData::SpiderWeb(state) => state.expired(now_ms),
            StateData::Seal(state) => state.expired(now_ms),
            StateData::Strike(state) => state.expired(now_ms),
            _ => false,
        });
    expired && end_blind_state(game, region_id, holder, key)
}

pub fn end_blind_state<Game: SelfCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state_data(key)).filter(|state| has_blind_lifecycle(state))
    else { return false; };
    let blocks_fighting = !matches!(state, StateData::BoaLock(_));
    game.update_applied_state_end_visual(region_id, holder, key, SelfCastPropertyTarget::Sufferer);
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    let Some(shape) = game.resolve_state_move_shape_mut(target_region, target) else { return false; };
    if blocks_fighting { shape.set_fightable(true); }
    shape.set_moveable(true);
    game.remove_applied_state_from(region_id, holder, key, (target_region, target), BLIND_STATE_BYTES)
}
