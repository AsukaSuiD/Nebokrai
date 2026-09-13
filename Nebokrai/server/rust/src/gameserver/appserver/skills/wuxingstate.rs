//! Постоянные состояния CWuXingMetal/Wood/Water/Fire/EarthState.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/wuxing*state.cpp.
//! Свежий GetSufferer задаёт получателя свойств: NULL возвращает false,
//! другой тип не меняется. Только Metal применяет дополнительный MAX_MP.
//! Целые прибавки сохраняют wrapping и unsigned cap; производные параметры
//! усекаются к нулю до сложения. Проценты используют f32-коэффициент 0.01,
//! но произведение округляется до f32 только вместе с окончательной суммой.
//! Сравнение с нижним пределом сохраняет NaN, как исходные setters.
//! DB-запись — ID и 0x5c байт tagWuXingState, включая два непрозрачных байта
//! выравнивания. CriticalRate идёт после FullMiss, не в порядке QueryProperty.
//! Общий primary Begin сохраняет U/S после часов; перезапуск Begin(NULL,S)
//! не подменяет U держателем и не читает часы. End сначала ставит ended,
//! затем удаляет состояние через свежий U; visual и собственных таймеров нет.

use super::fightdefense::truncate_original;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::setup::globesetup::GlobePlayerPropertyCoefficients;

pub(crate) const WUXING_STATE_BYTES: usize = 96;

/// Свойства применяются к свежему S без IsEnded-gate и чтения часов.
pub(crate) fn update_wuxing_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let coefficients = game.globe_setup().player_property_coefficients();
    crate::gameserver::appserver::states::state::update_player_state_properties::<WuXingState>(
        game, region_id, holder, key, |state, player| {
            let occupation = usize::from(player.occupation()).min(2);
            player.update_state_combat_properties(|properties| state.apply_to_player(properties, coefficients, occupation));
        },
    )
}

pub(crate) fn restart_wuxing_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WuXingState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
}

