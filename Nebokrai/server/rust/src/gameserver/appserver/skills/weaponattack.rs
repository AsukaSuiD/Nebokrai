//! Общий оружейный расчёт обычных, фронтальных и усиленных мечевых ударов.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые владельцы skills.
//!
//! Расчёт сохраняет PK-флаги и принадлежность CPlayer до Calculate. Обычный
//! контакт доставляет сырой OnBeenAttacked без повторного допуска, затем
//! вызывает IncreaseRp независимо от результата. NULL таблица Calculate оставляет
//! исходный UNKNOWN/1, но не отменяет удар. Допуск и дедупликация принадлежат AI.
//! GhostCut читает MIN→MAX и передаёт RNG сырую DWORD-ширину max-min+1;
//! остальные семейства читают MAX→MIN и используют abs(max-min)+1. Обе ветки
//! снова читают MIN после RNG. Mosou оставляет единичный коэффициент, не читая
//! уровень цели и модификатор оружия. Фронтальные удары добавляют живую
//! ловкость CPlayer после второго MIN. InverseChopped после hit modifier
//! расходует первое EnergyHolding и умножает три компонента с усечением
//! к младшему DWORD от i64; его контакт не начисляет RP.
//! Сохранены живые getter-ы CMoveShape, оба RNG,
//! unsigned коэффициент в x87 до записи float и усечение критического
//! множителя к нулю. У CMonster используются его MIN/MAX/SOUL; ELEMENT после
//! native нижней границы и CCH равны нулю. NPC, Build и CityGate наследуют
//! нулевые компоненты/CCH, единичный weapon modifier и пустой IncreaseRp.
//! Vec владеет уроном.

use super::energyholdingstate::consume_energy_holding_multiplier;
use super::fightdefense::truncate_original;
use super::flash::master_info;
use super::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const USER_HIT_MODIFIER: u32 = 20_001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PlayerWeaponRoll {
    AbsoluteRange,
    RawRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlayerWeaponDamageFactor {
    Unit,
    WeaponUsage(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WeaponPowerMode {
    Ordinary,
    Dexterity,
    EnergyHolding,
}

#[derive(Clone, Copy)]
enum SourceProperty {
    Minimum,
    Maximum,
    Element,
    Soul,
    CriticalChance,
}

fn source_property(game: &CGame, source: (i32, ShapeIdentity), property: SourceProperty) -> Option<u32> {
    resolve_state_move_shape(game, source.0, source.1)?;
    match source.1.object_type {
        400 => {
            let combat = game.find_player(source.1.id)?.combat_properties();
            Some(match property {
                SourceProperty::Minimum => combat.minimum_attack,
                SourceProperty::Maximum => combat.maximum_attack,
                SourceProperty::Element => combat.add_element_attack,
                SourceProperty::Soul => u32::from(combat.add_soul_attack),
                SourceProperty::CriticalChance => u32::from(combat.cch),
            })
        }
        600 => {
            // GetAddElementAtk монстра умножает pet factor на ноль. Даже
            // нечисловой factor после native FISTP и нижней границы даёт 0.
            if matches!(property, SourceProperty::Element | SourceProperty::CriticalChance) { return Some(0); }
            let monster = game.find_region(source.0)?.base().find_monster_by_id(source.1.id)?;
            let resource = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some(match property {
                SourceProperty::Minimum => monster.state_attack_bounds(resource.minimum_attack, resource.maximum_attack).0,
                SourceProperty::Maximum => monster.state_attack_bounds(resource.minimum_attack, resource.maximum_attack).1,
                SourceProperty::Soul => u32::from(monster.soul_attack(resource)),
                SourceProperty::Element | SourceProperty::CriticalChance => 0,
            })
        }
        500 | 1_100 | 1_200 => Some(0),
        _ => None,
    }
}

fn source_master(game: &CGame, source: (i32, ShapeIdentity)) -> Option<MasterInfo> {
    let identity = resolve_state_move_shape(game, source.0, source.1)?.shape().identity();
    if identity.object_type == 400 { return game.find_player(identity.id).map(master_info); }
    Some(MasterInfo { master_type: identity.object_type, master_id: identity.id, ..MasterInfo::default() })
}

fn fill_player_weapon_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), factor: PlayerWeaponDamageFactor, roll: PlayerWeaponRoll,
    power_mode: WeaponPowerMode, attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    attack.skill_id = skill.id();
    attack.skill_level = skill.level() as u8;
    attack.damage_modifier = 0;
    attack.damage_factor = match factor {
        PlayerWeaponDamageFactor::Unit => 1.0,
        PlayerWeaponDamageFactor::WeaponUsage(usage) => {
            let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
            let weapon_factor = if source.1.object_type == 400 {
                let Some(player) = game.find_player(source.1.id) else { return; };
                let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
                player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum_factor)
            } else {
                // У остальных CMoveShape унаследован единичный GetWeaponModifier.
                if resolve_state_move_shape(game, source.0, source.1).is_none() { return; }
                1.0
            };
            let damage_factor = properties.query_property(usage);
            (f64::from(damage_factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32
        }
    };
    attack.hit_modifier = properties.query_property(USER_HIT_MODIFIER) as i32;
    let multiplier = if power_mode == WeaponPowerMode::EnergyHolding {
        consume_energy_holding_multiplier(game, source)
    } else { 1.0 };
    let scale = |damage: i32| {
        if power_mode == WeaponPowerMode::EnergyHolding {
            truncate_original_i64_low(f64::from(damage) * multiplier)
        } else { damage }
    };
    let width = match roll {
        PlayerWeaponRoll::AbsoluteRange => {
            let Some(maximum) = source_property(game, source, SourceProperty::Maximum) else { return; };
            let Some(minimum) = source_property(game, source, SourceProperty::Minimum) else { return; };
            (maximum as i32).wrapping_sub(minimum as i32).wrapping_abs().wrapping_add(1)
        }
        PlayerWeaponRoll::RawRange => {
            let Some(minimum) = source_property(game, source, SourceProperty::Minimum) else { return; };
            let Some(maximum) = source_property(game, source, SourceProperty::Maximum) else { return; };
            (maximum as i32).wrapping_sub(minimum as i32).wrapping_add(1)
        }
    };
    let random = game.skill_random_below(width);
    let Some(minimum) = source_property(game, source, SourceProperty::Minimum) else { return; };
    let mut physical = (minimum as i32).wrapping_add(random);
    if power_mode != WeaponPowerMode::Ordinary && source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else { return; };
        physical = physical.wrapping_add(player.combat_properties().dexterity as i32);
    }
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: scale(physical.max(0)), mp_damage: 0 });
    let Some(element) = source_property(game, source, SourceProperty::Element) else { return; };
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: scale((element as i32).max(0)), mp_damage: 0 });
    let Some(soul) = source_property(game, source, SourceProperty::Soul) else { return; };
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: scale(i32::from(soul as u16)), mp_damage: 0 });
    let Some(critical_chance) = source_property(game, source, SourceProperty::CriticalChance) else { return; };
    if game.skill_random_below(100) < i32::from(critical_chance as u16) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

