//! Данные и числовое подавление атаки CRoarState в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/roarstate.cpp/.h;
//! конструктор 0x005EC7E0, Serialize 0x005F65F0, Unserialize 0x005ECC60,
//! AI 0x005EC9A0, OnUpdateProperties 0x005ECAA0.

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const ROAR_STATE_ID: u32 = 0x83;
pub const ROAR_STATE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoarState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_loss: i32,
    element_attack_loss: i32,
}

impl RoarState {
    pub const fn new(keep_time_ms: u32, attack_loss: i32, element_attack_loss: i32) -> Self {
        Self {
            started_at_ms: 0,
            keep_time_ms,
            attack_loss,
            element_attack_loss,
        }
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ROAR_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let keep_time_ms = reader.read_u32()?;
        let attack_loss = reader.read_i32()?;
        let element_attack_loss = reader.read_i32()?;
        // Unserialize берёт часы после всех трёх сохранённых полей.
        Ok(Self {
            started_at_ms: now(),
            keep_time_ms,
            attack_loss,
            element_attack_loss,
        })
    }

    pub fn encoded_for_install(self) -> [u8; ROAR_STATE_BYTES] {
        self.encoded_with_remaining(|| self.keep_time_ms)
    }

    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; ROAR_STATE_BYTES] {
        self.encoded_with_remaining(|| self.client_time(now) as u32)
    }

    fn encoded_with_remaining(self, remaining: impl FnOnce() -> u32) -> [u8; ROAR_STATE_BYTES] {
        let mut bytes = [0; ROAR_STATE_BYTES];
        bytes[..4].copy_from_slice(&ROAR_STATE_ID.to_le_bytes());
        // Serialize записывает ID перед запросом GetRemainedTime.
        bytes[4..8].copy_from_slice(&remaining().to_le_bytes());
        bytes[8..12].copy_from_slice(&self.attack_loss.to_le_bytes());
        bytes[12..].copy_from_slice(&self.element_attack_loss.to_le_bytes());
        bytes
    }

    pub fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn skill_id(self) -> u32 {
        ROAR_STATE_ID
    }

    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_time(self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }

    pub fn player_attack(self, minimum: u32, maximum: u32, element_modify: i32) -> (u32, u32, i32) {
        let attack_loss = self.attack_loss as u32;
        let minimum_loss = minimum.min(attack_loss);
        let maximum_loss = maximum.min(attack_loss);
        let element_loss = element_modify.min(self.element_attack_loss);
        (
            minimum.wrapping_sub(minimum_loss).min(i32::MAX as u32),
            maximum.wrapping_sub(maximum_loss).min(i32::MAX as u32),
            element_modify.wrapping_sub(element_loss),
        )
    }

    pub fn monster_losses(
        self,
        minimum: u32,
        maximum: u32,
        element_modify: u32,
    ) -> (i32, i32, i32) {
        (
            minimum.min(self.attack_loss as u32) as i32,
            maximum.min(self.attack_loss as u32) as i32,
            element_modify.min(self.element_attack_loss as u32) as i32,
        )
    }
}
