//! Прямое элементальное попадание Lightning и ChainLightning.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/lightning.cpp
//! и chainlightning.cpp, Attack/CalculateAttackPower.
//! Допуск, смерть цели и список повторных попаданий принадлежат AI/Attack
//! владельца. Здесь сохраняются PK источника, свежая таблица Calculate и сырой
//! OnBeenAttacked; отсутствие таблицы оставляет исходный UNKNOWN/1 и пустой урон.
//! ChainLightning читает уровень цели и оружейный множитель, а после контакта
//! начисляет RP источнику; Lightning сохраняет единичный множитель без этих чтений.
//! EM игрока захватывается до таблицы. Его масштабирование и усечение выполнены
//! до MAX→MIN→RNG→свежего MIN→живого AddElement, затем CCH→RNG100.
//! Расширенное вычисление EM и критического множителя
//! усекается к нулю; промежуточного округления EM к f32 нет. Vec владеет уроном.

use super::chainlightning::CHAIN_LIGHTNING_SKILL_ID;
use super::fightdefense::truncate_original;
use super::weaponattack::{SourceProperty, apply_weapon_critical, source_master, source_property};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

fn calculate(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), attack: &mut AttackInformation,
) {
    let element_modify = if source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else { return; };
        player.combat_properties().element_modify
    } else { 0 };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    attack.skill_id = skill.id();
    attack.skill_level = skill.level() as u8;
    attack.damage_modifier = properties.query_property(20_002) as i32;
    if skill.id() == CHAIN_LIGHTNING_SKILL_ID {
        let Some(level) = game.move_shape_level(target.0, target.1) else { return; };
        attack.damage_factor = if source.1.object_type == 400 {
            let Some(player) = game.find_player(source.1.id) else { return; };
            let (divisor, minimum) = game.globe_setup().weapon_damage_factors();
            player.weapon_modifier(game.goods_factory(), i32::from(level), divisor, minimum)
        } else { 1.0 };
    }
    attack.hit_modifier = properties.query_property(20_001) as i32;
    let modifier = properties.query_property(20_015);
    let bonus = truncate_original(f64::from(modifier) * f64::from(0.01_f32) * f64::from(element_modify));
    let maximum = properties.query_property(20_009) as i32;
    let minimum = properties.query_property(20_008) as i32;
    let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
    let random = game.skill_random_below(width);
    let minimum = properties.query_property(20_008) as i32;
    let Some(element) = source_property(game, source, SourceProperty::Element) else { return; };
    let damage = (element as i32).wrapping_add(random).wrapping_add(minimum).wrapping_add(bonus).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 });
    let Some(chance) = source_property(game, source, SourceProperty::CriticalChance) else { return; };
    apply_weapon_critical(game, i32::from(chance as u16), attack);
}

pub(super) fn apply_direct_element_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(source_shape) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let Some(target_shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    if std::ptr::eq(source_shape, target_shape) { return; }
    let Some(master) = source_master(game, source) else { return; };
    let chain = game.registered_skill(instance).is_some_and(|skill| skill.id() == CHAIN_LIGHTNING_SKILL_ID);
    let mut attack = AttackInformation::for_master(master);
    calculate(game, instance, source, target, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    if chain && source.1.object_type == 400 { game.increase_owned_player_rp(source.1.id, true, 0); }
}
