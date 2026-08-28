//! Каноническое состояние семейства `CWuXing*State`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает общий
//! 0x5c-байтный набор параметров и одинаковый `OnUpdateProperties` для пяти
//! элементов. Только Metal дополнительно применяет `MAX_MP_GAIN`. Порядок
//! состояний сохраняет исходную позицию при замене того же skill ID.

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::setup::globesetup::GlobePlayerPropertyCoefficients;

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
    pub(crate) blast_attack_scale_percent: i32,
    pub(crate) blast_defense_scale_percent: i32,
    pub(crate) critical_rate_percent: i32,
    pub(crate) element_blast_attack_scale_percent: i32,
    pub(crate) element_blast_defense_scale_percent: i32,
    pub(crate) full_miss_scale_percent: i32,
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
}

impl WuXingState {
    pub(crate) const fn new(
        skill_id: u32,
        kind: WuXingKind,
        parameters: WuXingStateParameters,
    ) -> Self {
        Self { skill_id, kind, parameters }
    }

    pub(crate) const fn skill_id(self) -> u32 { self.skill_id }

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
            parameters.blast_attack_scale_percent,
            1.0,
        );
        apply_scale(
            &mut properties.blast_defense_scale_bits,
            parameters.blast_defense_scale_percent,
            0.01,
        );
        apply_scale(
            &mut properties.critical_rate_bits,
            parameters.critical_rate_percent,
            1.0,
        );
        apply_scale(
            &mut properties.element_blast_attack_scale_bits,
            parameters.element_blast_attack_scale_percent,
            1.0,
        );
        apply_scale(
            &mut properties.element_blast_defense_scale_bits,
            parameters.element_blast_defense_scale_percent,
            0.01,
        );
        apply_scale(
            &mut properties.full_miss_scale_bits,
            parameters.full_miss_scale_percent,
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
    (value as f32 * coefficient).round() as i32
}

fn apply_scale(bits: &mut u32, percent: i32, minimum: f32) {
    if percent == 0 {
        return;
    }
    let value = f32::from_bits(*bits) + percent as f32 * 0.01;
    *bits = if value < minimum { minimum } else { value }.to_bits();
}

fn add_with_floor(value: i32, delta: i32, floor: i32) -> i32 {
    if delta == 0 { value } else { value.wrapping_add(delta).max(floor) }
}
