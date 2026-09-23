//! Данные и числовые правила CGodBlessState/CGodBlessState2 в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/godblessstate{,2}.cpp/.h;
//! Serialize 0x005EE310, Unserialize 0x00601830,
//! GetRemainedTime 0x00601480, AI 0x00601640, OnUpdateProperties 0x00601690.

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const GOD_BLESS_STATE_ID: u32 = 0x12f;
pub const GOD_BLESS_STATE_2_ID: u32 = 0x145;
pub const GOD_BLESS_STATE_BYTES: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GodBlessState {
    skill_id: u32,
    started_at_ms: u32,
    keep_time_ms: u32,
    minimum_attack_gain: u32,
    maximum_attack_gain: u32,
    element_gain: u32,
}

impl GodBlessState {
    pub const fn new(
        skill_id: u32, keep_time_ms: u32, minimum_attack_gain: u32,
        maximum_attack_gain: u32, element_gain: u32,
    ) -> Self {
        debug_assert!(matches!(skill_id, GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID));
        Self { skill_id, started_at_ms: 0, keep_time_ms, minimum_attack_gain,
            maximum_attack_gain, element_gain }
    }

    pub fn begin_for_install(
        &mut self, user_exists: bool, sufferer_exists: bool,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        let can_begin = if self.skill_id == GOD_BLESS_STATE_ID { sufferer_exists } else { user_exists };
        if !can_begin { return false; }
        if user_exists { self.started_at_ms = now(); }
        true
    }

    pub fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        if !matches!(skill_id, GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID) {
            return Err(LegacyReadBlock {
                offset, needed: 4, available: payload.len().saturating_sub(offset),
            });
        }
        // Unserialize читает часы до сохранённого срока и трёх прибавок.
        let started_at_ms = now();
        Ok(Self {
            skill_id, started_at_ms,
            keep_time_ms: reader.read_u32()?,
            minimum_attack_gain: reader.read_u32()?,
            maximum_attack_gain: reader.read_u32()?,
            element_gain: reader.read_u32()?,
        })
    }

    pub const fn skill_id(self) -> u32 { self.skill_id }
    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }
    pub fn client_time(self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }
    pub fn encoded_for_install(self) -> [u8; GOD_BLESS_STATE_BYTES] {
        self.encoded_with_remaining(|| self.keep_time_ms)
    }
    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; GOD_BLESS_STATE_BYTES] {
        self.encoded_with_remaining(|| self.client_time(now) as u32)
    }
    fn encoded_with_remaining(self, remaining: impl FnOnce() -> u32) -> [u8; GOD_BLESS_STATE_BYTES] {
        let mut bytes = [0; GOD_BLESS_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes());
        // Serialize пишет ID перед вызовом GetRemainedTime.
        bytes[4..8].copy_from_slice(&remaining().to_le_bytes());
        bytes[8..12].copy_from_slice(&self.minimum_attack_gain.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.maximum_attack_gain.to_le_bytes());
        bytes[16..].copy_from_slice(&self.element_gain.to_le_bytes());
        bytes
    }

    pub fn player_gains(self, minimum: u32, maximum: u32, element: i32) -> (u32, u32, i32) {
        (
            minimum.wrapping_add(self.minimum_attack_gain as u16 as u32).min(i32::MAX as u32),
            maximum.wrapping_add(self.maximum_attack_gain as u16 as u32).min(i32::MAX as u32),
            element.wrapping_add(self.element_gain as u16 as i32),
        )
    }

    pub fn monster_gains(self) -> (i32, i32, i32) {
        (self.minimum_attack_gain as i32, self.maximum_attack_gain as i32, self.element_gain as i32)
    }
}
