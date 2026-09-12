//! Общий оружейный контакт Flash/LittleFlash и ArmyBreak.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые владельцы skills.
//!
//! Attack сохраняет PK-флаги и принадлежность CPlayer до Calculate, доставляет
//! сырой OnBeenAttacked без повторного допуска, затем вызывает IncreaseRp
//! независимо от результата получателя. NULL таблица Calculate оставляет
//! исходный UNKNOWN/1, но не отменяет удар. Допуск и дедупликация принадлежат AI.
//! Формула различается только usage коэффициента основной/побочной цели;
//! сохранены живые getter-ы, оба RNG, unsigned коэффициент в x87 до записи
//! float и усечение критического множителя к нулю. Vec владеет записями урона.

use super::fightdefense::truncate_original;
use super::flash::master_info;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const USER_HIT_MODIFIER: u32 = 20_001;

fn calculate_player_weapon_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), damage_factor_usage: u32, attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    attack.skill_id = skill.id();
    attack.skill_level = skill.level() as u8;
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let Some(player) = game.find_player(source.1.id) else { return; };
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    let weapon_factor = player.weapon_modifier(
        game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
    );
    let damage_factor = properties.query_property(damage_factor_usage);
    attack.damage_factor =
        (f64::from(damage_factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32;
    attack.hit_modifier = properties.query_property(USER_HIT_MODIFIER) as i32;
    let maximum = player.combat_properties().maximum_attack;
    let minimum = player.combat_properties().minimum_attack;
    let width = (maximum as i32).wrapping_sub(minimum as i32).wrapping_abs().wrapping_add(1);
    let random = game.skill_random_below(width);
    let Some(player) = game.find_player(source.1.id) else { return; };
    let physical = (player.combat_properties().minimum_attack as i32).wrapping_add(random).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
    let element = (player.combat_properties().add_element_attack as i32).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 });
    let soul = i32::from(player.combat_properties().add_soul_attack);
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: soul, mp_damage: 0 });
    let critical_chance = player.combat_properties().cch;
    if game.skill_random_below(100) < i32::from(critical_chance) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

pub(super) fn apply_player_weapon_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), damage_factor_usage: u32, runtime: &mut Runtime,
) {
    let Some(master) = game.find_player(source.1.id).map(master_info) else { return; };
    let mut attack = AttackInformation::for_master(master);
    calculate_player_weapon_attack(game, instance, source, target, damage_factor_usage, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    game.increase_owned_player_rp(source.1.id, true, 0);
}
