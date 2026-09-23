//! Снимок и числовой расчёт элементального удара призванных областей.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! EXE SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E,
//! PDB SHA-256 B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016.
//! appserver/skills/firewallphalanx.cpp/.h,
//! yinyangphalanx{,2}.cpp/.h, godthunderphalanx{,2}.cpp/.h
//! и chaosspherephalanx.cpp/.h.
//! CalculateAttackPower VA 0x00600120, 0x005FE6E0, 0x005F2600,
//! 0x005F5E90, 0x005EF3F0 и 0x005FEFB0 соответственно.

use crate::combat::{AttackInformation, AttackPower, AttackPowerType, MasterInfo, truncate_original};

/// Живые свойства источника, которые читает призыв элементальной области.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementSummonLiveField { CriticalChance, AddElementAttack }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElementPhalanxAttack {
    pub master: MasterInfo,
    pub skill_id: u32,
    pub skill_level: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub element: i32,
    pub critical_chance: i32,
}

impl ElementPhalanxAttack {
    pub fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }

    /// Вызывается после разрешения Player по attacker ID и до поиска уровня цели.
    /// Отсутствующий Player оставляет исходную пустую AttackInformation.
    pub fn begin_calculation(self, attack: &mut AttackInformation) {
        attack.skill_id = self.skill_id;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
    }

    /// Вызывается после разрешения уровня цели и живого оружейного множителя.
    /// Возвращает необходимость позднего чтения критического множителя.
    pub fn roll_damage(
        self, attack: &mut AttackInformation, weapon_modifier: f32,
        mut random_below: impl FnMut(i32) -> i32,
    ) -> bool {
        attack.damage_factor = weapon_modifier;
        attack.hit_modifier = 100;
        let width = self.maximum.wrapping_sub(self.minimum).wrapping_abs().wrapping_add(1);
        let damage = random_below(width).wrapping_add(self.minimum)
            .wrapping_add(self.element).max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0,
        });
        if random_below(100) < self.critical_chance {
            attack.critical = true;
            return true;
        }
        false
    }

    pub fn scale_critical(attack: &mut AttackInformation, critical_rate: f32) {
        for power in &mut attack.damages {
            if matches!(power.kind, AttackPowerType::Physical | AttackPowerType::Element | AttackPowerType::Soul) {
                power.hp_damage = truncate_original(
                    f64::from(power.hp_damage) * f64::from(critical_rate),
                );
            }
        }
    }
}
