//! Поклеточные атаки PoisonMoth (0xCF) и BloodRose (0xD0).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/poisonmoth.cpp
//! и bloodrose.cpp. Список GetShapes сохраняется отдельно для каждой клетки,
//! но допуск, состояние навыка и боевые getter-ы читаются живыми.
//!
//! PoisonMoth проверяет IsDied до допуска и запоминает последнюю допущенную
//! цель перед контактом. BloodRose не проверяет IsDied; полная область 3×3
//! одинакова на всех уровнях и идёт X→Y,
//! её первая подходящая цель на центральном X при ненулевом Y сохраняется
//! для visual до дедупликации. Повторная цель подавляет только сам контакт,
//! но не успешный результат клетки. Списки и visual остаются в живом навыке.
//!
//! PK-снимок предшествует свежему Calculate. NULL таблица оставляет UNKNOWN/1,
//! не отменяя raw OnBeenAttacked. Общий оружейный хвост читает MIN→MAX→RNG
//! с сырой DWORD-шириной, снова MIN, затем ELEMENT/SOUL/CCH и второй RNG.
//! CF меняет знак hit; D0 читает добавку 20013 после физического урона и перед
//! ELEMENT. Контакт не начисляет RP, не наносит яд и не изнашивает оружие.

use super::bloodrose::{BLOOD_ROSE_SKILL_ID, BloodRoseExecutionState};
use super::flash::cell_views;
use super::poisonmoth::{POISON_MOTH_SKILL_ID, PoisonMothExecutionState};
use super::weaponattack::{
    PlayerWeaponRoll, fill_ordinary_weapon_damage_with_element_addition, source_master,
};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const USER_HIT_MODIFIER: u32 = 20_001;
const ADDITION_ELEMENT_ATTACK: u32 = 20_013;

fn calculate_crossbow_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let poison_moth = skill.id() == POISON_MOTH_SKILL_ID;
    attack.skill_id = skill.id();
    attack.skill_level = skill.level() as u8;
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let weapon_factor = if source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else { return; };
        let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
        player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum_factor)
    } else {
        if resolve_state_move_shape(game, source.0, source.1).is_none() { return; }
        1.0
    };
    let factor = properties.query_property(TARGET_DAMAGE_FACTOR);
    attack.damage_factor = (f64::from(factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32;
    let hit = properties.query_property(USER_HIT_MODIFIER) as i32;
    attack.hit_modifier = if poison_moth { hit.wrapping_neg() } else { hit };
    fill_ordinary_weapon_damage_with_element_addition(
        game, source, PlayerWeaponRoll::RawRange,
        || if poison_moth { 0 } else { properties.query_property(ADDITION_ELEMENT_ATTACK) }, attack,
    );
}

pub(super) fn apply_crossbow_skill_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let targets_self = std::ptr::eq(user, sufferer);
    let Some(skill) = game.registered_skill_mut(instance) else { return; };
    if skill.id() == BLOOD_ROSE_SKILL_ID {
        if targets_self { return; }
        let Some(state) = skill.player_state_mut::<BloodRoseExecutionState>() else { return; };
        if !state.mark_target_attacked(target) { return; }
    }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let mut attack = AttackInformation::for_master(master);
    calculate_crossbow_attack(game, instance, source, target, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

pub(super) fn run_poison_moth_cell<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    cell: (i32, i32), runtime: &mut Runtime,
) -> bool {
    if cell == (0, 0) { return false; }
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
    if !user.shape().is_assigned_to_server_region() { return false; }
    let region = user.shape().get_region_id();
    let user_identity = user.shape().identity();
    let mut attacked = false;
    for view in cell_views(game, region, cell.0, cell.1) {
        if view.identity == user_identity { continue; }
        let Some(target) = resolve_state_move_shape(game, region, view.identity) else { continue; };
        let target = (target.shape().get_region_id(), target.shape().identity());
        if game.move_shape_health(target.0, target.1).is_none_or(|health| health == 0)
            || !game.live_skill_target_attackable(region, source.1, target.1)
        { continue; }
        let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
        else { return attacked; };
        state.set_visual_target(target.1);
        apply_crossbow_skill_attack(game, instance, source, target, runtime);
        attacked = true;
    }
    attacked
}

pub(super) fn run_blood_rose_scope<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    center: (i32, i32), runtime: &mut Runtime,
) -> bool {
    if center == (0, 0) { return false; }
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
    if !user.shape().is_assigned_to_server_region() { return false; }
    let region = user.shape().get_region_id();
    let user_identity = user.shape().identity();
    let mut attacked = false;
    for offset_x in -1..=1 {
        let x = center.0.wrapping_add(offset_x);
        for offset_y in -1..=1 {
            let y = center.1.wrapping_add(offset_y);
            for view in cell_views(game, region, x, y) {
                if view.identity == user_identity { continue; }
                let Some(target) = resolve_state_move_shape(game, region, view.identity) else { continue; };
                let target = (target.shape().get_region_id(), target.shape().identity());
                if !game.live_skill_target_attackable(region, source.1, target.1) { continue; }
                if x == center.0 && y != 0 {
                    let Some(state) = game.registered_skill_mut(instance)
                        .and_then(|skill| skill.player_state_mut::<BloodRoseExecutionState>())
                    else { return attacked; };
                    state.select_visual_target_if_empty(target.1);
                }
                apply_crossbow_skill_attack(game, instance, source, target, runtime);
                attacked = true;
            }
        }
    }
    attacked
}
