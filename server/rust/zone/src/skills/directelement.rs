//! Числовой расчёт прямых элементальных ударов и его варианты.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! идентификаторы —
//! docs/reconstruction/gameserver-skills.md#идентификаторы-сборки.
//! CalculateAttackPower: Lightning 0x005ACBA0, ChainLightning 0x00576770,
//! Infernol 0x005A6EC0, Seal 0x005AA480, SoulMirror 0x005A4A30,
//! EnergyBolt 0x0053C2A0, SnakeBolt 0x00534390, ZombieClaw 0x00537B50.
//! Исходные модули: appserver/skills/lightning, chainlightning, infernol,
//! seal, soulmirror, energybolt, snakebolt, zombieclaw (.cpp/.h).

use crate::combat::{AttackInformation, AttackPower, AttackPowerType,
    truncate_original, truncate_original_i64_low};

const DAMAGE_MODIFIER_PROPERTY: u32 = 20_002;
const HIT_PROPERTY: u32 = 20_001;
const ELEMENT_SCALE_PROPERTY: u32 = 20_015;
const MAXIMUM_ATTACK_PROPERTY: u32 = 20_009;
const MINIMUM_ATTACK_PROPERTY: u32 = 20_008;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectElementProfile {
    Lightning,
    ChainLightning,
    Infernol,
    Seal,
    SoulMirror,
    PathWeapon,
    PathUnarmed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectElementLiveField { RandomBelow(i32), AddElementAttack }

impl DirectElementProfile {
    pub const fn uses_weapon_modifier(self) -> bool {
        matches!(self, Self::ChainLightning | Self::Infernol | Self::SoulMirror)
    }
    pub const fn reads_damage_modifier(self) -> bool {
        !matches!(self, Self::Infernol | Self::SoulMirror | Self::PathWeapon | Self::PathUnarmed)
    }
    pub const fn increases_player_rp(self) -> bool { matches!(self, Self::ChainLightning) }
    pub const fn uses_critical(self) -> bool {
        !matches!(self, Self::Seal | Self::SoulMirror | Self::PathWeapon | Self::PathUnarmed)
    }
    pub const fn consumes_souls(self) -> bool {
        matches!(self, Self::PathWeapon | Self::PathUnarmed)
    }

    /// Эта стадия выполняется после нахождения таблицы и до живого оружейного множителя.
    pub fn begin_calculation(
        self, attack: &mut AttackInformation, skill_id: u32, skill_level: u8,
        mut query_property: impl FnMut(u32) -> u32,
    ) {
        attack.skill_id = skill_id;
        attack.skill_level = skill_level;
        attack.damage_modifier = if self.reads_damage_modifier() {
            query_property(DAMAGE_MODIFIER_PROPERTY) as i32
        } else { 0 };
    }

    /// Читает таблицу и живой элемент в исходном порядке. После этого Game
    /// отдельно обрабатывает критический хвост только у нужных вариантов.
    pub fn roll_damage(
        self, attack: &mut AttackInformation, element_modify: i32, souls: Option<i32>,
        mut query_property: impl FnMut(u32) -> u32,
        mut read_live: impl FnMut(DirectElementLiveField) -> Option<i32>,
    ) -> Option<()> {
        attack.hit_modifier = query_property(HIT_PROPERTY) as i32;
        let modifier = query_property(ELEMENT_SCALE_PROPERTY);
        let bonus = truncate_original(
            f64::from(modifier) * f64::from(0.01_f32) * f64::from(element_modify),
        );
        let maximum = query_property(MAXIMUM_ATTACK_PROPERTY) as i32;
        let minimum = query_property(MINIMUM_ATTACK_PROPERTY) as i32;
        let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
        let random = read_live(DirectElementLiveField::RandomBelow(width))?;
        let minimum = query_property(MINIMUM_ATTACK_PROPERTY) as i32;
        let element = read_live(DirectElementLiveField::AddElementAttack)?;
        let damage = element.wrapping_add(random).wrapping_add(minimum).wrapping_add(bonus);
        let damage = souls.map_or(damage, |souls| truncate_original_i64_low(
            f64::from(damage) * (f64::from(souls) * f64::from(0.7_f32) + f64::from(1.0_f32)),
        )).max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0,
        });
        Some(())
    }
}
