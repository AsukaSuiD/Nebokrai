//! Оружейный roll семейства CalculateAttackPower: три живых вида ширины,
//! компоненты физического, элементного и душевного урона, живые property
//! источника по типу владельца и общий критический хвост. Тела разбросаны по
//! `appserver/skills/*.cpp` конкретных навыков (chuckstone, yakshaslash,
//! heartlessarrow и др.); сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb`. Старый серверный пакет разрешает CGame/find_player/таблицы
//! и передаёт живые поля обратным вызовом `WeaponDamageLiveField`; RNG-состояние
//! и `random(int)` остаются у владельца.
//!
//! Инварианты: критический хвост применяет множитель только к видам 1/3/4
//! (машинный фильтр, симметричный `skills/projectile.rs::apply_projectile_critical`);
//! отношение `+1` к ширине есть у RawRange и обоих abs-видов и нет у Archery
//! (живой Archery-roll — в `skills/projectile.rs`). `source_property` по типу
//! владельца — VERIFIED_DISASSEMBLY: getter-ы CPlayer — прямые чтения полей
//! (MIN/MAX/Element — DWORD, CCH/Soul — WORD), нулевые getter-ы
//! NPC/Build/CityGate — `xor eax,eax`/`xor ax,ax`; Element и CCH монстра
//! всегда 0. RNG при неположительной
//! ширине принадлежит владельцу (`game_legacy_random` возвращает 0 без расхода
//! состояния); формула передаёт ширину дословно.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#боевые-формулы

use super::{
    AttackInformation, AttackPower, AttackPowerType, truncate_original, truncate_original_i64_low,
};

/// Живые виды оружейного roll. Порядок чтений каждого вида и quirk-ширины —
/// в шапке модуля; мёртвый `Archery` прежнего пакета здесь не представлен
/// (живой roll — `skills/projectile`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerWeaponRoll {
    AbsoluteRange,
    RawRange,
    CapturedMinimumAbsoluteRange,
}

/// Усиление фронтальных ударов: ловкость источника и/или расходованный
/// множитель EnergyHolding; вычисление самого множителя остаётся у владельца.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WeaponPowerBoost {
    None,
    Dexterity,
    EnergyHolding(f64),
}

/// Живые поля источника и RNG оружейного roll. Разрешает их старый владелец
/// в порядке исходного CalculateAttackPower; сужения WORD у Soul и CCH и
/// типовой разворот Dexterity принадлежат формуле/подстановке этой схемы,
/// как у `skills/projectile.rs`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeaponDamageLiveField {
    RandomBelow(i32),
    MinimumAttack,
    MaximumAttack,
    AddElementAttack,
    AddSoulAttack,
    CriticalChance,
    Dexterity,
}

/// Живое property источника оружейного расчёта по типу владельца.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeaponSourceProperty {
    Minimum,
    Maximum,
    Element,
    Soul,
    CriticalChance,
}

/// Типизированный снимок боевых property источника. Для монстра MIN/MAX
/// приходят уже после `monsterformula::state_attack_bounds`, а SOUL — после
/// `monsterformula::soul_attack`; нулевые Element/CCH монстра и нули
/// NPC/Build/CityGate разрешает правило `weapon_source_property` без снимка.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WeaponSourceCombat {
    pub minimum_attack: u32,
    pub maximum_attack: u32,
    pub add_element_attack: u32,
    pub add_soul_attack: u16,
    pub critical_chance: u16,
}

/// Исходное правило `source_property`: боевые поля игрока, state-шкалы и
/// нулевые Element/CCH монстра, нули NPC/Build/CityGate. Отсутствующий
/// снимок отменяет чтение, как исходный NULL найденного объекта.
pub fn weapon_source_property(
    owner_type: i32, property: WeaponSourceProperty, combat: Option<WeaponSourceCombat>,
) -> Option<u32> {
    match owner_type {
        400 => combat.map(|combat| match property {
            WeaponSourceProperty::Minimum => combat.minimum_attack,
            WeaponSourceProperty::Maximum => combat.maximum_attack,
            WeaponSourceProperty::Element => combat.add_element_attack,
            WeaponSourceProperty::Soul => u32::from(combat.add_soul_attack),
            WeaponSourceProperty::CriticalChance => u32::from(combat.critical_chance),
        }),
        600 => match property {
            WeaponSourceProperty::Element | WeaponSourceProperty::CriticalChance => Some(0),
            WeaponSourceProperty::Minimum => combat.map(|combat| combat.minimum_attack),
            WeaponSourceProperty::Maximum => combat.map(|combat| combat.maximum_attack),
            WeaponSourceProperty::Soul => combat.map(|combat| u32::from(combat.add_soul_attack)),
        },
        500 | 1_100 | 1_200 => Some(0),
        _ => None,
    }
}

