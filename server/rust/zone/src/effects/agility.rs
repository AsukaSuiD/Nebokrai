//! Состояния CAgility, CNatural, CRapture и временная CAgilityState2.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/agilitystate.cpp`, `naturalstate.cpp`, `rapturestate .cpp`,
//! `agilitystate2.cpp` и соответствующие `.h`.
//! Конструкторы VA `0x005F4195/0x005F3795/0x005F3C85` задают ID;
//! vtable `0x00660774/0x006606B4/0x00660714` связывают общий writer
//! `0x005F3E40`, reader `0x005F4420` и отдельные property callbacks.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

use super::time::timed_client_state_time;

pub const AGILITY_SKILL_ID: u32 = 0xda;
pub const NATURAL_SKILL_ID: u32 = 0xdc;
pub const RAPTURE_SKILL_ID: u32 = 0xdb;
pub const PERSISTENT_AGILITY_FAMILY_STATE_BYTES: usize = 6;
pub const AGILITY_2_SKILL_ID: u32 = 0x81;
pub const AGILITY_STATE_2_BYTES: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistentAgilityFamilyState {
    Agility { full_miss: u16 },
    Natural { element_resistance_gain: u16 },
    Rapture { blast_attack_gain: u16 },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PersistentAgilityProperties {
    pub full_miss: u16,
    pub element_resistance: u32,
    pub blast_attack: u16,
}

impl PersistentAgilityFamilyState {
    pub const fn skill_id(self) -> u32 {
        match self {
            Self::Agility { .. } => AGILITY_SKILL_ID,
            Self::Natural { .. } => NATURAL_SKILL_ID,
            Self::Rapture { .. } => RAPTURE_SKILL_ID,
        }
    }

    pub const fn is_known_skill(skill_id: u32) -> bool {
        matches!(
            skill_id,
            AGILITY_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID
        )
    }

    pub fn apply_to_properties(
        self,
        mut properties: PersistentAgilityProperties,
    ) -> PersistentAgilityProperties {
        match self {
            Self::Agility { full_miss } => {
                properties.full_miss = properties.full_miss.wrapping_add(full_miss);
            }
            Self::Natural {
                element_resistance_gain,
            } => {
                properties.element_resistance = properties
                    .element_resistance
                    .wrapping_add(u32::from(element_resistance_gain))
                    .min(i32::MAX as u32);
            }
            Self::Rapture { blast_attack_gain } => {
                properties.blast_attack = properties.blast_attack.wrapping_add(blast_attack_gain);
            }
        }
        properties
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let value = reader.read_u16()?;
        match skill_id {
            AGILITY_SKILL_ID => Ok(Self::Agility { full_miss: value }),
            NATURAL_SKILL_ID => Ok(Self::Natural {
                element_resistance_gain: value,
            }),
            RAPTURE_SKILL_ID => Ok(Self::Rapture {
                blast_attack_gain: value,
            }),
            _ => Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            }),
        }
    }

    pub fn encoded(self) -> [u8; PERSISTENT_AGILITY_FAMILY_STATE_BYTES] {
        let value = match self {
            Self::Agility { full_miss } => full_miss,
            Self::Natural {
                element_resistance_gain,
            } => element_resistance_gain,
            Self::Rapture { blast_attack_gain } => blast_attack_gain,
        };
        let mut bytes = [0; PERSISTENT_AGILITY_FAMILY_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id().to_le_bytes());
        bytes[4..].copy_from_slice(&value.to_le_bytes());
        bytes
    }
}

/// Writer VA `0x005F1050` пишет ID, остаток через virtual getter `+0x30`
/// и WORD; reader VA `0x005F48E0` фиксирует часы до чтения DWORD и WORD.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgilityState2 {
    full_miss: u16,
    started_at_ms: u32,
    keep_time_ms: i32,
}

impl AgilityState2 {
    pub const fn new(full_miss: u16, started_at_ms: u32, keep_time_ms: i32) -> Self {
        Self {
            full_miss,
            started_at_ms,
            keep_time_ms,
        }
    }

    pub const fn skill_id(self) -> u32 {
        AGILITY_2_SKILL_ID
    }

    pub fn start_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn apply_to_full_miss(self, full_miss: u16) -> u16 {
        full_miss.wrapping_add(self.full_miss)
    }

    /// AI VA `0x005D60B0`: End вызывается только при unsigned `now > deadline`.
    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms as u32) < now_ms
    }

    pub fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(
            self.started_at_ms,
            self.keep_time_ms as u32,
            now_milliseconds,
        ) as i32
    }

    pub fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != AGILITY_2_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let keep_time_ms = reader.read_i32()?;
        Ok(Self::new(reader.read_u16()?, now_ms, keep_time_ms))
    }

    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; AGILITY_STATE_2_BYTES] {
        let mut bytes = [0; AGILITY_STATE_2_BYTES];
        bytes[..4].copy_from_slice(&AGILITY_2_SKILL_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.client_time(now).to_le_bytes());
        bytes[8..].copy_from_slice(&self.full_miss.to_le_bytes());
        bytes
    }

    pub fn encoded_for_install(self) -> [u8; AGILITY_STATE_2_BYTES] {
        let mut bytes = [0; AGILITY_STATE_2_BYTES];
        bytes[..4].copy_from_slice(&AGILITY_2_SKILL_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.keep_time_ms.to_le_bytes());
        bytes[8..].copy_from_slice(&self.full_miss.to_le_bytes());
        bytes
    }
}
