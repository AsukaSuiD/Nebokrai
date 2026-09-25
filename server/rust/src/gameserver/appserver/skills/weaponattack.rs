//! Общий оружейный расчёт обычных, фронтальных и усиленных мечевых ударов.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые владельцы skills.
//!
//! Формульная база (виды roll, порядок живых чтений, quirk-ширины,
//! компоненты, критический хвост и выбор property источника по типу
//! владельца) перенесена в `nebokrai_zone::combat::weaponattack`; основание
//! и машинные статусы см. там. Сырая ветвь `PlayerWeaponRoll::Archery`
//! прежнего файла не имела вызывающих (живой Archery-roll — в
//! `nebokrai_zone::skills::projectile`) и удалена. Здесь остаются
//! CGame-разрешение живых полей и владельцы стадий навыков.
//!
//! Расчёт сохраняет PK-флаги и принадлежность CPlayer до Calculate. Обычный
//! контакт доставляет сырой OnBeenAttacked без повторного допуска, затем
//! вызывает IncreaseRp независимо от результата. NULL таблица Calculate оставляет
//! исходный UNKNOWN/1, но не отменяет удар. Допуск и дедупликация принадлежат AI.
//! Mosou оставляет единичный коэффициент, не читая уровень цели и модификатор
//! оружия. Фронтальные удары добавляют живую ловкость CPlayer после второго
//! MIN, а InverseChopped после hit modifier расходует первое EnergyHolding и
//! передаёт его множитель формуле Zone; оба вида различаются начислением RP.
//! Сохранены unsigned коэффициент в x87 до записи float и единичный weapon
//! modifier для монстра/NPC/Build/CityGate с пустым IncreaseRp.
//! LightingArrowPhalanx использует тот же порядок компонентов и RNG без
//! ловкости; её сохранённый знаковый коэффициент и яд остаются у формы.
//! PoisonMoth использует сырую ширину, BloodRose дополнительно читает свою
//! добавку после физического урона, непосредственно перед живым ELEMENT —
//! добавка вычисляется колбэком в момент, заданный формулой Zone.
//! Первые два удара Scorpion сохраняют живой weapon modifier без запроса
//! skill factor; третий использует обычный WeaponUsage. RP остаётся у caller-а.
//! Strike меняет знак hit modifier сразу после запроса, до компонентов и RNG.
//! HeartLessArrowPhalanx сохраняет MIN до RNG и использует знаковый CCH
//! конструктора вместо повторного чтения текущего критического шанса.
//! BaseMagicPhalanx использует только общий критический хвост: её сохранённый
//! диапазон и единственный элементный компонент рассчитывает сам владелец.
//! ChuckStone/SkeletonArchery (appserver/skills/chuckstone.cpp и
//! skeletonarchery.cpp) используют RawRange без оружейного множителя и RP.
//! Их Calculate не меняет исходные UNKNOWN/1 даже при найденной таблице;
//! перед тремя компонентами записываются только нулевой damage modifier и hit.

use super::energyholdingstate::consume_energy_holding_multiplier;
use super::flash::master_info;
use nebokrai_zone::combat::{
    WeaponDamageLiveField, WeaponPowerBoost, WeaponSourceCombat, weapon_source_property,
};
pub(super) use nebokrai_zone::combat::{PlayerWeaponRoll, WeaponSourceProperty as SourceProperty};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const USER_HIT_MODIFIER: u32 = 20_001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlayerWeaponDamageFactor {
    Unit,
    WeaponOnly,
    WeaponUsage(u32),
}

#[derive(Clone, Copy)]
enum WeaponHitSign { Bonus, Penalty }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WeaponPowerMode {
    Ordinary,
    Dexterity,
    EnergyHolding,
}