/// Общий критический хвост: после живого шанса и RNG(100) ставится флаг,
/// а компоненты физического, элементного и душевного видов (исходная
/// фильтрация 1/3/4, VERIFIED_DISASSEMBLY — см. шапку) масштабируются
/// float-множителем с усечением FISTP. Та же машинная форма, что и у
/// `skills/projectile.rs::apply_projectile_critical`.
pub fn apply_weapon_critical(
    attack: &mut AttackInformation, chance: i32, roll: i32, rate: f32,
) {
    if roll >= chance { return; }
    attack.critical = true;
    for power in &mut attack.damages {
        if matches!(power.kind,
            AttackPowerType::Physical | AttackPowerType::Element | AttackPowerType::Soul)
        {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

/// Тело CalculateAttackPower семейства: roll по виду `PlayerWeaponRoll`,
/// фронтальная ловкость и i64-усечённый множитель EnergyHolding, компоненты
/// Physical/Element/Soul с живыми чтениями и общий критический хвост.
/// Отсутствие живого поля обрывает расчёт в исходной точке (компоненты и
/// записи до неё сохраняются), RNG владельца вызывается строго в исходном
/// порядке; `captured_critical_chance` заменяет живое чтение CCH знаковым
/// значением конструктора (см. шапку про HeartLessArrowPhalanx2).
pub fn fill_weapon_damage(
    roll: PlayerWeaponRoll, boost: WeaponPowerBoost,
    captured_critical_chance: Option<i32>, critical_rate: f32,
    element_addition: impl FnOnce() -> u32,
    mut read_live: impl FnMut(WeaponDamageLiveField) -> Option<i32>,
    attack: &mut AttackInformation,
) {
    let scale = |damage: i32| match boost {
        WeaponPowerBoost::EnergyHolding(multiplier) => {
            truncate_original_i64_low(f64::from(damage) * multiplier)
        }
        _ => damage,
    };
    let (width, captured_minimum) = match roll {
        PlayerWeaponRoll::AbsoluteRange => {
            let Some(maximum) = read_live(WeaponDamageLiveField::MaximumAttack) else { return; };
            let Some(minimum) = read_live(WeaponDamageLiveField::MinimumAttack) else { return; };
            (maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1), None)
        }
        PlayerWeaponRoll::RawRange => {
            let Some(minimum) = read_live(WeaponDamageLiveField::MinimumAttack) else { return; };
            let Some(maximum) = read_live(WeaponDamageLiveField::MaximumAttack) else { return; };
            (maximum.wrapping_sub(minimum).wrapping_add(1), None)
        }
        PlayerWeaponRoll::CapturedMinimumAbsoluteRange => {
            let Some(minimum) = read_live(WeaponDamageLiveField::MinimumAttack) else { return; };
            let Some(maximum) = read_live(WeaponDamageLiveField::MaximumAttack) else { return; };
            (maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1), Some(minimum))
        }
    };
    let Some(random) = read_live(WeaponDamageLiveField::RandomBelow(width)) else { return; };
    let Some(minimum) =
        captured_minimum.or_else(|| read_live(WeaponDamageLiveField::MinimumAttack))
    else { return; };
    let mut physical = minimum.wrapping_add(random);
    if !matches!(boost, WeaponPowerBoost::None) {
        let Some(dexterity) = read_live(WeaponDamageLiveField::Dexterity) else { return; };
        physical = physical.wrapping_add(dexterity);
    }
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Physical, hp_damage: scale(physical.max(0)), mp_damage: 0,
    });
    let element_addition = element_addition();
    let Some(element) = read_live(WeaponDamageLiveField::AddElementAttack) else { return; };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Element,
        hp_damage: scale(element.wrapping_add(element_addition as i32).max(0)),
        mp_damage: 0,
    });
    let Some(soul) = read_live(WeaponDamageLiveField::AddSoulAttack) else { return; };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Soul, hp_damage: scale(i32::from(soul as u16)), mp_damage: 0,
    });
    let critical_chance = match captured_critical_chance {
        Some(chance) => chance,
        None => {
            let Some(chance) = read_live(WeaponDamageLiveField::CriticalChance) else { return; };
            i32::from(chance as u16)
        }
    };
    let Some(critical_roll) = read_live(WeaponDamageLiveField::RandomBelow(100)) else { return; };
    apply_weapon_critical(attack, critical_chance, critical_roll, critical_rate);
}
