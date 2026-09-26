//! Данные и 96-байтная запись постоянных состояний У-син.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/wuxing*state.cpp/.h`.
//! Vtable пяти вариантов направляют writer и reader на общие тела:
//! ID DWORD и 0x5c сырых байт с `[this+0x38]`.
//! Два байта между WORD и DWORD сохраняются без интерпретации; у Metal —
//! собственный property callback.
//! Порядок, ветви и x87-точность сверены по property callbacks (общий
//! `0x5E0080`, Metal `0x5E08C0` — полное тело, VERIFIED): промежуточных
//! f32/f64-сбросов нет, продукты `fild × f32`-коэффициент и `percent × 0.01f`
//! остаются в ST0 до единственного конечного `fstp`/`fistp` (модель
//! неокруглённого x87-продукта через f64 — прецедент lifeshield), Metal
//! добавляет ровно один блок MAX_MP с клампом по `i32::MAX`.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

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

/// Только поля действующих свойств, которые меняет WuXing callback.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WuXingProperties {
    pub blast_attack_scale_bits: u32,
    pub blast_defense_scale_bits: u32,
    pub constitution: u32,
    pub critical_rate_bits: u32,
    pub defense: u32,
    pub dexterity: u32,
    pub element_blast_attack_scale_bits: u32,
    pub element_blast_defense_scale_bits: u32,
    pub element_modify: i32,
    pub element_resistance: u32,
    pub full_miss_scale_bits: u32,
    pub intelligence: u32,
    pub maximum_attack: u32,
    pub maximum_hp: u32,
    pub maximum_mp: u32,
    pub minimum_attack: u32,
    pub restored_hp_fight: i32,
    pub restored_hp_peace: i32,
    pub restored_mp_fight: i32,
    pub restored_mp_peace: i32,
    pub resume_hp_fight: i32,
    pub resume_hp_peace: i32,
    pub resume_mp_fight: i32,
    pub resume_mp_peace: i32,
    pub strength: u32,
}

/// Коэффициенты текущей профессии из установленного GlobeSetup.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WuXingCoefficients {
    pub str_to_max_attack: f32,
    pub dex_to_min_attack: f32,
    pub con_to_max_hp: f32,
    pub con_to_defense: f32,
    pub int_to_max_mp: f32,
    pub int_to_resistant: f32,
    pub int_to_element: f32,
}

