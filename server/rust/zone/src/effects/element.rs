//! Сохранённые прибавки TaiJi и Origin к свойствам стихии.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/taijistate.cpp/.h` и `originstate.cpp/.h`.
//! Конструкторы задают ID `0x12D/0x130`;
//! обе vtable используют общие writer и reader, но разные property callbacks.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const TAIJI_STATE_ID: u32 = 0x12d;
pub const ORIGIN_STATE_ID: u32 = 0x130;
pub const ELEMENT_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ElementState<const ID: u32> {
    gain: i32,
}

pub type TaiJiState = ElementState<TAIJI_STATE_ID>;
pub type OriginState = ElementState<ORIGIN_STATE_ID>;

impl<const ID: u32> ElementState<ID> {
    pub const fn new(gain: i32) -> Self {
        Self { gain }
    }

    pub const fn skill_id(self) -> u32 {
        ID
    }

    pub const fn monster_gain(self) -> i32 {
        self.gain
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(reader.read_i32()?))
    }

    pub fn encoded(self) -> [u8; ELEMENT_STATE_BYTES] {
        let mut bytes = [0; ELEMENT_STATE_BYTES];
        bytes[..4].copy_from_slice(&ID.to_le_bytes());
        bytes[4..].copy_from_slice(&self.gain.to_le_bytes());
        bytes
    }
}

impl ElementState<TAIJI_STATE_ID> {
    pub const fn apply_player_resistance(self, value: u32) -> u32 {
        let sum = value.wrapping_add((self.gain as u16) as u32);
        if sum > i32::MAX as u32 {
            i32::MAX as u32
        } else {
            sum
        }
    }
}

impl ElementState<ORIGIN_STATE_ID> {
    pub const fn apply_player_modify(self, value: i32) -> i32 {
        value.wrapping_add((self.gain as i16) as i32)
    }
}
