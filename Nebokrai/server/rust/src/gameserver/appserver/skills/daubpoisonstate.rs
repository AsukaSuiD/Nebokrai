//! Каноническое состояние смазки оружия ядом `CDaubPoisonState` (`0xDF`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/daubpoisonstate.cpp`. Состояние принадлежит только
//! игроку, хранит строгий wrapping-срок и публикует исходные пакеты начала и
//! завершения. Проверки стрел читают этот единственный типизированный
//! экземпляр через `GetStateBySkillID`. Persisted-запись `ID + remaining time`
//! занимает 8 байт и активируется при spatial login. Vtable exact EXE
//! подтверждает общий с `CBlindState` `GetRemainedTime` по адресу
//! `0x005F2CD0`, включая отдельное чтение часов для положительного остатка.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::states::state::timed_client_state_time;
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

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != DAUB_POISON_STATE_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); }
        Ok(Self::new(0, reader.read_u32()?))
    }
    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self { self.started_at_ms = now_ms; self }
    pub(crate) fn encoded_for_install(self) -> [u8; DAUB_POISON_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; DAUB_POISON_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; DAUB_POISON_STATE_BYTES] { let mut bytes = [0; DAUB_POISON_STATE_BYTES]; bytes[..4].copy_from_slice(&DAUB_POISON_STATE_ID.to_le_bytes()); bytes[4..].copy_from_slice(&remaining.to_le_bytes()); bytes }

    pub(crate) const fn skill_id(self) -> u32 { DAUB_POISON_STATE_ID }

    /// Исходный `GameAiTick::Passed`: равенство с границей ещё активно.
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
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

pub(crate) fn expire_player_daub_poison_state(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    let state = game
        .find_player_mut(player_id)
        .and_then(|player| player.take_expired_daub_poison_state(now_ms));
    let Some(state) = state else { return false };
    send_daub_poison_state_visual(game, player_id, state, false, || now_ms);
    true
}