pub(super) fn source_property(game: &CGame, source: (i32, ShapeIdentity), property: SourceProperty) -> Option<u32> {
    resolve_state_move_shape(game, source.0, source.1)?;
    match source.1.object_type {
        400 => {
            let combat = game.find_player(source.1.id)?.combat_properties();
            weapon_source_property(400, property, Some(WeaponSourceCombat {
                minimum_attack: combat.minimum_attack,
                maximum_attack: combat.maximum_attack,
                add_element_attack: combat.add_element_attack,
                add_soul_attack: combat.add_soul_attack,
                critical_chance: combat.cch,
            }))
        }
        600 => {
            // GetAddElementAtk монстра умножает pet factor на ноль. Даже
            // нечисловой factor после native FISTP и нижней границы даёт 0.
            if matches!(property, SourceProperty::Element | SourceProperty::CriticalChance) {
                return weapon_source_property(600, property, None);
            }
            let monster = game.find_region(source.0)?.base().find_monster_by_id(source.1.id)?;
            let resource = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            let (minimum_attack, maximum_attack) =
                monster.state_attack_bounds(resource.minimum_attack, resource.maximum_attack);
            weapon_source_property(600, property, Some(WeaponSourceCombat {
                minimum_attack,
                maximum_attack,
                add_element_attack: 0,
                add_soul_attack: monster.soul_attack(resource),
                critical_chance: 0,
            }))
        }
        object_type => weapon_source_property(object_type, property, None),
    }
}

pub(super) fn source_master(game: &CGame, source: (i32, ShapeIdentity)) -> Option<MasterInfo> {
    let identity = resolve_state_move_shape(game, source.0, source.1)?.shape().identity();
    if identity.object_type == 400 { return game.find_player(identity.id).map(master_info); }
    Some(MasterInfo { master_type: identity.object_type, master_id: identity.id, ..MasterInfo::default() })
}

fn fill_player_weapon_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), factor: PlayerWeaponDamageFactor, roll: PlayerWeaponRoll,
    power_mode: WeaponPowerMode, hit_sign: WeaponHitSign, attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    attack.skill_id = skill.id();
    attack.skill_level = skill.level() as u8;
    attack.damage_modifier = 0;
    attack.damage_factor = match factor {
        PlayerWeaponDamageFactor::Unit => 1.0,
        PlayerWeaponDamageFactor::WeaponOnly | PlayerWeaponDamageFactor::WeaponUsage(_) => {
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
            match factor {
                PlayerWeaponDamageFactor::WeaponUsage(usage) => {
                    let damage_factor = properties.query_property(usage);
                    (f64::from(damage_factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32
                }
                _ => weapon_factor,
            }
        }
    };
    attack.hit_modifier = properties.query_property(USER_HIT_MODIFIER) as i32;
    if matches!(hit_sign, WeaponHitSign::Penalty) {
        attack.hit_modifier = attack.hit_modifier.wrapping_neg();
    }
    let boost = match power_mode {
        WeaponPowerMode::Ordinary => WeaponPowerBoost::None,
        WeaponPowerMode::Dexterity => WeaponPowerBoost::Dexterity,
        WeaponPowerMode::EnergyHolding => {
            WeaponPowerBoost::EnergyHolding(consume_energy_holding_multiplier(game, source))
        }
    };
    fill_weapon_damage(game, source, roll, boost, None, || 0, attack);
}

pub(super) fn fill_ordinary_weapon_damage(
    game: &mut CGame, source: (i32, ShapeIdentity), roll: PlayerWeaponRoll,
    attack: &mut AttackInformation,
) {
    fill_ordinary_weapon_damage_with_element_addition(game, source, roll, || 0, attack);
}

pub(super) fn fill_ordinary_weapon_damage_with_element_addition(
    game: &mut CGame, source: (i32, ShapeIdentity), roll: PlayerWeaponRoll,
    element_addition: impl FnOnce() -> u32, attack: &mut AttackInformation,
) {
    fill_weapon_damage(game, source, roll, WeaponPowerBoost::None, None, element_addition, attack);
}

pub(super) fn fill_captured_weapon_damage(
    game: &mut CGame, source: (i32, ShapeIdentity), critical_chance: i32,
    attack: &mut AttackInformation,
) {
    fill_weapon_damage(
        game, source, PlayerWeaponRoll::CapturedMinimumAbsoluteRange,
        WeaponPowerBoost::None, Some(critical_chance), || 0, attack,
    );
}

fn fill_weapon_damage(
    game: &mut CGame, source: (i32, ShapeIdentity), roll: PlayerWeaponRoll,
    boost: WeaponPowerBoost, captured_critical_chance: Option<i32>,
    element_addition: impl FnOnce() -> u32, attack: &mut AttackInformation,
) {
    let critical_rate = game.globe_setup().critical_rate();
    nebokrai_zone::combat::fill_weapon_damage(
        roll, boost, captured_critical_chance, critical_rate, element_addition,
        |field| match field {
            WeaponDamageLiveField::RandomBelow(bound) => Some(game.skill_random_below(bound)),
            WeaponDamageLiveField::MinimumAttack => {
                source_property(game, source, SourceProperty::Minimum).map(|value| value as i32)
            }
            WeaponDamageLiveField::MaximumAttack => {
                source_property(game, source, SourceProperty::Maximum).map(|value| value as i32)
            }
            WeaponDamageLiveField::AddElementAttack => {
                source_property(game, source, SourceProperty::Element).map(|value| value as i32)
            }
            WeaponDamageLiveField::AddSoulAttack => {
                source_property(game, source, SourceProperty::Soul).map(|value| value as i32)
            }
            WeaponDamageLiveField::CriticalChance => {
                source_property(game, source, SourceProperty::CriticalChance)
                    .map(|value| value as i32)
            }
            WeaponDamageLiveField::Dexterity => {
                if source.1.object_type == 400 {
                    game.find_player(source.1.id)
                        .map(|player| player.combat_properties().dexterity as i32)
                } else {
                    Some(0)
                }
            }
        },
        attack,
    );
}

pub(super) fn apply_weapon_critical(
    game: &mut CGame, critical_chance: i32, attack: &mut AttackInformation,
) {
    let rate = game.globe_setup().critical_rate();
    let roll = game.skill_random_below(100);
    nebokrai_zone::combat::apply_weapon_critical(attack, critical_chance, roll, rate);
}

pub(super) fn apply_direct_projectile_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    if std::ptr::eq(user, sufferer) { return; }
    let Some(master) = source_master(game, source) else { return; };
    let mut attack = AttackInformation::for_master(master);
    if let Some(properties) = game.registered_skill(instance)
        .and_then(|skill| game.skill_base_properties(skill.id(), skill.level()))
    {
        attack.damage_modifier = 0;
        attack.hit_modifier = properties.query_property(USER_HIT_MODIFIER) as i32;
        fill_ordinary_weapon_damage(game, source, PlayerWeaponRoll::RawRange, &mut attack);
    }
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

pub(super) fn calculate_player_weapon_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), damage_factor_usage: u32, roll: PlayerWeaponRoll,
) -> Option<(MasterInfo, AttackInformation)> {
    calculate_player_weapon_attack_with_factor(
        game, instance, source, target, PlayerWeaponDamageFactor::WeaponUsage(damage_factor_usage), roll,
        WeaponPowerMode::Ordinary, WeaponHitSign::Bonus,
    )
}

