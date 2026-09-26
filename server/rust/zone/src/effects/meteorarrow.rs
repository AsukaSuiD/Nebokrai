//! Запас метеорных стрел `CMeteorArrowState` в Zone.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/meteorarrowstate.cpp` и `meteorarrowstate.h`.
//! Опорные адреса (конструктор, vtable, writer, reader, добавление):
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const METEOR_ARROW_MASS_SKILL_ID: u32 = 0xcc;
pub const METEOR_ARROW_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeteorArrowState {
    arrows: i32,
    maximum_arrows: u32,
}

impl MeteorArrowState {
    pub const fn new(maximum_arrows: u32) -> Self {
        Self {
            arrows: 0,
            maximum_arrows,
        }
    }

    pub const fn skill_id(self) -> u32 {
        METEOR_ARROW_MASS_SKILL_ID
    }

    pub const fn arrows(self) -> i32 {
        self.arrows
    }

    pub const fn additional_data(self) -> i32 {
        self.arrows
    }

    /// Вызывающий Game предварительно проверяет наличие живого S.
    /// Успех означает, что Add дошёл до visual, даже при нулевом amount.
    pub fn add_arrows(&mut self, amount: u32) -> bool {
        let maximum = self.maximum_arrows as i32;
        if self.arrows >= maximum {
            return false;
        }
        self.arrows = self.arrows.wrapping_add(amount as i32).min(maximum);
        true
    }

    pub fn encoded(self) -> [u8; METEOR_ARROW_STATE_BYTES] {
        let mut bytes = [0; METEOR_ARROW_STATE_BYTES];
        bytes[..4].copy_from_slice(&METEOR_ARROW_MASS_SKILL_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.arrows.to_le_bytes());
        bytes[8..].copy_from_slice(&self.maximum_arrows.to_le_bytes());
        bytes
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != METEOR_ARROW_MASS_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self {
            arrows: reader.read_i32()?,
            maximum_arrows: reader.read_u32()?,
        })
    }
}
