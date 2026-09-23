//! Границы обхода клеток CRoar в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/roar.cpp/.h;
//! CRoar::AI, VA 0x0054B1CC–0x0054B23E.

pub const ROAR_SKILL_ID: u32 = 0x83;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoarBounds {
    pub minimum_x: i32,
    pub minimum_y: i32,
    pub maximum_x: i32,
    pub maximum_y: i32,
}

/// Максимумы ограничены размером региона и обходятся включительно.
pub fn roar_bounds(source_x: i32, source_y: i32, width: i32, height: i32) -> RoarBounds {
    RoarBounds {
        minimum_x: source_x.wrapping_sub(2).max(0),
        minimum_y: source_y.wrapping_sub(2).max(0),
        maximum_x: source_x.wrapping_add(2).min(width),
        maximum_y: source_y.wrapping_add(2).min(height),
    }
}
