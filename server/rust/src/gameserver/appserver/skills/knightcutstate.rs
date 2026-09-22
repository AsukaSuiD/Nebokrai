//! Оглушение рыцарского удара CKnightCutState (0x67).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/knightcutstate.cpp.
//!
//! Объектный Begin, AI, End, региональная смена и visual используют общий
//! blindstate: S обязательна, NULL U сохраняет timestamp. Visual предшествует
//! запретам движения и боя, публикация занимает выбранный caller-ом слот.
//! End снимает запреты живой S и удаляет тот же экземпляр, а не первый ID.
//! Защитное действие завершает состояние; Cure использует общий direct End.
//! Duration-конструктор оставляет timestamp нулевым. DB хранит ID/remaining
//! в восьми байтах; Load читает отдельные часы до оставшегося срока.
//! Вход в регион восстанавливает visual и запреты без нового отсчёта.

use super::blindstate::BlindStatePayload;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::states::state::timed_client_state_time;

pub(crate) const KNIGHT_CUT_STATE_ID: u32 = 0x67;
pub(crate) const KNIGHT_CUT_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KnightCutState { started_at_ms: u32, keep_time_ms: u32 }

impl KnightCutState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != KNIGHT_CUT_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }

    pub(crate) fn encoded_for_install(self) -> [u8; KNIGHT_CUT_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    pub(crate) fn encoded(self, now: impl FnMut() -> u32) -> [u8; KNIGHT_CUT_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now) as u32)
    }

    fn encoded_with_remaining(self, remaining: u32) -> [u8; KNIGHT_CUT_STATE_BYTES] {
        let mut bytes = [0; KNIGHT_CUT_STATE_BYTES];
        bytes[..4].copy_from_slice(&KNIGHT_CUT_STATE_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&remaining.to_le_bytes());
        bytes
    }

    pub(crate) const fn skill_id(self) -> u32 { KNIGHT_CUT_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }
    pub(crate) fn client_time(self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }
}

impl BlindStatePayload for KnightCutState {
    fn blind_state_id(&self) -> u32 { KNIGHT_CUT_STATE_ID }
    fn begin_at(&mut self, now_ms: u32) { self.started_at_ms = now_ms; }
    fn remaining(&self, now: &mut dyn FnMut() -> u32) -> u32 { self.client_time(now) as u32 }
    fn install_record(&self) -> [u8; KNIGHT_CUT_STATE_BYTES] { self.encoded_for_install() }
}
