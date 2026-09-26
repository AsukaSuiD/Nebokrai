//! Сохраняемые данные периодической потери крови Zone.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/bloodlossstate.cpp` и `bloodlossstate.h`.
//! Vtable состояния связывает writer/reader; хвост записи содержит
//! два DWORD с битами f32 и два WORD. Игровой RNG остаётся у Game.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use crate::combat::MasterInfo;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyWriter};

use super::{
    PeriodicAttackCore, PeriodicAttackRecord, encode_periodic_state,
    encode_periodic_state_for_install,
};

pub const BLOOD_LOSS_STATE_ID: u32 = 0x21d;
pub const BLOOD_LOSS_STATE_BYTES: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BloodLossState {
    core: PeriodicAttackCore,
    damage_factor_bits: u32,
    damage_modifier_bits: u32,
    minimum_attack: u16,
    maximum_attack: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct BloodLossAttackSeed {
    pub damage_factor: f32,
    pub damage_modifier: f32,
    pub minimum_attack: u16,
    pub maximum_attack: u16,
}

impl BloodLossState {
    pub const fn new(
        master: MasterInfo,
        keep_time_ms: u32,
        frequency_ms: u32,
        damage_factor: f32,
        damage_modifier: f32,
        minimum_attack: u16,
        maximum_attack: u16,
    ) -> Self {
        Self {
            core: PeriodicAttackCore::new(master, keep_time_ms, frequency_ms),
            damage_factor_bits: damage_factor.to_bits(),
            damage_modifier_bits: damage_modifier.to_bits(),
            minimum_attack,
            maximum_attack,
        }
    }

    pub const fn skill_id(&self) -> u32 {
        BLOOD_LOSS_STATE_ID
    }

    pub fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        self.core.client_state_time(now)
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let (core, mut reader) =
            PeriodicAttackCore::decode(payload, offset, BLOOD_LOSS_STATE_ID, now)?;
        Ok(Self {
            core,
            damage_factor_bits: reader.read_u32()?,
            damage_modifier_bits: reader.read_u32()?,
            minimum_attack: reader.read_u16()?,
            maximum_attack: reader.read_u16()?,
        })
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; BLOOD_LOSS_STATE_BYTES] {
        encode_periodic_state(self, now)
            .try_into()
            .expect("запись потери крови содержит 64 байта")
    }

    pub fn encoded_for_install(&self) -> [u8; BLOOD_LOSS_STATE_BYTES] {
        encode_periodic_state_for_install(self)
            .try_into()
            .expect("запись потери крови содержит 64 байта")
    }

    pub fn core_mut(&mut self) -> &mut PeriodicAttackCore {
        &mut self.core
    }

    pub fn attack_seed(&self) -> BloodLossAttackSeed {
        BloodLossAttackSeed {
            damage_factor: f32::from_bits(self.damage_factor_bits),
            damage_modifier: f32::from_bits(self.damage_modifier_bits),
            minimum_attack: self.minimum_attack,
            maximum_attack: self.maximum_attack,
        }
    }
}

impl PeriodicAttackRecord for BloodLossState {
    const STATE_ID: u32 = BLOOD_LOSS_STATE_ID;
    const RECORD_BYTES: usize = BLOOD_LOSS_STATE_BYTES;

    fn core(&self) -> &PeriodicAttackCore {
        &self.core
    }
    fn encode_attack(&self, writer: &mut LegacyWriter<'_>) {
        writer.write_u32(self.damage_factor_bits);
        writer.write_u32(self.damage_modifier_bits);
        writer.write_u16(self.minimum_attack);
        writer.write_u16(self.maximum_attack);
    }
}
