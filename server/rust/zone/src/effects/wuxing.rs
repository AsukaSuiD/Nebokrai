//! Данные и 96-байтная запись постоянных состояний У-син.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/wuxing*state.cpp/.h`.
//! Vtable пяти вариантов направляют writer на VA `0x005E0030`, reader
//! на VA `0x005E0880`: ID DWORD и 0x5c сырых байт с `[this+0x38]`.
//! Два байта между WORD и DWORD сохраняются без интерпретации.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub const WUXING_STATE_BYTES: usize = 96;
pub const WUXING_METAL_STATE_ID: u32 = 0x353;
pub const WUXING_WOOD_STATE_ID: u32 = 0x354;
pub const WUXING_WATER_STATE_ID: u32 = 0x355;
pub const WUXING_FIRE_STATE_ID: u32 = 0x356;
pub const WUXING_EARTH_STATE_ID: u32 = 0x357;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WuXingKind {
    Metal,
    Wood,
    Water,
    Fire,
    Earth,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WuXingStateParameters {
    pub element_modify: i16,
    pub maximum_attack: i16,
    pub minimum_attack: i16,
    pub defense: i16,
    pub element_resistance: i16,
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub maximum_hp: i32,
    pub maximum_mp: u32,
    pub blast_attack_scale_bits: u32,
    pub blast_defense_scale_bits: u32,
    pub critical_rate_bits: u32,
    pub element_blast_attack_scale_bits: u32,
    pub element_blast_defense_scale_bits: u32,
    pub full_miss_scale_bits: u32,
    pub resume_hp_peace: i32,
    pub resume_mp_peace: i32,
    pub resume_hp_fight: i32,
    pub resume_mp_fight: i32,
    pub restored_hp_peace: i32,
    pub restored_mp_peace: i32,
    pub restored_hp_fight: i32,
    pub restored_mp_fight: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WuXingState {
    skill_id: u32,
    kind: WuXingKind,
    parameters: WuXingStateParameters,
    alignment: u16,
}

impl WuXingState {
    pub const fn new(skill_id: u32, kind: WuXingKind, parameters: WuXingStateParameters) -> Self {
        Self {
            skill_id,
            kind,
            parameters,
            alignment: 0,
        }
    }

    pub const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let kind = kind_for_skill_id(skill_id).ok_or(LegacyReadBlock {
            offset,
            needed: 4,
            available: payload.len().saturating_sub(offset),
        })?;
        let element_modify = reader.read_i16()?;
        let maximum_attack = reader.read_i16()?;
        let minimum_attack = reader.read_i16()?;
        let defense = reader.read_i16()?;
        let element_resistance = reader.read_i16()?;
        let alignment = reader.read_u16()?;
        let parameters = WuXingStateParameters {
            element_modify,
            maximum_attack,
            minimum_attack,
            defense,
            element_resistance,
            strength: reader.read_i32()?,
            dexterity: reader.read_i32()?,
            constitution: reader.read_i32()?,
            intelligence: reader.read_i32()?,
            maximum_hp: reader.read_i32()?,
            maximum_mp: reader.read_u32()?,
            blast_attack_scale_bits: reader.read_u32()?,
            blast_defense_scale_bits: reader.read_u32()?,
            element_blast_attack_scale_bits: reader.read_u32()?,
            element_blast_defense_scale_bits: reader.read_u32()?,
            full_miss_scale_bits: reader.read_u32()?,
            critical_rate_bits: reader.read_u32()?,
            resume_hp_peace: reader.read_i32()?,
            resume_mp_peace: reader.read_i32()?,
            resume_hp_fight: reader.read_i32()?,
            resume_mp_fight: reader.read_i32()?,
            restored_hp_peace: reader.read_i32()?,
            restored_mp_peace: reader.read_i32()?,
            restored_hp_fight: reader.read_i32()?,
            restored_mp_fight: reader.read_i32()?,
        };
        Ok(Self {
            skill_id,
            kind,
            parameters,
            alignment,
        })
    }

    pub fn encoded(self) -> [u8; WUXING_STATE_BYTES] {
        let parameters = self.parameters;
        let mut bytes = Vec::with_capacity(WUXING_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(self.skill_id);
        writer.write_i16(parameters.element_modify);
        writer.write_i16(parameters.maximum_attack);
        writer.write_i16(parameters.minimum_attack);
        writer.write_i16(parameters.defense);
        writer.write_i16(parameters.element_resistance);
        writer.write_u16(self.alignment);
        for value in [
            parameters.strength,
            parameters.dexterity,
            parameters.constitution,
            parameters.intelligence,
            parameters.maximum_hp,
        ] {
            writer.write_i32(value);
        }
        writer.write_u32(parameters.maximum_mp);
        for value in [
            parameters.blast_attack_scale_bits,
            parameters.blast_defense_scale_bits,
            parameters.element_blast_attack_scale_bits,
            parameters.element_blast_defense_scale_bits,
            parameters.full_miss_scale_bits,
            parameters.critical_rate_bits,
        ] {
            writer.write_u32(value);
        }
        for value in [
            parameters.resume_hp_peace,
            parameters.resume_mp_peace,
            parameters.resume_hp_fight,
            parameters.resume_mp_fight,
            parameters.restored_hp_peace,
            parameters.restored_mp_peace,
            parameters.restored_hp_fight,
            parameters.restored_mp_fight,
        ] {
            writer.write_i32(value);
        }
        bytes.try_into().expect("размер состояния У-син фиксирован")
    }

    pub const fn kind(self) -> WuXingKind {
        self.kind
    }

    pub const fn parameters(self) -> WuXingStateParameters {
        self.parameters
    }
}

pub const fn kind_for_skill_id(skill_id: u32) -> Option<WuXingKind> {
    match skill_id {
        WUXING_METAL_STATE_ID => Some(WuXingKind::Metal),
        WUXING_WOOD_STATE_ID => Some(WuXingKind::Wood),
        WUXING_WATER_STATE_ID => Some(WuXingKind::Water),
        WUXING_FIRE_STATE_ID => Some(WuXingKind::Fire),
        WUXING_EARTH_STATE_ID => Some(WuXingKind::Earth),
        _ => None,
    }
}
