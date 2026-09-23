//! Данные, срок и запись `CHeartenState` в Zone.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/heartenstate.cpp` и `heartenstate.h`.
//! Конструктор VA `0x005EE500`, vtable `0x006600D4`, writer `0x005D4D10`,
//! reader `0x004F9D80`, property callback `0x005EE740`.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

use super::{maxresource::apply_max_resource_gain, timed_client_state_time};

pub const HEARTEN_STATE_ID: u32 = 0x144;
pub const HEARTEN_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HeartenState {
    started_at_ms: u32,
    keep_time_ms: u32,
    max_hp_gain: i32,
}

impl HeartenState {
    pub const fn new(started_at_ms: u32, keep_time_ms: u32, max_hp_gain: i32) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            max_hp_gain,
        }
    }

    pub const fn skill_id(&self) -> u32 {
        HEARTEN_STATE_ID
    }

    pub fn begin_at(&mut self, started_at_ms: u32) {
        self.started_at_ms = started_at_ms;
    }

    pub const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_time(&self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }

    pub const fn apply(&self, value: u32) -> u32 {
        apply_max_resource_gain(value, self.max_hp_gain)
    }

    pub fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != HEARTEN_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?))
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; HEARTEN_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(HEARTEN_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(HEARTEN_STATE_ID);
        writer.write_u32(self.client_time(now) as u32);
        writer.write_i32(self.max_hp_gain);
        bytes
            .try_into()
            .expect("размер состояния воодушевления фиксирован")
    }

    pub fn encoded_for_install(&self) -> [u8; HEARTEN_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(HEARTEN_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(HEARTEN_STATE_ID);
        writer.write_u32(self.keep_time_ms);
        writer.write_i32(self.max_hp_gain);
        bytes
            .try_into()
            .expect("размер состояния воодушевления фиксирован")
    }
}
