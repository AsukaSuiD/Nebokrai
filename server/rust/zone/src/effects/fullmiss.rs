//! Постоянная прибавка к полному уклонению.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/enlargefullmissstate.cpp` и `enlargefullmissstate.h`.
//! Конструктор задаёт ID `0x25B`; vtable
//! связывает его property callback, writer и reader.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const ENLARGE_FULL_MISS_STATE_ID: u32 = 0x25b;
pub const ENLARGE_FULL_MISS_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnlargeFullMissState {
    gain: i32,
}

impl EnlargeFullMissState {
    pub const fn new(gain: i32) -> Self {
        Self { gain }
    }

    pub const fn skill_id(self) -> u32 {
        ENLARGE_FULL_MISS_STATE_ID
    }

    pub const fn apply(self, value: u16) -> u16 {
        value.wrapping_add(self.gain as u16)
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ENLARGE_FULL_MISS_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(reader.read_i32()?))
    }

    pub fn encoded(self) -> [u8; ENLARGE_FULL_MISS_STATE_BYTES] {
        let mut bytes = [0; ENLARGE_FULL_MISS_STATE_BYTES];
        bytes[..4].copy_from_slice(&ENLARGE_FULL_MISS_STATE_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&self.gain.to_le_bytes());
        bytes
    }
}
