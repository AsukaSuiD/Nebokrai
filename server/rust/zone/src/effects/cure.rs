//! Данные, срок и сохраняемая запись CCureState в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/curestate.cpp/.h;
//! конструктор 0x005E9E50, Serialize 0x005F51E0,
//! Unserialize 0x005EAAC0, AI 0x005D5BA0.

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const CURE_STATE_SKILL_ID: u32 = 305;
pub const CURE_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CureState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl CureState {
    pub const fn new(keep_time_ms: u32) -> Self {
        Self {
            started_at_ms: 0,
            keep_time_ms,
        }
    }

    pub fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn skill_id(self) -> u32 {
        CURE_STATE_SKILL_ID
    }

    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_state_time(self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != CURE_STATE_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let started_at_ms = now();
        Ok(Self {
            started_at_ms,
            keep_time_ms: reader.read_u32()?,
        })
    }

    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; CURE_STATE_BYTES] {
        self.encoded_with_remaining(|| self.client_state_time(now))
    }

    pub fn encoded_for_install(self) -> [u8; CURE_STATE_BYTES] {
        self.encoded_with_remaining(|| self.keep_time_ms)
    }

    fn encoded_with_remaining(self, remaining: impl FnOnce() -> u32) -> [u8; CURE_STATE_BYTES] {
        let mut bytes = [0; CURE_STATE_BYTES];
        bytes[..4].copy_from_slice(&CURE_STATE_SKILL_ID.to_le_bytes());
        // Serialize записывает ID до вызова GetRemainedTime.
        bytes[4..].copy_from_slice(&remaining().to_le_bytes());
        bytes
    }
}
