//! Постоянные прибавки минимальной и максимальной атаки Swordship.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/swordshipstate{,2,3,4}.cpp/.h`.
//! Vtable `0x00660D4C/0x006602AC/0x0065FEB4/0x0065FE64` направляют
//! property callback на `0x005F8870`, writer на `0x005ECE70`, reader
//! на `0x005F0010`. Четыре конструктора задают ID ниже.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const SWORDSHIP_STATE_ID: u32 = 0x6f;
pub const SWORDSHIP_2_STATE_ID: u32 = 0xe0;
pub const SWORDSHIP_3_STATE_ID: u32 = 0xe8;
pub const SWORDSHIP_4_STATE_ID: u32 = 0xe9;
pub const SWORDSHIP_STATE_BYTES: usize = 12;

pub const fn is_swordship_state_id(skill_id: u32) -> bool {
    matches!(
        skill_id,
        SWORDSHIP_STATE_ID | SWORDSHIP_2_STATE_ID | SWORDSHIP_3_STATE_ID | SWORDSHIP_4_STATE_ID
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SwordshipState {
    skill_id: u32,
    minimum_attack_gain: i32,
    maximum_attack_gain: i32,
}

impl SwordshipState {
    pub const fn new(skill_id: u32, minimum_attack_gain: i32, maximum_attack_gain: i32) -> Self {
        debug_assert!(is_swordship_state_id(skill_id));
        Self {
            skill_id,
            minimum_attack_gain,
            maximum_attack_gain,
        }
    }

    pub const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub const fn minimum_attack_gain(self) -> i32 {
        self.minimum_attack_gain
    }

    pub const fn maximum_attack_gain(self) -> i32 {
        self.maximum_attack_gain
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        if !is_swordship_state_id(skill_id) {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(skill_id, reader.read_i32()?, reader.read_i32()?))
    }

    pub fn encoded(self) -> [u8; SWORDSHIP_STATE_BYTES] {
        let mut bytes = [0; SWORDSHIP_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.minimum_attack_gain.to_le_bytes());
        bytes[8..].copy_from_slice(&self.maximum_attack_gain.to_le_bytes());
        bytes
    }

    pub const fn apply_player_minimum(self, value: u32) -> u32 {
        apply_player_gain(value, self.minimum_attack_gain)
    }

    pub const fn apply_player_maximum(self, value: u32) -> u32 {
        apply_player_gain(value, self.maximum_attack_gain)
    }
}

const fn apply_player_gain(value: u32, gain: i32) -> u32 {
    let sum = value.wrapping_add((gain as i16 as i32) as u32);
    if sum > i32::MAX as u32 {
        i32::MAX as u32
    } else {
        sum
    }
}
