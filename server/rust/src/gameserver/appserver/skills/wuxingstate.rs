//! Постоянные состояния CWuXingMetal/Wood/Water/Fire/EarthState.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/wuxing*state.cpp/.h`.
//! Writer VA `0x005E0030` пишет ID и сырой блок 0x5c байт из `[this+0x38]`;
//! reader VA `0x005E0880` копирует обратно 0x5c байт. Обе функции стоят
//! в vtable всех пяти вариантов; детали — в `docs/gameplay/attributes-and-states.md`.
//! Данные и раскладка перенесены в `zone/effects/wuxing.rs`; здесь остаются
//! применение к живому игроку и формулы текущего Rust-кода. Их округления
//! и ветви оригинала ещё требуют отдельной прямой проверки.

use super::fightdefense::truncate_original;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
use crate::setup::globesetup::GlobePlayerPropertyCoefficients;

pub(crate) use nebokrai_zone::effects::{
    WUXING_STATE_BYTES, WuXingKind, WuXingState, WuXingStateParameters, kind_for_skill_id,
};

/// Свойства применяются к свежему S без IsEnded-gate и чтения часов.
pub(crate) fn update_wuxing_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let coefficients = game.globe_setup().player_property_coefficients();
    crate::gameserver::appserver::states::state::update_player_state_properties::<WuXingState>(
        game,
        region_id,
        holder,
        key,
        |state, player| {
            let occupation = usize::from(player.occupation()).min(2);
            player.update_state_combat_properties(|properties| {
                apply_wuxing_to_player(state, properties, coefficients, occupation)
            });
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
        .and_then(|shape| shape.applied_state::<WuXingState>(key))
        .is_none()
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
        .and_then(|shape| shape.applied_state::<WuXingState>(key))
        .is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, WUXING_STATE_BYTES)
}

fn apply_wuxing_to_player(
    state: WuXingState,
    mut properties: PlayerCombatProperties,
    coefficients: GlobePlayerPropertyCoefficients,
    occupation: usize,
) -> PlayerCombatProperties {
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
            derived(
                parameters.strength,
                coefficients.str_to_max_attack[occupation],
            ),
        );
    }
    if parameters.dexterity > 0 {
        properties.dexterity = capped_add(properties.dexterity, parameters.dexterity);
        properties.minimum_attack = capped_add(
            properties.minimum_attack,
            derived(
                parameters.dexterity,
                coefficients.dex_to_min_attack[occupation],
            ),
        );
    }
    if parameters.constitution > 0 {
        properties.constitution = capped_add(properties.constitution, parameters.constitution);
        properties.maximum_hp = capped_add(
            properties.maximum_hp,
            derived(
                parameters.constitution,
                coefficients.con_to_max_hp[occupation],
            ),
        );
        properties.defense = capped_add(
            properties.defense,
            derived(
                parameters.constitution,
                coefficients.con_to_defense[occupation],
            ),
        );
    }
    if parameters.intelligence > 0 {
        properties.intelligence = capped_add(properties.intelligence, parameters.intelligence);
        properties.maximum_mp = capped_add(
            properties.maximum_mp,
            derived(
                parameters.intelligence,
                coefficients.int_to_max_mp[occupation],
            ),
        );
        properties.element_resistance = capped_add(
            properties.element_resistance,
            derived(
                parameters.intelligence,
                coefficients.int_to_resistant[occupation],
            ),
        );
        properties.element_modify = properties.element_modify.wrapping_add(derived(
            parameters.intelligence,
            coefficients.int_to_element[occupation],
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
    truncate_original(f64::from(value) * f64::from(coefficient))
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
