//! Данные и срок защитной стойки CPillarState в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/pillarstate.cpp/.h;
//! конструктор 0x005F4A60, Serialize 0x005E7330,
//! Unserialize 0x005D6190, AI 0x005D60B0.

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const PILLAR_STATE_ID: u32 = 0x74;
pub const PILLAR_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PillarState {
    started_at_ms: u32,
    keep_time_ms: u32,
    damage_factor_bits: u32,
}

impl PillarState {
    pub const fn new(keep_time_ms: u32, damage_factor: f32) -> Self {
        Self { started_at_ms: 0, keep_time_ms, damage_factor_bits: damage_factor.to_bits() }
    }

    pub fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != PILLAR_STATE_ID {
            return Err(LegacyReadBlock {
                offset, needed: 4, available: payload.len().saturating_sub(offset),
            });
        }
        // Unserialize берёт часы до срока и коэффициента.
        let started_at_ms = now();
        Ok(Self {
            started_at_ms,
            keep_time_ms: reader.read_u32()?,
            damage_factor_bits: reader.read_u32()?,
        })
    }

    pub fn encoded_for_install(self) -> [u8; PILLAR_STATE_BYTES] {
        self.encoded_with_remaining(|| self.keep_time_ms)
    }

    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; PILLAR_STATE_BYTES] {
        self.encoded_with_remaining(|| self.client_time(now) as u32)
    }

    fn encoded_with_remaining(self, remaining: impl FnOnce() -> u32) -> [u8; PILLAR_STATE_BYTES] {
        let mut bytes = [0; PILLAR_STATE_BYTES];
        bytes[..4].copy_from_slice(&PILLAR_STATE_ID.to_le_bytes());
        // Serialize пишет ID до вызова GetRemainedTime.
        bytes[4..8].copy_from_slice(&remaining().to_le_bytes());
        bytes[8..].copy_from_slice(&self.damage_factor_bits.to_le_bytes());
        bytes
    }

    pub fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn skill_id(self) -> u32 { PILLAR_STATE_ID }

    pub const fn damage_factor(self) -> f32 { f32::from_bits(self.damage_factor_bits) }

    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_time(self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }
}
