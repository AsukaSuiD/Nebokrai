//! Правила призыва и маска области CFireWall.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/firewall.cpp и firewallphalanx.cpp/.h.
//! Summon VA 0x005ABC70, конструктор области VA 0x005FFF30.

use crate::combat::truncate_original;

pub const FIRE_WALL_SKILL_ID: u32 = 0x134;
const LIFETIME_SCALE_PROPERTY: u32 = 20_010;
const LIFETIME_PROPERTY: u32 = 30_001;
const CROSS: [bool; 9] = [false, true, false, true, true, true, false, true, false];
const FULL: [bool; 9] = [true; 9];

/// В исходном Summon коэффициент округляется до f32 до запроса базового срока.
pub fn fire_wall_lifetime(mut query_property: impl FnMut(u32) -> u32, scaled_element: i32) -> u32 {
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
