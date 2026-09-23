//! Состояние сбора душ CSoulCollectState.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! EXE SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E,
//! PDB SHA-256 B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016.
//! Конструктор VA 0x005E1AD0, AddSoul 0x005E1BC0, Serialize 0x005E1D50,
//! Unserialize 0x005E1E80, GetRemainedTime 0x00601200,
//! GetAdditionalData 0x004D7090 (appserver/skills/soulcollectstate.cpp/.h).

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const SOUL_COLLECT_STATE_ID: u32 = 0x13b;
pub const SOUL_COLLECT_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SoulCollectState {
    skill_level: i32,
    variable_percent: u32,
    souls: i32,
}

impl SoulCollectState {
    pub const fn new(skill_level: i32, variable_percent: u32) -> Self {
        Self { skill_level, variable_percent, souls: 0 }
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != SOUL_COLLECT_STATE_ID {
            return Err(LegacyReadBlock {
                offset, needed: 4, available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self { skill_level: reader.read_i32()?, variable_percent: 0,
            souls: reader.read_i32()? })
    }

    pub fn encoded(self) -> [u8; SOUL_COLLECT_STATE_BYTES] {
        let mut bytes = [0; SOUL_COLLECT_STATE_BYTES];
        for (index, value) in [SOUL_COLLECT_STATE_ID as i32, self.skill_level, self.souls]
            .into_iter().enumerate()
        {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub const fn skill_id(self) -> u32 { SOUL_COLLECT_STATE_ID }
    pub const fn variable_percent(self) -> u32 { self.variable_percent }
    pub const fn souls(self) -> i32 { self.souls }
    pub const fn client_fields(self) -> (u32, u32) { (0, self.souls as u32) }

    /// Только после проверки живого Sufferer адаптер вызывает этот переход.
    pub fn increment(&mut self) -> bool {
        if self.souls >= self.skill_level { return false; }
        self.souls = self.souls.wrapping_add(1);
        true
    }
}
