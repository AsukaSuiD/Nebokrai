//! Формулы боевого опыта и property-пакеты `CMonster` Zone. Исходный
//! владелец — `appserver/monster.h/.cpp`; переходный `CMonster` старого
//! пакета делегирует сюда без изменения сигнатур методов, передавая свои
//! hub-поля типизированными параметрами.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Публичные символы семейства: `GetMaxHP` (RVA `0x000E65A0`),
//! `GetAttackAvoid` (`0x000E64F0`), `GetElementAvoid` (`0x000E6520`),
//! `GetHit` (`0x000E6760`), `GetDef` (`0x000E6780`), `GetDodge` (`0x000E6800`),
//! `GetElementResistant` (`0x000E6880`), `GetElementModify` (`0x000E6900`),
//! `GetSoulResistant` (`0x000E6970`), `GetHpRecoverSpeed` (`0x000E6990`),
//! `GetAddSoulAtk` (`0x000E69D0`), `GetAtcInterval` (`0x000E69F0`),
//! `GetStopFrame` (`0x000E6A40`), `GetSpeed` (`0x000E79B0`),
//! `CalculateExperienceQuota` (`0x000E7FE0`),
//! `CalculateExperienceCorrective` (`0x000E7A70`).
//! Pet attack/speed/timing и elemental modifier применяют факторы только при
//! валидной player-owner связи; целочисленные результаты сохраняют x87
//! truncation. Таблица квоты опыта индексируется числом живых участников, а
//! оба результата усекаются после x87-порядка операций; состав живой группы,
//! поэтапные масштабы игрока/региона и выдачу координирует игровой владелец.

use nebokrai_shared::resources::MonsterProperties;

use crate::regions::moveshape::MoveShapePropertyModifiers;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MonsterCombatProperties {
    pub level: u8,
    pub defense: u32,
    pub dodge: u32,
    pub element_resistance: u32,
    pub soul_resistance: u16,
    pub attack_avoid: u16,
    pub element_avoid: u16,
    pub promotion_magic_attack_factor: Option<u16>,
}

/// Параметры точных `CalculateExperienceQuota` и
/// `CalculateExperienceCorrective`; состав группы и поэтапное применение
/// результата остаются у игрового владельца. Вычисления ведутся через `f64`
/// как безопасный адаптер для x87-стека оригинала, а коэффициенты сохраняют
/// исходную `f32` точность.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonsterExperienceFormula {
    ratios: [f32; 8],
    difference: f32,
    limit: f32,
    amerce: f32,
    amerce_limit: f32,
    amerce_start_level: i32,
    hit_base_level: i32,
    hit_prize: f32,
    maximum_hit_prize: f32,
}

impl MonsterExperienceFormula {
    pub const fn new(
        experience: ([f32; 8], f32, f32, f32, f32, i32),
        continuous_kill: (i32, f32, f32),
    ) -> Self {
        Self {
            ratios: experience.0,
            difference: experience.1,
            limit: experience.2,
            amerce: experience.3,
            amerce_limit: experience.4,
            amerce_start_level: experience.5,
            hit_base_level: continuous_kill.0,
            hit_prize: continuous_kill.1,
            maximum_hit_prize: continuous_kill.2,
        }
    }

    pub fn quota(
        self,
        property: &MonsterProperties,
        team_id: i32,
        average_level: f32,
        alive_amount: u32,
        player_level: u8,
    ) -> u32 {
        if team_id <= 0 {
            return property.experience;
        }
        let factor = ((1.0
            - f64::from(self.difference)
                * (f64::from(average_level) - f64::from(player_level)))
            / f64::from(alive_amount))
        .max(f64::from(self.limit));
        let ratio = self.ratios[alive_amount.min(7) as usize];
        (f64::from(property.experience) * factor * f64::from(ratio)).trunc() as i32 as u32
    }

