//! Сохранённые прибавки к максимумам HP и MP.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/enlargemaxhpstate.cpp` и `enlargemaxhpstate.h`,
//! `enlargemaxmpstate.cpp` и `enlargemaxmpstate.h`.
//! Конструкторы VA `0x005E22E0/0x005E2160` задают ID `0x259/0x25A`;
//! обе vtable ведут к writer `0x005E23D0` и reader `0x00601350`.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const ENLARGE_MAX_HP_STATE_ID: u32 = 0x259;
pub const ENLARGE_MAX_MP_STATE_ID: u32 = 0x25a;
pub const MAX_RESOURCE_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MaxResourceState<const ID: u32> {
    gain: i32,
}

pub type EnlargeMaxHpState = MaxResourceState<ENLARGE_MAX_HP_STATE_ID>;
pub type EnlargeMaxMpState = MaxResourceState<ENLARGE_MAX_MP_STATE_ID>;

impl<const ID: u32> MaxResourceState<ID> {
    pub const fn new(gain: i32) -> Self {
        Self { gain }
    }

    pub const fn skill_id(self) -> u32 {
        ID
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

    pub fn encoded(self) -> [u8; MAX_RESOURCE_STATE_BYTES] {
        let mut bytes = [0; MAX_RESOURCE_STATE_BYTES];
        bytes[..4].copy_from_slice(&ID.to_le_bytes());
        bytes[4..].copy_from_slice(&self.gain.to_le_bytes());
        bytes
    }

    /// Оригинальные callbacks HP и MP используют одинаковое сложение DWORD.
    pub const fn apply(self, value: u32) -> u32 {
        let result = value.wrapping_add(self.gain as u32);
        if result > i32::MAX as u32 {
            i32::MAX as u32
        } else {
            result
        }
    }
}