pub(super) fn calculate_unscaled_weapon_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), roll: PlayerWeaponRoll,
) -> Option<(MasterInfo, AttackInformation)> {
    calculate_player_weapon_attack_with_factor(
        game, instance, source, target, PlayerWeaponDamageFactor::WeaponOnly, roll,
        WeaponPowerMode::Ordinary, WeaponHitSign::Bonus,
    )
}

pub(super) fn calculate_penalized_weapon_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), damage_factor_usage: u32, roll: PlayerWeaponRoll,
) -> Option<(MasterInfo, AttackInformation)> {
    calculate_player_weapon_attack_with_factor(
        game, instance, source, target, PlayerWeaponDamageFactor::WeaponUsage(damage_factor_usage), roll,
        WeaponPowerMode::Ordinary, WeaponHitSign::Penalty,
    )
}

fn calculate_player_weapon_attack_with_factor(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), factor: PlayerWeaponDamageFactor, roll: PlayerWeaponRoll,
    power_mode: WeaponPowerMode, hit_sign: WeaponHitSign,
) -> Option<(MasterInfo, AttackInformation)> {
    let master = source_master(game, source)?;
    let mut attack = AttackInformation::for_master(master);
    fill_player_weapon_attack(game, instance, source, target, factor, roll, power_mode, hit_sign, &mut attack);
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
        WeaponHitSign::Bonus,
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
        PlayerWeaponRoll::AbsoluteRange, mode, WeaponHitSign::Bonus,
    ) else { return; };
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    if !inverse && source.1.object_type == 400 { game.increase_owned_player_rp(source.1.id, true, 0); }
}
