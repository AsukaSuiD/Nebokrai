//! Постоянные состояния CWuXingMetal/Wood/Water/Fire/EarthState.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/wuxing*state.cpp/.h`.
//! Writer VA `0x005E0030` пишет ID и сырой блок 0x5c байт из `[this+0x38]`;
//! reader VA `0x005E0880` копирует обратно 0x5c байт. Обе функции стоят
//! в vtable всех пяти вариантов; детали — в `docs/gameplay/attributes-and-states.md`.
//! Данные и раскладка перенесены в `zone/effects/wuxing.rs`; здесь остаются
//! доступ к живому игроку; формула действует над проекцией Zone.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::effects::{
    WUXING_STATE_BYTES, WuXingCoefficients, WuXingKind, WuXingProperties, WuXingState,
    WuXingStateParameters, apply_wuxing_to_properties, kind_for_skill_id,
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
                let projection = WuXingProperties {
                    blast_attack_scale_bits: properties.blast_attack_scale_bits,
                    blast_defense_scale_bits: properties.blast_defense_scale_bits,
                    constitution: properties.constitution,
                    critical_rate_bits: properties.critical_rate_bits,
                    defense: properties.defense,
                    dexterity: properties.dexterity,
                    element_blast_attack_scale_bits: properties.element_blast_attack_scale_bits,
                    element_blast_defense_scale_bits: properties.element_blast_defense_scale_bits,
                    element_modify: properties.element_modify,
                    element_resistance: properties.element_resistance,
                    full_miss_scale_bits: properties.full_miss_scale_bits,
                    intelligence: properties.intelligence,
                    maximum_attack: properties.maximum_attack,
                    maximum_hp: properties.maximum_hp,
                    maximum_mp: properties.maximum_mp,
                    minimum_attack: properties.minimum_attack,
                    restored_hp_fight: properties.restored_hp_fight,
                    restored_hp_peace: properties.restored_hp_peace,
                    restored_mp_fight: properties.restored_mp_fight,
                    restored_mp_peace: properties.restored_mp_peace,
                    resume_hp_fight: properties.resume_hp_fight,
                    resume_hp_peace: properties.resume_hp_peace,
                    resume_mp_fight: properties.resume_mp_fight,
                    resume_mp_peace: properties.resume_mp_peace,
                    strength: properties.strength,
                };
                let coefficients = WuXingCoefficients {
                    str_to_max_attack: coefficients.str_to_max_attack[occupation],
                    dex_to_min_attack: coefficients.dex_to_min_attack[occupation],
                    con_to_max_hp: coefficients.con_to_max_hp[occupation],
                    con_to_defense: coefficients.con_to_defense[occupation],
                    int_to_max_mp: coefficients.int_to_max_mp[occupation],
                    int_to_resistant: coefficients.int_to_resistant[occupation],
                    int_to_element: coefficients.int_to_element[occupation],
                };
                let updated = apply_wuxing_to_properties(state, projection, coefficients);
                PlayerCombatProperties {
                    blast_attack_scale_bits: updated.blast_attack_scale_bits,
                    blast_defense_scale_bits: updated.blast_defense_scale_bits,
                    constitution: updated.constitution,
                    critical_rate_bits: updated.critical_rate_bits,
                    defense: updated.defense,
                    dexterity: updated.dexterity,
                    element_blast_attack_scale_bits: updated.element_blast_attack_scale_bits,
                    element_blast_defense_scale_bits: updated.element_blast_defense_scale_bits,
                    element_modify: updated.element_modify,
                    element_resistance: updated.element_resistance,
                    full_miss_scale_bits: updated.full_miss_scale_bits,
                    intelligence: updated.intelligence,
                    maximum_attack: updated.maximum_attack,
                    maximum_hp: updated.maximum_hp,
                    maximum_mp: updated.maximum_mp,
                    minimum_attack: updated.minimum_attack,
                    restored_hp_fight: updated.restored_hp_fight,
                    restored_hp_peace: updated.restored_hp_peace,
                    restored_mp_fight: updated.restored_mp_fight,
                    restored_mp_peace: updated.restored_mp_peace,
                    resume_hp_fight: updated.resume_hp_fight,
                    resume_hp_peace: updated.resume_hp_peace,
                    resume_mp_fight: updated.resume_mp_fight,
                    resume_mp_peace: updated.resume_mp_peace,
                    strength: updated.strength,
                    ..properties
                }
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
