//! Каноническое состояние смазки оружия ядом `CDaubPoisonState` (`0xDF`).
//! Истечение получает ключ конкретного экземпляра общей арены; проверка
//! срока и End не подменяют его первым состоянием с тем же ID.
//! Vtable 0x0066047c, слот +0x0c: CBlindState::AI (0x005d5ba0).
//! Истечение использует строгий абсолютный wrapping deadline, включая ноль.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/daubpoisonstate.cpp`. Достигнутый cast создаёт состояние
//! игроку; само состояние хранит строгий wrapping-срок и публикует пакеты начала и
//! завершения. Проверки стрел читают этот единственный типизированный
//! экземпляр через `GetStateBySkillID`. Persisted-запись `ID + remaining time`
//! занимает 8 байт и активируется StartAllStates после 8F801. Vtable exact EXE
//! подтверждает общий с `CBlindState` `GetRemainedTime` по адресу
//! `0x005F2CD0`, включая отдельное чтение часов для положительного остатка.
//! End vtable+0x1C→0x005FD420 не имеет Player-gate: сначала visual, затем
//! GetSufferer и RemoveState. Общий AI поэтому завершает и регионального
//! держателя; фактическое удаление вызывает общий virtual UpdateProperty.
//! Object Begin +0x08→0x005F1A50: guard null sufferer, base Begin,
//! новый visual(0xC), BeginVisualEffect(1), Update(state,0), затем return 1.
//! При Begin(NULL,holder) clock не читается и срок не перезапускается:
//! timestamp уже записан унаследованной Unserialize0x005EAAC0. Визуальный
//! GetRemainedTime читает собственные часы после создания ресурса.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, resolve_state_move_shape,
    resolve_state_move_shape_mut, timed_client_state_time, update_applied_state_visual_base,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const DAUB_POISON_STATE_ID: u32 = 0xdf;
pub(crate) const DAUB_POISON_STATE_BYTES: usize = 8;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DaubPoisonState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl DaubPoisonState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != DAUB_POISON_STATE_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }
    pub(crate) fn encoded_for_install(self) -> [u8; DAUB_POISON_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; DAUB_POISON_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; DAUB_POISON_STATE_BYTES] { let mut bytes = [0; DAUB_POISON_STATE_BYTES]; bytes[..4].copy_from_slice(&DAUB_POISON_STATE_ID.to_le_bytes()); bytes[4..].copy_from_slice(&remaining.to_le_bytes()); bytes }

    pub(crate) const fn skill_id(self) -> u32 { DAUB_POISON_STATE_ID }

    /// Исходный `GameAiTick::Passed`: равенство с границей ещё активно.
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
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
    if !begin_base_applied_state(game, region_id, holder, key)
        || !begin_applied_state_visual(game, region_id, holder, key, 1)
    {
        return false;
    }
    let Some(remaining) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key))
        .map(|state| state.client_time(now))
    else { return false };
    let mut message = CMessage::new(STATE_BEGIN_MESSAGE);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(DAUB_POISON_STATE_ID as i32);
    message.add_long(remaining);
    message.add_long(0);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let _ = update_applied_state_visual_base(game, region_id, holder, key);
    true
}

pub(crate) fn send_daub_poison_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: DaubPoisonState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let Some(player) = game.find_player(player_id) else { return };
    let identity = player.shape().identity();
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn replace_player_daub_poison_state(
    game: &mut CGame,
    player_id: i32,
    state: DaubPoisonState,
    mut now_milliseconds: impl FnMut() -> u32,
) -> bool {
    let previous = game
        .find_player_mut(player_id)
        .map(|player| player.replace_daub_poison_state(state));
    let Some(previous) = previous else { return false };
    if let Some(previous) = previous {
        send_daub_poison_state_visual(game, player_id, previous, false, || now_milliseconds());
    }
    send_daub_poison_state_visual(game, player_id, state, true, now_milliseconds);
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
    let mut message = CMessage::new(STATE_END_MESSAGE);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(DAUB_POISON_STATE_ID as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<DaubPoisonState>(key, DAUB_POISON_STATE_BYTES))
        .is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}