pub(super) fn calculate_player_weapon_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), damage_factor_usage: u32, roll: PlayerWeaponRoll,
) -> Option<(MasterInfo, AttackInformation)> {
    calculate_player_weapon_attack_with_factor(
        game, instance, source, target, PlayerWeaponDamageFactor::WeaponUsage(damage_factor_usage), roll,
        WeaponPowerMode::Ordinary,
    )
}

fn calculate_player_weapon_attack_with_factor(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), factor: PlayerWeaponDamageFactor, roll: PlayerWeaponRoll,
    power_mode: WeaponPowerMode,
) -> Option<(MasterInfo, AttackInformation)> {
    let master = source_master(game, source)?;
    let mut attack = AttackInformation::for_master(master);
    fill_player_weapon_attack(game, instance, source, target, factor, roll, power_mode, &mut attack);
    Some((master, attack))
}

pub(super) fn apply_player_weapon_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), damage_factor_usage: u32, runtime: &mut Runtime,
) {
    apply_player_weapon_attack_with_factor(
        game, instance, source, target, PlayerWeaponDamageFactor::WeaponUsage(damage_factor_usage), runtime,
    );
}

pub(super) fn apply_player_unmodified_weapon_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    apply_player_weapon_attack_with_factor(
        game, instance, source, target, PlayerWeaponDamageFactor::Unit, runtime,
    );
}

fn apply_player_weapon_attack_with_factor<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), factor: PlayerWeaponDamageFactor, runtime: &mut Runtime,
) {
    let Some((master, attack)) = calculate_player_weapon_attack_with_factor(
        game, instance, source, target, factor, PlayerWeaponRoll::AbsoluteRange, WeaponPowerMode::Ordinary,
    ) else { return; };
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    if source.1.object_type == 400 { game.increase_owned_player_rp(source.1.id, true, 0); }
}

pub(super) fn apply_front_cell_weapon_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), inverse: bool, runtime: &mut Runtime,
) {
    let mode = if inverse { WeaponPowerMode::EnergyHolding } else { WeaponPowerMode::Dexterity };
    let Some((master, attack)) = calculate_player_weapon_attack_with_factor(
        game, instance, source, target, PlayerWeaponDamageFactor::WeaponUsage(20_003),
        PlayerWeaponRoll::AbsoluteRange, mode,
    ) else { return; };
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    if !inverse && source.1.object_type == 400 { game.increase_owned_player_rp(source.1.id, true, 0); }
}