    pub fn corrective(
        self,
        property: &MonsterProperties,
        quota: u32,
        player_level: u8,
        is_first_attacker: bool,
        continuous_kill_amount: u32,
    ) -> u32 {
        let level_delta = i32::from(player_level) - property.level as i32;
        let amerce_level = level_delta.wrapping_sub(self.amerce_start_level).max(0);
        let corrective_factor = (1.0
            - f64::from(amerce_level) * f64::from(self.amerce))
        .max(f64::from(self.amerce_limit));
        let mut corrected = (f64::from(quota) * corrective_factor)
            .max(0.0)
            .trunc() as i32 as u32;
        corrected = corrected.min(property.experience);
        if level_delta.max(0) <= self.hit_base_level
            && is_first_attacker
            && continuous_kill_amount != 0
        {
            let prize = (f64::from(continuous_kill_amount) * f64::from(self.hit_prize))
                .min(f64::from(self.maximum_hit_prize));
            corrected = (f64::from(corrected) * (prize + 1.0))
                .max(0.0)
                .trunc() as i32 as u32;
        }
        corrected
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PetAttackProperties {
    pub minimum_attack: u32,
    pub maximum_attack: u32,
    pub attack_interval: u32,
    pub stop_frame: u32,
    pub speed_bits: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PetExperienceUpdate {
    pub level: u32,
    pub experience: u32,
    pub maximum_hp: u32,
    pub hit_points: u32,
}

/// Exact `CMonster::GetMaxHP` (RVA `0x000E65A0`): factor `6` действует
/// только при валидной player-owner связи, а x87 результат усекается.
pub fn maximum_hp(
    property: &MonsterProperties,
    modifiers: &MoveShapePropertyModifiers,
    has_player_pet_master: bool,
    factors: [f32; 10],
) -> u32 {
    let base = property.maximum_hp
        .wrapping_add_signed(modifiers.maximum_hp);
    if !has_player_pet_master {
        return base;
    }
    let scaled = f64::from(base)
        * f64::from(factors[6]);
    scaled.trunc() as i32 as u32
}

/// Виртуальный `CMonster::GetDodge` (RVA `0x000E6800`): базовое значение не
/// ниже единицы, а приручённый monster-owner применяет свой pet-level factor
/// до сужения результата к `ushort`.
pub fn dodge(
    property: &MonsterProperties,
    modifiers: &MoveShapePropertyModifiers,
    has_player_pet_master: bool,
    factors: [f32; 10],
) -> u16 {
    let base = (property.dodge as i32)
        .wrapping_add(modifiers.dodge).max(1) as u16;
    if has_player_pet_master {
        let scaled = f64::from(base) * f64::from(factors[4]);
        return scaled.trunc() as i32 as u16;
    }
    base
}

/// `CMonster::GetHit` (RVA `0x000E6760`): signed сумма ресурса и modifier
/// не выше нуля становится единицей до сужения к `ushort`.
pub fn hit(property: &MonsterProperties, modifiers: &MoveShapePropertyModifiers) -> u16 {
    (property.hit as i32)
        .wrapping_add(modifiers.hit).max(1) as u16
}

/// Достигнутая ресурсная часть `CMonster::GetAttackAvoid`
/// (RVA `0x000E64F0`): неположительное signed значение становится нулём,
/// а положительное ограничивается `99`.
pub fn attack_avoid(property: &MonsterProperties, modifiers: &MoveShapePropertyModifiers) -> u16 {
    (property.attack_avoid as i32)
        .wrapping_add(modifiers.attack_avoid).clamp(0, 99) as u16
}

/// Достигнутая ресурсная часть `CMonster::GetElementAvoid`
/// (RVA `0x000E6520`): контракт совпадает с physical avoid, кроме
/// разрешённой верхней границы `100`.
pub fn element_avoid(property: &MonsterProperties, modifiers: &MoveShapePropertyModifiers) -> u16 {
    (property.element_avoid as i32)
        .wrapping_add(modifiers.element_avoid).clamp(0, 100) as u16
}

/// Exact `CMonster::GetDef` (RVA `0x000E6780`): отрицательная сумма
/// свойства и runtime modifier сначала становится нулём, затем pet factor
/// `5` усекается x87 к signed DWORD.
pub fn defense(
    property: &MonsterProperties,
    modifiers: &MoveShapePropertyModifiers,
    has_player_pet_master: bool,
    factors: [f32; 10],
) -> u32 {
    let base = (property.defence as i32)
        .wrapping_add(modifiers.defense).max(0) as u32;
    if !has_player_pet_master {
        return base;
    }
    let scaled = f64::from(base) * f64::from(factors[5]);
    scaled.trunc() as i32 as u32
}

/// Exact `CMonster::GetElementResistant` (RVA `0x000E6880`): runtime
/// modifier входит до нижней границы и pet factor `3`.
pub fn element_resistance(
    property: &MonsterProperties,
    modifiers: &MoveShapePropertyModifiers,
    has_player_pet_master: bool,
    factors: [f32; 10],
) -> u32 {
    let base = (property.element_resistant as i32)
        .wrapping_add(modifiers.element_resistance).max(1) as u32;
    if !has_player_pet_master {
        return base;
    }
    let scaled = f64::from(base) * f64::from(factors[3]);
    scaled.trunc() as i32 as u32
}

/// Достигнутая часть `CMonster::GetSoulResistant` (RVA `0x000E6970`):
/// сумма ресурса и modifier проходит нижнюю границу до `ushort`.
pub fn soul_resistance(property: &MonsterProperties, modifiers: &MoveShapePropertyModifiers) -> u16 {
    (property.soul_resistant as i32)
        .wrapping_add(modifiers.soul_resistance).max(1) as u16
}

/// Достигнутая часть `CMonster::GetHpRecoverSpeed` (RVA `0x000E6990`):
/// dormant AI использует ресурсное значение после нижней границы `1` и
/// исходного сужения к `ushort`.
pub fn hp_recovery_speed(
    property: &MonsterProperties,
    modifiers: &MoveShapePropertyModifiers,
) -> u16 {
    (property.hp_recover_speed as i32)
        .wrapping_add(modifiers.hp_recovery_speed).max(1) as u16
}

/// `CMonster::GetAddSoulAtk`
/// (RVA `0x000E69D0`): signed DWORD не выше нуля даёт `0`, положительное
/// значение сужается к младшим шестнадцати битам.
pub fn soul_attack(property: &MonsterProperties, modifiers: &MoveShapePropertyModifiers) -> u16 {
    let value = (property.yao_attack as i32)
        .wrapping_add(modifiers.additional_soul_attack);
    if value > 0 { value as u16 } else { 0 }
}

/// Exact `CMonster::GetStopFrame` (RVA `0x000E6A40`): только приручённый
/// монстр с живой player-owner связью применяет pet factor `9`. Оригинал
/// умножает signed DWORD на f32 в x87 и временно включает truncation.
pub fn stop_frame(
    property: &MonsterProperties,
    has_player_pet_master: bool,
    factors: [f32; 10],
) -> u32 {
    if has_player_pet_master {
        let scaled = f64::from(property.stop_frame as i32)
            * f64::from(factors[9]);
        return scaled.trunc() as i32 as u32;
    }
    property.stop_frame
}

fn pet_scaled_attack(
    value: u32,
    factor_index: usize,
    has_player_pet_master: bool,
    factors: [f32; 10],
) -> u32 {
    let base = (value as i32).max(1) as u32;
    if !has_player_pet_master {
        return base;
    }
    let scaled = f64::from(base)
        * f64::from(factors[factor_index]);
    let scaled = scaled.trunc() as i32;
    if scaled > 0 { scaled as u32 } else { base }
}

/// Exact `CMonster::GetAtcInterval` (RVA `0x000E69F0`): базовый virtual
/// сначала сужает интервал к WORD, pet factor `7` затем усекается `__ftol2`.
pub fn attack_interval(
    property: &MonsterProperties,
    modifiers: &MoveShapePropertyModifiers,
    has_player_pet_master: bool,
    factors: [f32; 10],
) -> u32 {
    let base = (property.attack_speed as u16)
        .wrapping_add(modifiers.attack_speed as u16);
    if !has_player_pet_master {
        return u32::from(base);
    }
    let scaled = f64::from(base) * f64::from(factors[7]);
    u32::from(scaled.trunc() as i32 as u16)
}

/// Exact `CMonster::GetSpeed` (RVA `0x000E79B0`): x87 умножает два f32
/// только для валидной player-owner связи; Rust округляет результат при
/// сохранении canonical f32 скорости.
pub fn speed(base: f32, has_player_pet_master: bool, factors: [f32; 10]) -> f32 {
    if !has_player_pet_master {
        return base;
    }
    (f64::from(base) * f64::from(factors[8])) as f32
}

pub fn pet_attack_properties(
    property: &MonsterProperties,
    modifiers: &MoveShapePropertyModifiers,
    has_player_pet_master: bool,
    factors: [f32; 10],
    base_speed: f32,
) -> PetAttackProperties {
    let (minimum_attack, maximum_attack) = state_attack_bounds(
        property.minimum_attack,
        property.maximum_attack,
        modifiers,
        has_player_pet_master,
        factors,
    );
    PetAttackProperties {
        minimum_attack,
        maximum_attack,
        attack_interval: attack_interval(property, modifiers, has_player_pet_master, factors),
        stop_frame: stop_frame(property, has_player_pet_master, factors),
        speed_bits: speed(base_speed, has_player_pet_master, factors).to_bits(),
    }
}

pub fn state_attack_bounds(
    minimum: u32,
    maximum: u32,
    modifiers: &MoveShapePropertyModifiers,
    has_player_pet_master: bool,
    factors: [f32; 10],
) -> (u32, u32) {
    (
        pet_scaled_attack(
            minimum.wrapping_add_signed(modifiers.minimum_attack),
            1,
            has_player_pet_master,
            factors,
        ),
        pet_scaled_attack(
            maximum.wrapping_add_signed(modifiers.maximum_attack),
            0,
            has_player_pet_master,
            factors,
        ),
    )
}

/// Exact `CMonster::GetElementModify` (RVA `0x000E6900`): накопленный
/// runtime modifier сначала ограничивается нулём, затем pet factor `2`
/// умножается в x87 и усекается к signed DWORD.
pub fn element_modifier(
    modifiers: &MoveShapePropertyModifiers,
    has_player_pet_master: bool,
    factors: [f32; 10],
) -> u32 {
    let base = modifiers.element_modify.max(0) as u32;
    if !has_player_pet_master {
        return base;
    }
    let scaled = f64::from(base) * f64::from(factors[2]);
    scaled.trunc() as i32 as u32
}
