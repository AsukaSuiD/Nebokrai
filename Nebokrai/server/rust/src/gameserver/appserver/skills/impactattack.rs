//! Попадание и отбрасывание Mosou и ThunderBlow2.
//! Источник: gameserver.exe/GameServer.pdb, skills/mosou.cpp и thunderblow2.cpp.
//!
//! Mosou обходит один снимок лицевой клетки без дедупликации; допуск проверяется
//! перед каждым контактом. Оружейный расчёт и RP общие с weaponattack. После
//! попадания только игрок уменьшает вероятность оглушения своим avoidance.
//! Cure запрещает установку KnockOut, но не последующее отбрасывание; отказ
//! Begin нового состояния также не отменяет ForceMove.
//!
//! ThunderBlow2 фиксирует element-modify до свежего Calculate, сохраняет
//! два RNG и x87-усечение, доставляет сырой контакт без начисления RP.
//! NULL таблица Calculate оставляет UNKNOWN/1 и пустой урон. Общая геометрия
//! отбрасывания читает позиции после callback-ов и использует захваченный
//! caller-ом регион для GetBlock, а ForceMove — фактический регион цели.

use super::basemagic::{SKILL_USAGE_ELEMENT_MODIFIER, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK};
use super::cure::CURE_SKILL_ID;
use super::fightdefense::truncate_original;
use super::flash::{cell_views, master_info};
use super::knockoutstate::{KnockOutState, replace_knock_out_state};
use super::skillbaseproperties::CSkillBaseProperties;
use super::weaponattack::apply_player_unmodified_weapon_attack;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::public::tools::get_line_direction;

const USER_HIT_MODIFIER: u32 = 20_001;
const TARGET_FINAL_DAMAGE_MODIFIER: u32 = 20_002;
const BASE_PROBABILITY: u32 = 40_001;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_BACK_STEP: u32 = 1_001;
const TARGET_MOVE_SPEED: u32 = 2_001;

fn fill_thunder_blow_2_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    attack: &mut AttackInformation,
) {
    let element_modify = game.find_player(source.1.id)
        .map_or(0, |player| player.combat_properties().element_modify);
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    attack.skill_id = skill.id();
    attack.skill_level = skill.level() as u8;
    attack.damage_modifier = properties.query_property(TARGET_FINAL_DAMAGE_MODIFIER) as i32;
    attack.hit_modifier = properties.query_property(USER_HIT_MODIFIER) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let random = game.skill_random_below(maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1));
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let Some(player) = game.find_player(source.1.id) else { return; };
    let add_element = player.combat_properties().add_element_attack as i32;
    let element_bonus = truncate_original(
        f64::from(element_modifier) * f64::from(0.01_f32) * f64::from(element_modify),
    );
    let damage = add_element.wrapping_add(random).wrapping_add(minimum).wrapping_add(element_bonus).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 });
    let critical_chance = player.combat_properties().cch;
    if game.skill_random_below(100) < i32::from(critical_chance) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

pub(super) fn apply_thunder_blow_2_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    if source.1 == target.1 || resolve_state_move_shape(game, target.0, target.1).is_none() { return; }
    let Some(master) = game.find_player(source.1.id).map(master_info) else { return; };
    let mut attack = AttackInformation::for_master(master);
    fill_thunder_blow_2_attack(game, instance, source, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

pub(super) fn knock_back_impact_target(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    region_id: i32, properties: &CSkillBaseProperties,
) {
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
    let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
    let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
    let direction = get_line_direction(source_x, source_y, target_x, target_y);
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let mut destination = ShapeAreaCoordinates {
        x: sufferer.shape().get_tile_x().unwrap_or(i32::MIN),
        y: sufferer.shape().get_tile_y().unwrap_or(i32::MIN),
    };
    let mut candidate = ShapeAreaCoordinates {
        x: sufferer.shape().get_tile_x().unwrap_or(i32::MIN),
        y: sufferer.shape().get_tile_y().unwrap_or(i32::MIN),
    };
    let mut moved = 0_u32;
    let mut steps = properties.query_property(TARGET_BACK_STEP);
    while moved < steps {
        let Ok(next) = CShape::get_direction_position(direction, candidate) else { return; };
        candidate = next;
        let Some(owner) = game.find_region(region_id) else { return; };
        if owner.base().block_at(candidate.x, candidate.y) != Some(0) { break; }
        destination = candidate;
        moved = moved.wrapping_add(1);
        steps = properties.query_property(TARGET_BACK_STEP);
    }
    let speed = properties.query_property(TARGET_MOVE_SPEED);
    let _ = game.force_move_skill_target(
        target.0, target.1, destination.x, destination.y, speed.wrapping_mul(moved),
    );
}

pub(super) fn run_mosou_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let shape = user.shape();
    if !shape.is_assigned_to_server_region() { return; }
    let region_id = shape.get_region_id();
    if game.find_region(region_id).is_none() { return; }
    let position = ShapeAreaCoordinates {
        x: shape.get_tile_x().unwrap_or(i32::MIN),
        y: shape.get_tile_y().unwrap_or(i32::MIN),
    };
    let Ok(front) = CShape::get_direction_position(shape.get_direction(), position) else { return; };
    for view in cell_views(game, region_id, front.x, front.y) {
        let Some(sufferer) = resolve_state_move_shape(game, region_id, view.identity) else { continue; };
        let target = (sufferer.shape().get_region_id(), sufferer.shape().identity());
        if source.1 == target.1 || !game.live_skill_target_attackable(target.0, source.1, target.1) { continue; }
        apply_player_unmodified_weapon_attack(game, instance, source, target, runtime);
        let avoidance = if target.1.object_type == 400 {
            game.find_player(target.1.id).map_or(0, |player| i32::from(player.combat_properties().attack_avoid))
        } else { 0 };
        let probability = (properties.query_property(BASE_PROBABILITY) as i32).wrapping_sub(avoidance);
        if game.skill_random_below(100) >= probability { continue; }
        let Some(target_level) = game.move_shape_level(target.0, target.1) else { continue; };
        let Some(source_level) = game.move_shape_level(source.0, source.1) else { continue; };
        if target_level > source_level || game.move_shape_health(target.0, target.1).is_none_or(|health| health == 0) { continue; }
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { continue; };
        if !sufferer.has_state_by_skill_id(CURE_SKILL_ID) {
            let persist = properties.query_property(STATE_PERSIST_TIME);
            let state = KnockOutState::new(0, persist);
            let _ = replace_knock_out_state(game, source, target, state, &mut || runtime.now_milliseconds());
        }
        knock_back_impact_target(game, source, target, region_id, properties);
    }
}