pub fn apply_wuxing_to_properties(
    state: WuXingState,
    mut properties: WuXingProperties,
    coefficients: WuXingCoefficients,
) -> WuXingProperties {
    let parameters = state.parameters();
    properties.element_modify = properties
        .element_modify
        .wrapping_add(i32::from(parameters.element_modify));
    properties.maximum_attack = capped_add_nonzero(
        properties.maximum_attack,
        i32::from(parameters.maximum_attack),
    );
    properties.minimum_attack = capped_add_nonzero(
        properties.minimum_attack,
        i32::from(parameters.minimum_attack),
    );
    properties.defense = capped_add_nonzero(properties.defense, i32::from(parameters.defense));
    properties.element_resistance = capped_add_nonzero(
        properties.element_resistance,
        i32::from(parameters.element_resistance),
    );

    if parameters.strength > 0 {
        properties.strength = capped_add(properties.strength, parameters.strength);
        properties.maximum_attack = capped_add(
            properties.maximum_attack,
            derived(parameters.strength, coefficients.str_to_max_attack),
        );
    }
    if parameters.dexterity > 0 {
        properties.dexterity = capped_add(properties.dexterity, parameters.dexterity);
        properties.minimum_attack = capped_add(
            properties.minimum_attack,
            derived(parameters.dexterity, coefficients.dex_to_min_attack),
        );
    }
    if parameters.constitution > 0 {
        properties.constitution = capped_add(properties.constitution, parameters.constitution);
        properties.maximum_hp = capped_add(
            properties.maximum_hp,
            derived(parameters.constitution, coefficients.con_to_max_hp),
        );
        properties.defense = capped_add(
            properties.defense,
            derived(parameters.constitution, coefficients.con_to_defense),
        );
    }
    if parameters.intelligence > 0 {
        properties.intelligence = capped_add(properties.intelligence, parameters.intelligence);
        properties.maximum_mp = capped_add(
            properties.maximum_mp,
            derived(parameters.intelligence, coefficients.int_to_max_mp),
        );
        properties.element_resistance = capped_add(
            properties.element_resistance,
            derived(parameters.intelligence, coefficients.int_to_resistant),
        );
        properties.element_modify = properties.element_modify.wrapping_add(derived(
            parameters.intelligence,
            coefficients.int_to_element,
        ));
    }
    if parameters.maximum_hp > 0 {
        properties.maximum_hp = capped_add(properties.maximum_hp, parameters.maximum_hp);
    }
    if state.kind() == WuXingKind::Metal && parameters.maximum_mp != 0 {
        properties.maximum_mp = properties
            .maximum_mp
            .wrapping_add(parameters.maximum_mp)
            .min(i32::MAX as u32);
    }

    apply_scale(
        &mut properties.blast_attack_scale_bits,
        parameters.blast_attack_scale_bits,
        1.0,
    );
    apply_scale(
        &mut properties.blast_defense_scale_bits,
        parameters.blast_defense_scale_bits,
        0.01,
    );
    apply_scale(
        &mut properties.critical_rate_bits,
        parameters.critical_rate_bits,
        1.0,
    );
    apply_scale(
        &mut properties.element_blast_attack_scale_bits,
        parameters.element_blast_attack_scale_bits,
        1.0,
    );
    apply_scale(
        &mut properties.element_blast_defense_scale_bits,
        parameters.element_blast_defense_scale_bits,
        0.01,
    );
    apply_scale(
        &mut properties.full_miss_scale_bits,
        parameters.full_miss_scale_bits,
        0.01,
    );

    properties.resume_hp_peace =
        add_with_floor(properties.resume_hp_peace, parameters.resume_hp_peace, 1000);
    properties.resume_mp_peace =
        add_with_floor(properties.resume_mp_peace, parameters.resume_mp_peace, 1000);
    properties.resume_hp_fight =
        add_with_floor(properties.resume_hp_fight, parameters.resume_hp_fight, 1000);
    properties.resume_mp_fight =
        add_with_floor(properties.resume_mp_fight, parameters.resume_mp_fight, 1000);
    properties.restored_hp_peace = add_with_floor(
        properties.restored_hp_peace,
        parameters.restored_hp_peace,
        0,
    );
    properties.restored_mp_peace = add_with_floor(
        properties.restored_mp_peace,
        parameters.restored_mp_peace,
        0,
    );
    properties.restored_hp_fight = add_with_floor(
        properties.restored_hp_fight,
        parameters.restored_hp_fight,
        0,
    );
    properties.restored_mp_fight = add_with_floor(
        properties.restored_mp_fight,
        parameters.restored_mp_fight,
        0,
    );
    properties
}

fn capped_add(value: u32, delta: i32) -> u32 {
    value.wrapping_add(delta as u32).min(i32::MAX as u32)
}

fn capped_add_nonzero(value: u32, delta: i32) -> u32 {
    if delta == 0 {
        value
    } else {
        capped_add(value, delta)
    }
}

fn derived(value: i32, coefficient: f32) -> i32 {
    crate::combat::truncate_original(f64::from(value) * f64::from(coefficient))
}

fn apply_scale(bits: &mut u32, percent_bits: u32, minimum: f32) {
    let percent = f32::from_bits(percent_bits);
    if percent == 0.0 {
        return;
    }
    let value =
        (f64::from(f32::from_bits(*bits)) + f64::from(percent) * f64::from(0.01_f32)) as f32;
    *bits = if value < minimum { minimum } else { value }.to_bits();
}

fn add_with_floor(value: i32, delta: i32, floor: i32) -> i32 {
    if delta == 0 {
        value
    } else {
        value.wrapping_add(delta).max(floor)
    }
}
