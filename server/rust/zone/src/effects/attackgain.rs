//! Данные усиления атаки Fury и RageBreak в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/furystate.cpp/.h
//! и ragebreakstate.cpp/.h; общий Serialize 0x005E7330,
//! OnUpdateProperties 0x005FD480, Unserialize 0x005FD660, AI 0x005EA4C0.

use super::time::timed_client_state_time;
use crate::combat::truncate_original_i64_low;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const FURY_STATE_SKILL_ID: u32 = 0x1a3;
pub const RAGE_BREAK_STATE_ID: u32 = 0x6e;
pub const ATTACK_GAIN_STATE_BYTES: usize = 12;
pub type FuryState = AttackGainState<FURY_STATE_SKILL_ID>;
pub type RageBreakState = AttackGainState<RAGE_BREAK_STATE_ID>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttackGainState<const ID: u32> {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_gain_percent: i32,
}

impl<const ID: u32> AttackGainState<ID> {
    pub const fn new(keep_time_ms: u32, attack_gain_percent: i32) -> Self {
        Self { started_at_ms: 0, keep_time_ms, attack_gain_percent }
    }

    pub const fn skill_id(&self) -> u32 { ID }

    pub fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ID {
            return Err(LegacyReadBlock {
                offset, needed: 4, available: payload.len().saturating_sub(offset),
            });
        }
        // Unserialize читает время перед двумя оставшимися DWORD.
        let started_at_ms = now();
        Ok(Self {
            started_at_ms,
            keep_time_ms: reader.read_u32()?,
            attack_gain_percent: reader.read_i32()?,
        })
    }

    pub fn encoded_for_install(&self) -> [u8; ATTACK_GAIN_STATE_BYTES] {
        self.encoded_with_remaining(|| self.keep_time_ms)
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; ATTACK_GAIN_STATE_BYTES] {
        self.encoded_with_remaining(|| self.client_time(now) as u32)
    }

    fn encoded_with_remaining(&self, remaining: impl FnOnce() -> u32) -> [u8; ATTACK_GAIN_STATE_BYTES] {
        let mut bytes = [0; ATTACK_GAIN_STATE_BYTES];
        bytes[..4].copy_from_slice(&ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining().to_le_bytes());
        bytes[8..].copy_from_slice(&self.attack_gain_percent.to_le_bytes());
        bytes
    }

    pub const fn restart_timer(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_time(&self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }

    pub fn truncated_gain(&self, maximum: u32) -> i32 {
        truncate_original_i64_low(
            f64::from(self.attack_gain_percent)
                * f64::from(0.01_f32)
                * f64::from(maximum),
        )
    }

    pub fn apply_to_player_maximum_attack(&self, maximum: u32) -> u32 {
        let mut gain = self.truncated_gain(maximum) as u16 as u32;
        if maximum.wrapping_add(gain) > u16::MAX as u32 {
            gain = (u16::MAX as u32).wrapping_sub(maximum);
        }
        maximum.wrapping_add(gain as u16 as u32).min(i32::MAX as u32)
    }
}
