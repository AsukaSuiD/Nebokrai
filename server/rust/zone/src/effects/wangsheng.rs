//! Сохранённые данные состояния CWangshengState в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/wangshengstate.cpp/.h;
//! конструктор 0x00605D90, общий Serialize 0x005E7330,
//! Unserialize 0x005FD660, AI 0x005E6E20, OnUpdateProperties 0x00605FB0.

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const WANGSHENG_STATE_ID: u32 = 0x221;
pub const WANGSHENG_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WangshengState {
    started_at_ms: u32,
    keep_time_ms: u32,
    gain: i32,
}

impl WangshengState {
    pub const fn new(started_at_ms: u32, keep_time_ms: u32, gain: i32) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            gain,
        }
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != WANGSHENG_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        // Общий Unserialize берёт время перед чтением оставшихся DWORD.
        let started_at_ms = now();
        Ok(Self::new(
            started_at_ms,
            reader.read_u32()?,
            reader.read_i32()?,
        ))
    }

    pub const fn state_id(self) -> u32 {
        WANGSHENG_STATE_ID
    }

    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_time(self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    /// Property callback записывает HP только при достижении максимума.
    pub const fn capped_health(self, current: u32, maximum: u32) -> Option<u32> {
        let actual = current.wrapping_add(self.gain as u32);
        if maximum <= actual {
            Some(maximum)
        } else {
            None
        }
    }

    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; WANGSHENG_STATE_BYTES] {
        let mut bytes = [0; WANGSHENG_STATE_BYTES];
        bytes[..4].copy_from_slice(&WANGSHENG_STATE_ID.to_le_bytes());
        // Serialize пишет ID до вызова GetRemainedTime.
        bytes[4..8].copy_from_slice(&self.client_time(now).to_le_bytes());
        bytes[8..].copy_from_slice(&self.gain.to_le_bytes());
        bytes
    }
}
