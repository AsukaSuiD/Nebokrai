//! Данные и правила CWeakState; доступ к живому S остаётся в Game-адаптере.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/weakstate.cpp/.h`.
//! Опорные адреса vtable (AI, property callback, writer, reader, getter
//! срока):
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

use super::time::timed_client_state_time;

pub const WEAK_STATE_ID: u32 = 0x12e;
pub const WEAK_STATE_BYTES: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeakState {
    state_type: u32,
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_loss: u32,
    center_x: i32,
    center_y: i32,
    length: i32,
    height: i32,
}

impl WeakState {
    pub const fn new(
        attack_loss: u32,
        center_x: i32,
        center_y: i32,
        length: i32,
        height: i32,
    ) -> Self {
        Self {
            state_type: 2,
            started_at_ms: 0,
            keep_time_ms: 0,
            attack_loss,
            center_x,
            center_y,
            length,
            height,
        }
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != WEAK_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let state_type = reader.read_u32()?;
        let started_at_ms = now();
        let keep_time_ms = reader.read_u32()?;
        Ok(Self {
            state_type,
            started_at_ms,
            keep_time_ms,
            ..Self::new(reader.read_u32()?, 0, 0, 0, 0)
        })
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; WEAK_STATE_BYTES] {
        self.encode_record(|| self.client_time(now) as u32)
    }

    pub fn encoded_for_install(&self) -> [u8; WEAK_STATE_BYTES] {
        self.encode_record(|| self.keep_time_ms)
    }

    fn encode_record(&self, remaining: impl FnOnce() -> u32) -> [u8; WEAK_STATE_BYTES] {
        let mut bytes = [0; WEAK_STATE_BYTES];
        bytes[..4].copy_from_slice(&WEAK_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.state_type.to_le_bytes());
        bytes[8..12].copy_from_slice(&remaining().to_le_bytes());
        bytes[12..].copy_from_slice(&self.attack_loss.to_le_bytes());
        bytes
    }

    pub const fn skill_id(&self) -> u32 {
        WEAK_STATE_ID
    }

    pub const fn state_type(&self) -> u32 {
        self.state_type
    }

    pub fn start_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_time(&self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub const fn contains(&self, tile_x: i32, tile_y: i32) -> bool {
        let start_x = self.center_x.wrapping_sub(self.length / 2);
        let start_y = self.center_y.wrapping_sub(self.height / 2);
        tile_x >= start_x
            && tile_x < start_x.wrapping_add(self.length)
            && tile_y >= start_y
            && tile_y < start_y.wrapping_add(self.height)
    }

    pub fn apply_to_player_attacks(&self, minimum: u32, maximum: u32) -> (u32, u32) {
        let minimum_loss = minimum.min(self.attack_loss) & 0xffff;
        let maximum_loss = maximum.min(self.attack_loss) & 0xffff;
        (
            minimum.wrapping_sub(minimum_loss).min(i32::MAX as u32),
            maximum.wrapping_sub(maximum_loss).min(i32::MAX as u32),
        )
    }

    /// CMonster vtable `0x00652AE4`: `+0x1A8/+0x1AC` прибавляют signed delta
    /// к DWORD-модификаторам без ограничения результата.
    pub const fn apply_to_monster_attacks(&self, minimum: i32, maximum: i32) -> (i32, i32) {
        let loss = self.attack_loss as i32;
        (minimum.wrapping_sub(loss), maximum.wrapping_sub(loss))
    }
}
