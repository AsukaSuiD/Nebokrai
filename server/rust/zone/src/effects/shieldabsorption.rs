//! Общая числовая часть PreDefense мана- и машинного щитов.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/fightdefense.cpp/.h;
//! ветви CFightDefense::PreDefense VA 0x005B0ABC–0x005B0C72 и 0x005B0CD8–0x005B0E5F.

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
        // Оригинал сохраняет только MP-коэффициент в float перед вторым умножением.
        let mp_factor = mp_factor_percent as f32 * 0.01_f32;
        let hp_shield = truncate_original(hp_factor * f64::from(power.hp_damage));
        let mp_damage = truncate_original(f64::from(mp_factor) * f64::from(power.hp_damage));
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
                f64::from(mp_damage.wrapping_sub(available_mana)) / f64::from(mp_factor),
            );
        } else {
            *life = (*life).wrapping_sub(hp_shield);
            power.hp_damage = 0;
            power.mp_damage = mp_damage;
        }
    }
    power.hp_damage = truncate_original(f64::from(power.hp_damage) / f64::from(damage_factor));
}
