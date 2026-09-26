//! Данные трёх периодических состояний LeafCut в Zone.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/leafcutstate*.cpp` и соответствующие `.h`.
//! Конструкторы задают ID `0x6B/0x80/0x8F`.
//! Все три таблицы используют общие writer, reader и getter срока;
//! хвост записи — 2 DWORD и 4 WORD.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use crate::combat::MasterInfo;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyWriter};

use super::{
    PeriodicAttackCore, PeriodicAttackRecord, encode_periodic_state,
    encode_periodic_state_for_install,
};

pub const LEAF_CUT_STATE_ID: u32 = 0x6b;
pub const LEAF_CUT_2_STATE_ID: u32 = 0x80;
pub const LEAF_CUT_3_STATE_ID: u32 = 0x8f;
pub const LEAF_CUT_STATE_BYTES: usize = 68;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeafCutState<const ID: u32 = LEAF_CUT_STATE_ID> {
    core: PeriodicAttackCore,
    damage_factor_bits: u32,
    damage_modifier_bits: u32,
    minimum_attack: u16,
    maximum_attack: u16,
    element_attack: u16,
    soul_attack: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct LeafCutAttackSeed {
    pub damage_factor: f32,
    pub damage_modifier: f32,
    pub minimum_attack: u16,
    pub maximum_attack: u16,
    pub element_attack: u16,
    pub soul_attack: u16,
}

impl<const ID: u32> LeafCutState<ID> {
    #[allow(
        clippy::too_many_arguments,
        reason = "параметры снимка атаки сохраняют контракт состояния"
    )]
    pub const fn new(
        master: MasterInfo,
        keep_time_ms: u32,
        frequency_ms: u32,
        damage_factor: f32,
        damage_modifier: f32,
        minimum_attack: u16,
        maximum_attack: u16,
        element_attack: u16,
        soul_attack: u16,
    ) -> Self {
        Self {
            core: PeriodicAttackCore::new(master, keep_time_ms, frequency_ms),
            damage_factor_bits: damage_factor.to_bits(),
            damage_modifier_bits: damage_modifier.to_bits(),
            minimum_attack,
            maximum_attack,
            element_attack,
            soul_attack,
        }
    }

    pub const fn skill_id(&self) -> u32 {
        ID
    }

    pub fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        self.core.client_state_time(now)
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let (core, mut reader) = PeriodicAttackCore::decode(payload, offset, ID, now)?;
        Ok(Self {
            core,
            damage_factor_bits: reader.read_u32()?,
            damage_modifier_bits: reader.read_u32()?,
            minimum_attack: reader.read_u16()?,
            maximum_attack: reader.read_u16()?,
            element_attack: reader.read_u16()?,
            soul_attack: reader.read_u16()?,
        })
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; LEAF_CUT_STATE_BYTES] {
        encode_periodic_state(self, now)
            .try_into()
            .expect("запись рассечения содержит 68 байт")
    }

    pub fn encoded_for_install(&self) -> [u8; LEAF_CUT_STATE_BYTES] {
        encode_periodic_state_for_install(self)
            .try_into()
            .expect("запись рассечения содержит 68 байт")
    }

    pub fn core_mut(&mut self) -> &mut PeriodicAttackCore {
        &mut self.core
    }

    pub fn attack_seed(&self) -> LeafCutAttackSeed {
        LeafCutAttackSeed {
            damage_factor: f32::from_bits(self.damage_factor_bits),
            damage_modifier: f32::from_bits(self.damage_modifier_bits),
            minimum_attack: self.minimum_attack,
            maximum_attack: self.maximum_attack,
            element_attack: self.element_attack,
            soul_attack: self.soul_attack,
        }
    }
}

impl<const ID: u32> PeriodicAttackRecord for LeafCutState<ID> {
    const STATE_ID: u32 = ID;
    const RECORD_BYTES: usize = LEAF_CUT_STATE_BYTES;

    fn core(&self) -> &PeriodicAttackCore {
        &self.core
    }
    fn encode_attack(&self, writer: &mut LegacyWriter<'_>) {
        writer.write_u32(self.damage_factor_bits);
        writer.write_u32(self.damage_modifier_bits);
        writer.write_u16(self.minimum_attack);
        writer.write_u16(self.maximum_attack);
        writer.write_u16(self.element_attack);
        writer.write_u16(self.soul_attack);
    }
}
