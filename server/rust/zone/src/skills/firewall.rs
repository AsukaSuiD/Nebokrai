//! Правила призыва и маска области CFireWall.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/firewall.cpp и firewallphalanx.cpp/.h.
//! Summon VA 0x005ABC70, конструктор области VA 0x005FFF30.

use crate::combat::truncate_original;
use super::ElementSummonLiveField;

pub const FIRE_WALL_SKILL_ID: u32 = 0x134;
const LIFETIME_SCALE_PROPERTY: u32 = 20_010;
const LIFETIME_PROPERTY: u32 = 30_001;
const MAXIMUM_ATTACK_PROPERTY: u32 = 20_009;
const MINIMUM_ATTACK_PROPERTY: u32 = 20_008;
const FREQUENCY_PROPERTY: u32 = 6_001;
const CROSS: [bool; 9] = [false, true, false, true, true, true, false, true, false];
const FULL: [bool; 9] = [true; 9];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FireWallSummonParameters {
    pub lifetime_ms: u32,
    pub critical_chance: i32,
    pub element_attack: i32,
    pub maximum_attack: i32,
    pub minimum_attack: i32,
    pub frequency_ms: u32,
    pub skill_level: i32,
}

impl FireWallSummonParameters {
    /// Порядок Summon: срок → живые CCH и элемент → диапазон → частота → уровень.
    pub fn read(
        mut query_property: impl FnMut(u32) -> u32,
        mut read_live: impl FnMut(ElementSummonLiveField) -> Option<i32>,
        current_level: impl FnOnce() -> Option<i32>,
        scaled_element: i32,
    ) -> Option<Self> {
        let lifetime_ms = fire_wall_lifetime(&mut query_property, scaled_element);
        let critical_chance = (read_live(ElementSummonLiveField::CriticalChance)? as u16) as i32;
        let element_attack = read_live(ElementSummonLiveField::AddElementAttack)?
            .wrapping_add(scaled_element);
        let maximum_attack = query_property(MAXIMUM_ATTACK_PROPERTY) as i32;
        let minimum_attack = query_property(MINIMUM_ATTACK_PROPERTY) as i32;
        let frequency_ms = query_property(FREQUENCY_PROPERTY);
        let skill_level = current_level()?;
        Some(Self { lifetime_ms, critical_chance, element_attack, maximum_attack,
            minimum_attack, frequency_ms, skill_level })
    }
}

/// В исходном Summon коэффициент округляется до f32 до запроса базового срока.
fn fire_wall_lifetime(mut query_property: impl FnMut(u32) -> u32, scaled_element: i32) -> u32 {
    let constant = query_property(LIFETIME_SCALE_PROPERTY);
    let scale = constant.wrapping_mul(scaled_element as u32).wrapping_add(100);
    let factor = (f64::from(scale) * f64::from(0.01_f32)) as f32;
    let base_lifetime = query_property(LIFETIME_PROPERTY);
    truncate_original(f64::from(base_lifetime) * f64::from(factor)) as u32
}

pub fn fire_wall_scope(level: i32) -> (i32, i32, &'static [bool]) {
    match level {
        1 => (1, 1, &[true]),
        2 => (3, 3, &CROSS),
        _ => (3, 3, &FULL),
    }
}
