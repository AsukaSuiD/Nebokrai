//! Состояния CCallosityState/CCallosityState2 с общей записью и формулой.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/callositystate.cpp/.h` и `callositystate2.cpp/.h`.
//! Конструкторы задают ID `0x75/0x7D`;
//! обе vtable используют общие writer, reader, AI и property callback.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

use super::time::timed_client_state_time;

pub const CALLOSITY_SKILL_ID: u32 = 0x75;
pub const CALLOSITY_2_SKILL_ID: u32 = 0x7d;
pub const CALLOSITY_STATE_BYTES: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallosityFamilyState {
    skill_id: u32,
    blast_factor: u16,
    started_at_ms: u32,
    time_to_keep: i32,
}

impl CallosityFamilyState {
    pub const fn new(
        skill_id: u32,
        blast_factor: u16,
        started_at_ms: u32,
        time_to_keep: i32,
    ) -> Self {
        Self {
            skill_id,
            blast_factor,
            started_at_ms,
            time_to_keep,
        }
    }

    pub const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub fn start_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.time_to_keep as u32) < now_ms
    }

    pub fn client_state_time(self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.time_to_keep as u32, now) as i32
    }

    pub fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        if !matches!(skill_id, CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID) {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let time_to_keep = reader.read_i32()?;
        let blast_factor = reader.read_u16()?;
        Ok(Self::new(skill_id, blast_factor, now_ms, time_to_keep))
    }

    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; CALLOSITY_STATE_BYTES] {
        let mut bytes = [0; CALLOSITY_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.client_state_time(now).to_le_bytes());
        bytes[8..].copy_from_slice(&self.blast_factor.to_le_bytes());
        bytes
    }

    pub fn encoded_for_install(self) -> [u8; CALLOSITY_STATE_BYTES] {
        let mut bytes = [0; CALLOSITY_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.time_to_keep.to_le_bytes());
        bytes[8..].copy_from_slice(&self.blast_factor.to_le_bytes());
        bytes
    }

    pub const fn apply_to_blast_attack(self, blast_attack: u16) -> u16 {
        blast_attack.wrapping_add(self.blast_factor)
    }
}
