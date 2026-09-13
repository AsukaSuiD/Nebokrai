//! Параметры огненной стены.
//! Источник: gameserver.exe/GameServer.pdb, firewallphalanx.cpp.
//! Уровень 1 выбирает 1×1, уровень 2 — крест 3×3, любой другой — квадрат 3×3.
//! Общие маска, inherited wire и независимые часы находятся в maskedelementphalanx;
//! попадание использует общий элементальный расчёт без RP.

use super::elementphalanxattack::ElementPhalanxAttack;
use super::firewall::FIRE_WALL_SKILL_ID;
use super::maskedelementphalanx::{MaskedAreaPulse, MaskedElementPhalanx};
use crate::gameserver::appserver::masterinfo::MasterInfo;

pub(super) fn scope_for_level(level: i32) -> (i32, i32, &'static [bool]) {
    match level {
        1 => (1, 1, &[true]),
        2 => (3, 3, &[false, true, false, true, true, true, false, true, false]),
        _ => (3, 3, &[true; 9]),
    }
}

#[allow(clippy::too_many_arguments, reason = "параметры исходного конструктора стены")]
pub(super) fn new_fire_wall_phalanx(
    id: i32, master: MasterInfo, started: u32, lifetime: u32, skill_level: i32,
    frequency_ms: u32, minimum: i32, maximum: i32, element: i32, critical_chance: i32,
) -> MaskedElementPhalanx {
    MaskedElementPhalanx::new(
        id, ElementPhalanxAttack { master, skill_id: FIRE_WALL_SKILL_ID, skill_level, minimum, maximum, element, critical_chance },
        started, lifetime, MaskedAreaPulse::Periodic { frequency_ms, last_attack_ms: 0 }, scope_for_level(skill_level),
    )
}
