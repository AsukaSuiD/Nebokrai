//! Счётчик и сохраняемые поля CEnergyHoldingState.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! идентификаторы —
//! docs/reconstruction/gameserver-npc-and-regions.md#идентификаторы-сборки.
//! Serialize/Unserialize общие с SoulCollect
//! (appserver/skills/energyholdingstate.cpp/.h).
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const ENERGY_HOLDING_STATE_ID: u32 = 0x89;
pub const ENERGY_HOLDING_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnergyHoldingState {
    skill_level: u32,
    energy_count: u32,
    parameter_percent: u32,
}

impl EnergyHoldingState {
    pub const fn new(skill_level: u32, parameter_percent: u32) -> Self {
        Self {
            skill_level,
            energy_count: 0,
            parameter_percent,
        }
    }
    pub const fn skill_id(self) -> u32 {
        ENERGY_HOLDING_STATE_ID
    }
    pub const fn skill_level(self) -> u32 {
        self.skill_level
    }
    pub const fn energy_count(self) -> u32 {
        self.energy_count
    }
    pub const fn parameter_percent(self) -> u32 {
        self.parameter_percent
    }

    /// CInverseChopped::CalculateAttackPower: unsigned поля переходят в double
    /// до умножения на коэффициент 0.01 и прибавления единицы.
    pub fn attack_multiplier(self) -> f64 {
        (f64::from(self.parameter_percent) * 0.01) * f64::from(self.energy_count) + 1.0
    }

    /// Game предварительно разрешает живого User; число меняется до visual.
    pub fn add_energy(&mut self) -> bool {
        if self.energy_count >= self.skill_level {
            return false;
        }
        self.energy_count = self.energy_count.wrapping_add(1);
        true
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ENERGY_HOLDING_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self {
            skill_level: reader.read_u32()?,
            energy_count: reader.read_u32()?,
            parameter_percent: 0,
        })
    }

    pub fn encoded(self) -> [u8; ENERGY_HOLDING_STATE_BYTES] {
        let mut bytes = [0; ENERGY_HOLDING_STATE_BYTES];
        for (index, value) in [ENERGY_HOLDING_STATE_ID, self.skill_level, self.energy_count]
            .into_iter()
            .enumerate()
        {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub const fn client_fields(self) -> (u32, u32) {
        (0, 0)
    }
}
