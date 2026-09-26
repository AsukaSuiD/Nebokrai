//! Общая числовая часть PreDefense мана- и машинного щитов.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/fightdefense.cpp/.h;
//! защитные ветви (ID 0x141 и 0xDE) — в `CFightDefense::PreDefense`.
//! Произведение MP-фактора идёт неокруглённым
//! x87-продуктом `mp*0.01`; f32-копия создаётся только для деления
//! mana-ветви. Машинные якоря и история поправки округления:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use crate::combat::{AttackPower, AttackPowerType, truncate_original, truncate_original_i64_low};

pub(super) fn absorb_shield_damage(
    life: &mut i32,
    hp_factor_percent: u16,
    mp_factor_percent: u16,
    damage_factor: f32,
    player_mana: u32,
    power: &mut AttackPower,
    defenses: Option<(i32, i32)>,
) {
    power.hp_damage =
        truncate_original_i64_low(f64::from(power.hp_damage) * f64::from(damage_factor));
    if *life > 0 && player_mana > 0 && power.hp_damage > 0 {
        if let Some((physical, element)) = defenses {
            let reduction = match power.kind {
                AttackPowerType::Physical => physical / 2,
                AttackPowerType::Element => element / 2,
                AttackPowerType::Soul | AttackPowerType::Poison => 0,
            };
            power.hp_damage = power.hp_damage.wrapping_sub(reduction).max(0);
        }

        let hp_factor = f64::from(hp_factor_percent) * f64::from(0.01_f32);
        // Оригинал умножает неокруглённый x87-продукт `mp*0.01`; f32-копия
        // фактора (`fst [esp+0x18]` / `fst [esp+0x1C]`) служит только делителем mana-ветви.
        let mp_factor = f64::from(mp_factor_percent) * f64::from(0.01_f32);
        let mp_factor_f32 = mp_factor as f32;
        let hp_shield = truncate_original(hp_factor * f64::from(power.hp_damage));
        let mp_damage = truncate_original(mp_factor * f64::from(power.hp_damage));
        if *life < hp_shield {
            let old_life = *life;
            *life = 0;
            power.mp_damage = mp_damage;
            power.hp_damage =
                truncate_original(f64::from(hp_shield.wrapping_sub(old_life)) / hp_factor);
        } else if ((player_mana & 0xffff) as i32) < mp_damage {
            let available_mana = (player_mana & 0xffff) as i32;
            *life = 0;
            power.mp_damage = mp_damage;
            power.hp_damage = truncate_original(
                f64::from(mp_damage.wrapping_sub(available_mana)) / f64::from(mp_factor_f32),
            );
        } else {
            *life = (*life).wrapping_sub(hp_shield);
            power.hp_damage = 0;
            power.mp_damage = mp_damage;
        }
    }
    power.hp_damage = truncate_original(f64::from(power.hp_damage) / f64::from(damage_factor));
}
