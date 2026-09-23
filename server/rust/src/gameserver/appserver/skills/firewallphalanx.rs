//! Параметры огненной стены.
//! Источник: gameserver.exe/GameServer.pdb, firewallphalanx.cpp.
//! Выбор маски по уровню находится в zone/skills/firewall.rs.
//! Inherited wire и независимые часы находятся в maskedelementphalanx;
//! попадание использует общий элементальный расчёт без RP.

use nebokrai_zone::skills::ElementPhalanxAttack;
use super::firewall::FIRE_WALL_SKILL_ID;
use super::maskedelementphalanx::{MaskedAreaPulse, MaskedElementPhalanx};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use nebokrai_zone::skills::fire_wall_scope;

#[allow(clippy::too_many_arguments, reason = "параметры исходного конструктора стены")]
pub(super) fn new_fire_wall_phalanx(
    id: i32, master: MasterInfo, started: u32, lifetime: u32, skill_level: i32,
    frequency_ms: u32, minimum: i32, maximum: i32, element: i32, critical_chance: i32,
) -> MaskedElementPhalanx {
    MaskedElementPhalanx::new(
        id, ElementPhalanxAttack { master, skill_id: FIRE_WALL_SKILL_ID, skill_level, minimum, maximum, element, critical_chance },
        started, lifetime, MaskedAreaPulse::Periodic { frequency_ms, last_attack_ms: 0 }, fire_wall_scope(skill_level),
    )
}
