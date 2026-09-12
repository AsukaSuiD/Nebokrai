//! CDaubPoisonState (0xDF), gameserver.exe + GameServer.pdb,
//! исходный owner appserver/skills/daubpoisonstate.cpp. Общая арена хранит
//! один экземпляр, стороны Begin, serialized span и visual; заимствования
//! заменяют CState* без копирования payload между End и callback.
//! Ctor0x005F17B0 сохраняет keep без часов. Object Begin0x005F1A50 требует S:
//! base clock при non-NULL U до getters → visual loop1/Update0 → append
//! у caller. NULL-user restart сохраняет timestamp и User. Visual0x005F1B00
//! читает actual S; BFE03 содержит type/id/DF/remaining/0, BFE04 — type/id/DF.
//! End0x005FD420: optional visual1 → свежий S → общий RemoveState(pointer),
//! без записи state.ended. Missing/ended visual подавляет только пакет;
//! существующий ресурс получает base tail и при отсутствующем S.
//! AI0x005D5BA0 читает один clock и сравнивает unsigned start+keep<now,
//! включая keep0, без death/holder-gates. Remaining0x005F2CD0 читает clock
//! повторно только при положительном остатке. Save: ID/remaining, 8 байт;
//! Unserialize0x005EAAC0 читает clock до keep. Список/свойства после удаления
//! обслуживает общий lifecycle; первичный append сам UpdateProperty не вызывает.
//! Первичное Begin(U,U) и append принадлежат захваченному CMoveShape, не
//! только игроку. После часов отдельно сохраняются стороны U/S; свежий S
//! обслуживает visual. Player-only проверка переноса яда принадлежит ударам.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut, timed_client_state_time, update_applied_state_end_visual,
    update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const DAUB_POISON_STATE_ID: u32 = 0xdf;
pub(crate) const DAUB_POISON_STATE_BYTES: usize = 8;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DaubPoisonState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl DaubPoisonState {
    const fn new(keep_time_ms: u32) -> Self {
        Self { started_at_ms: 0, keep_time_ms }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != DAUB_POISON_STATE_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); }
        let mut state = Self::new(reader.read_u32()?);
        state.started_at_ms = now_ms;
        Ok(state)
    }
    pub(crate) fn encoded_for_install(&self) -> [u8; DAUB_POISON_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(&self, now_milliseconds: impl FnMut() -> u32) -> [u8; DAUB_POISON_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(&self, remaining: u32) -> [u8; DAUB_POISON_STATE_BYTES] { let mut bytes = [0; DAUB_POISON_STATE_BYTES]; bytes[..4].copy_from_slice(&DAUB_POISON_STATE_ID.to_le_bytes()); bytes[4..].copy_from_slice(&remaining.to_le_bytes()); bytes }

    pub(crate) const fn skill_id(&self) -> u32 { DAUB_POISON_STATE_ID }

    /// Исходный `GameAiTick::Passed`: равенство с границей ещё активно.
    pub(crate) const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(&self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
}

pub(crate) fn begin_primary_daub_poison_state(
    game: &mut CGame,
    source: (i32, ShapeIdentity),
    keep_time_ms: u32,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let mut state = DaubPoisonState::new(keep_time_ms);
    resolve_state_move_shape(game, source.0, source.1)?;
    state.started_at_ms = now();
    let participant = |source: (i32, ShapeIdentity)| {
        let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
        Some((shape.get_region_id(), ShapeIdentity {
            ex_id: CGuid::GUID_INVALID, ..shape.identity()
        }))
    };
    let user = participant(source)?;
    let sufferer = participant(source)?;
    if let Some(shape) = resolve_state_move_shape(game, sufferer.0, sufferer.1) {
        let target = (shape.shape().get_region_id(), shape.shape().identity());
        let mut message = CMessage::new(STATE_BEGIN_MESSAGE);
        message.add_long(target.1.object_type);
        message.add_long(target.1.id);
        message.add_ulong(state.skill_id());
        message.add_long(state.client_time(now));
        message.add_ulong(0);
        let _ = game.send_move_shape_around(target.0, target.1, &message);
    }
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, source.0, source.1)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(user));
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

pub(crate) fn restart_daub_poison_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<DaubPoisonState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer,
            now, |state, now| state.client_time(now) as u32,
        );
    }
    true
}

pub(crate) fn update_daub_poison_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key))
        .is_some_and(|state| state.expired(now_ms))
    {
        return false;
    }
    end_daub_poison_state(game, region_id, holder, key)
}

pub(crate) fn end_daub_poison_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key)).is_none()
    {
        return false;
    }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, DAUB_POISON_STATE_BYTES)
}