pub(crate) fn end_wuxing_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WuXingState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, WUXING_STATE_BYTES)
}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WuXingKind {
    Metal,
    Wood,
    Water,
    Fire,
    Earth,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WuXingStateParameters {
    pub(crate) element_modify: i16,
    pub(crate) maximum_attack: i16,
    pub(crate) minimum_attack: i16,
    pub(crate) defense: i16,
    pub(crate) element_resistance: i16,
    pub(crate) strength: i32,
    pub(crate) dexterity: i32,
    pub(crate) constitution: i32,
    pub(crate) intelligence: i32,
    pub(crate) maximum_hp: i32,
    pub(crate) maximum_mp: u32,
    pub(crate) blast_attack_scale_bits: u32,
    pub(crate) blast_defense_scale_bits: u32,
    pub(crate) critical_rate_bits: u32,
    pub(crate) element_blast_attack_scale_bits: u32,
    pub(crate) element_blast_defense_scale_bits: u32,
    pub(crate) full_miss_scale_bits: u32,
    pub(crate) resume_hp_peace: i32,
    pub(crate) resume_mp_peace: i32,
    pub(crate) resume_hp_fight: i32,
    pub(crate) resume_mp_fight: i32,
    pub(crate) restored_hp_peace: i32,
    pub(crate) restored_mp_peace: i32,
    pub(crate) restored_hp_fight: i32,
    pub(crate) restored_mp_fight: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WuXingState {
    skill_id: u32,
    kind: WuXingKind,
    parameters: WuXingStateParameters,
    alignment: u16,
}

impl WuXingState {
    pub(crate) const fn new(
        skill_id: u32,
        kind: WuXingKind,
        parameters: WuXingStateParameters,
    ) -> Self {
        Self { skill_id, kind, parameters, alignment: 0 }
    }

    pub(crate) const fn skill_id(self) -> u32 { self.skill_id }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
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
        Ok(Self { skill_id, kind, parameters, alignment })
    }

    pub(crate) fn encoded(self) -> [u8; WUXING_STATE_BYTES] {
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

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
        coefficients: GlobePlayerPropertyCoefficients,
        occupation: usize,
    ) -> PlayerCombatProperties {
        let parameters = self.parameters;
        properties.element_modify = properties
            .element_modify
            .wrapping_add(i32::from(parameters.element_modify));
        properties.maximum_attack = capped_add_nonzero(properties.maximum_attack, i32::from(parameters.maximum_attack));
        properties.minimum_attack = capped_add_nonzero(properties.minimum_attack, i32::from(parameters.minimum_attack));
        properties.defense = capped_add_nonzero(properties.defense, i32::from(parameters.defense));
        properties.element_resistance = capped_add_nonzero(
            properties.element_resistance,
            i32::from(parameters.element_resistance),
        );

        if parameters.strength > 0 {
            properties.strength = capped_add(properties.strength, parameters.strength);
            properties.maximum_attack = capped_add(
                properties.maximum_attack,
                derived(parameters.strength, coefficients.str_to_max_attack[occupation]),
            );
        }
        if parameters.dexterity > 0 {
            properties.dexterity = capped_add(properties.dexterity, parameters.dexterity);
            properties.minimum_attack = capped_add(
                properties.minimum_attack,
                derived(parameters.dexterity, coefficients.dex_to_min_attack[occupation]),
            );
        }
        if parameters.constitution > 0 {
            properties.constitution = capped_add(properties.constitution, parameters.constitution);
            properties.maximum_hp = capped_add(
                properties.maximum_hp,
                derived(parameters.constitution, coefficients.con_to_max_hp[occupation]),
            );
            properties.defense = capped_add(
                properties.defense,
                derived(parameters.constitution, coefficients.con_to_defense[occupation]),
            );
        }
        if parameters.intelligence > 0 {
            properties.intelligence = capped_add(properties.intelligence, parameters.intelligence);
            properties.maximum_mp = capped_add(
                properties.maximum_mp,
                derived(parameters.intelligence, coefficients.int_to_max_mp[occupation]),
            );
            properties.element_resistance = capped_add(
                properties.element_resistance,
                derived(parameters.intelligence, coefficients.int_to_resistant[occupation]),
            );
            properties.element_modify = properties.element_modify.wrapping_add(derived(
                parameters.intelligence,
                coefficients.int_to_element[occupation],
            ));
        }
        if parameters.maximum_hp > 0 {
            properties.maximum_hp = capped_add(properties.maximum_hp, parameters.maximum_hp);
        }
        if self.kind == WuXingKind::Metal && parameters.maximum_mp != 0 {
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

        properties.resume_hp_peace = add_with_floor(properties.resume_hp_peace, parameters.resume_hp_peace, 1000);
        properties.resume_mp_peace = add_with_floor(properties.resume_mp_peace, parameters.resume_mp_peace, 1000);
        properties.resume_hp_fight = add_with_floor(properties.resume_hp_fight, parameters.resume_hp_fight, 1000);
        properties.resume_mp_fight = add_with_floor(properties.resume_mp_fight, parameters.resume_mp_fight, 1000);
        properties.restored_hp_peace = add_with_floor(properties.restored_hp_peace, parameters.restored_hp_peace, 0);
        properties.restored_mp_peace = add_with_floor(properties.restored_mp_peace, parameters.restored_mp_peace, 0);
        properties.restored_hp_fight = add_with_floor(properties.restored_hp_fight, parameters.restored_hp_fight, 0);
        properties.restored_mp_fight = add_with_floor(properties.restored_mp_fight, parameters.restored_mp_fight, 0);
        properties
    }
}

fn capped_add(value: u32, delta: i32) -> u32 {
    value.wrapping_add(delta as u32).min(i32::MAX as u32)
}

fn capped_add_nonzero(value: u32, delta: i32) -> u32 {
    if delta == 0 { value } else { capped_add(value, delta) }
}

fn derived(value: i32, coefficient: f32) -> i32 {
    truncate_original(f64::from(value) * f64::from(coefficient))
}

fn apply_scale(bits: &mut u32, percent_bits: u32, minimum: f32) {
    let percent = f32::from_bits(percent_bits);
    if percent == 0.0 {
        return;
    }
    let value = (f64::from(f32::from_bits(*bits))
        + f64::from(percent) * f64::from(0.01_f32)) as f32;
    *bits = if value < minimum { minimum } else { value }.to_bits();
}

pub(crate) const fn kind_for_skill_id(skill_id: u32) -> Option<WuXingKind> {
    match skill_id {
        0x353 => Some(WuXingKind::Metal),
        0x354 => Some(WuXingKind::Wood),
        0x355 => Some(WuXingKind::Water),
        0x356 => Some(WuXingKind::Fire),
        0x357 => Some(WuXingKind::Earth),
        _ => None,
    }
}

fn add_with_floor(value: i32, delta: i32, floor: i32) -> i32 {
    if delta == 0 { value } else { value.wrapping_add(delta).max(floor) }
}
