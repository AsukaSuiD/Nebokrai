//! Общая блокировка движения и боя для Blind и состояний рывка;
//! BoaLock использует тот же lifecycle, но запрещает только движение.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/blindstate.cpp
//! и boalockstate.cpp.
//!
//! Объектный Begin требует S; NULL U сохраняет timestamp. Visual Update(0)
//! предшествует move/fight-lock и публикации нового экземпляра в общей арене.
//! Перезапуск обновляет существующий visual и S, не заменяя payload и его срок.
//! End выполняет visual → актуальная S → fight-unlock → move-unlock →
//! RemoveState того же объекта. Отсутствующая S не снимает запреты держателя.
//!
//! Строгий wrapping deadline действует и при нулевом сроке. Восьмибайтный
//! ID/remaining codec читает часы перед remaining при Load и после ID при Save.
//! Общие AI/End обслуживают также KnockOut/SpiderWeb/Seal/Strike/KnightCut/BoaLock. Для primary
//! KnockOut/SpiderWeb/BossBlueQuake/KnightCut/BoaLock payload-адаптер и Seal сохраняют тот же Begin;
//! caller выбирает append либо освобождённый прежний слот без второго хранилища.
//! OnAction не объединён: Blind/KnockOut/Seal/KnightCut заканчиваются при Defense,
//! Rush/Rush2/SpiderWeb/Strike/BoaLock ничего не делают.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::{AppliedState, StateData, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, end_and_destroy_state_at, remove_applied_state_from,
    resolve_applied_state_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
    state_client_record, timed_client_state_time, update_applied_state_end_visual,
    update_applied_state_visual_base, StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const BLIND_STATE_ID: u32 = 0x76;
pub(crate) const BLIND_STATE_BYTES: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BlindState<const ID: u32 = BLIND_STATE_ID> {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl<const ID: u32> BlindState<ID> {
    pub(crate) const fn new(keep_time_ms: u32) -> Self {
        Self { started_at_ms: 0, keep_time_ms }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let started_at_ms = now();
        let keep_time_ms = reader.read_u32()?;
        Ok(Self { started_at_ms, keep_time_ms })
    }

    pub(crate) const fn skill_id(&self) -> u32 { ID }

    pub(crate) const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    pub(crate) fn client_time(&self, now: impl FnMut() -> u32) -> i32 {
        self.client_state_time(now) as i32
    }

    pub(crate) fn encoded(&self, now: impl FnMut() -> u32) -> [u8; BLIND_STATE_BYTES] {
        self.encode_record(|| self.client_state_time(now))
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; BLIND_STATE_BYTES] {
        self.encode_record(|| self.keep_time_ms)
    }

    fn encode_record(&self, remaining: impl FnOnce() -> u32) -> [u8; BLIND_STATE_BYTES] {
        let mut record = [0; BLIND_STATE_BYTES];
        record[..4].copy_from_slice(&ID.to_le_bytes());
        record[4..].copy_from_slice(&remaining().to_le_bytes());
        record
    }
}

pub(crate) trait BlindStatePayload: AppliedState {
    fn blind_state_id(&self) -> u32;
    fn begin_at(&mut self, now_ms: u32);
    fn remaining(&self, now: &mut dyn FnMut() -> u32) -> u32;
    fn install_record(&self) -> [u8; BLIND_STATE_BYTES];
    fn blocks_fighting(&self) -> bool { true }
}

impl<const ID: u32> BlindStatePayload for BlindState<ID>
where BlindState<ID>: AppliedState {
    fn blind_state_id(&self) -> u32 { ID }
    fn begin_at(&mut self, now_ms: u32) { self.started_at_ms = now_ms; }
    fn remaining(&self, now: &mut dyn FnMut() -> u32) -> u32 { self.client_state_time(now) }
    fn install_record(&self) -> [u8; BLIND_STATE_BYTES] { self.encoded_for_install() }
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
pub(crate) fn begin_primary_blind_state<T: BlindStatePayload>(
    game: &mut CGame,
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
pub(crate) fn replace_primary_blind_state<T: BlindStatePayload>(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    state: T, now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return false; };
    let previous = shape.find_state_position(|old| old.state_id() == state.blind_state_id());
    let placement = previous.and_then(|(_, key)| shape.applied_state_replacement_location(key));
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    begin_primary_blind_state_at(
        game, target.0, target.1, Some(source), Some(target), state, placement, now,
    ).is_some()
}

#[allow(clippy::too_many_arguments, reason = "место публикации задаёт конкретный caller после End старого состояния")]
pub(crate) fn begin_primary_blind_state_at<T: BlindStatePayload>(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: T,
    placement: Option<(usize, usize)>,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    resolve_state_move_shape(game, holder_region, holder)?;
    resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
    if user.is_some() { state.begin_at(now()); }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let message = begin_visual_message(sufferer.1, state.blind_state_id(), || state.remaining(&mut *now));
    let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
    let target = resolve_state_move_shape_mut(game, sufferer.0, sufferer.1)?;
    target.set_moveable(false);
    if state.blocks_fighting() { target.set_fightable(false); }
    // После Begin(1)/Update(0) loop1 остаётся незавершённым. Общая арена
    // создаёт этот visual лишь после полного Begin, без повторного пакета.
    let record = state.install_record();
    let shape = resolve_state_move_shape_mut(game, holder_region, holder)?;
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

pub(crate) fn restart_blind_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_data(key)).filter(|state| has_blind_lifecycle(state))
    else { return false; };
    let blocks_fighting = !matches!(state, StateData::BoaLock(_));
    if !begin_base_applied_state(game, region_id, holder, key)
        || !begin_applied_state_visual(game, region_id, holder, key, 1)
    { return false; }
    if let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
        && let Some(state) = resolve_state_move_shape(game, region_id, holder)
            .and_then(|shape| shape.applied_state_data(key))
    {
        let message = begin_visual_message(target.1, state.state_id(), || state_client_record(state, 0, now).time as u32);
        let _ = game.send_move_shape_around(target.0, target.1, &message);
    }
    update_applied_state_visual_base(game, region_id, holder, key);
    // Объектный Begin сохраняет свой параметр S через visual, а не ищет
    // другого участника после доставки. Для restart этим параметром был holder.
    if let Some(target) = resolve_state_move_shape_mut(game, region_id, holder) {
        target.set_moveable(false);
        if blocks_fighting { target.set_fightable(false); }
    }
    true
}

pub(crate) fn update_blind_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = resolve_state_move_shape(game, region_id, holder)
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

pub(crate) fn end_blind_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_data(key)).filter(|state| has_blind_lifecycle(state))
    else { return false; };
    let blocks_fighting = !matches!(state, StateData::BoaLock(_));
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let Some(shape) = resolve_state_move_shape_mut(game, target_region, target) else { return false; };
    if blocks_fighting { shape.set_fightable(true); }
    shape.set_moveable(true);
    remove_applied_state_from(game, region_id, holder, key, (target_region, target), BLIND_STATE_BYTES)
}
