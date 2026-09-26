//! CSuperHeal: усиленное периодическое лечение, ID 0xD9.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/superheal.cpp.
//! Общий cast принадлежит heal; установка заменяет первый Heal 0xD3,
//! а не прежний SuperHeal. Primary Begin получает захваченные U/S.
//! ID живёт в Zone `skills/heal.rs` (порция №6a).

pub(crate) const SUPER_HEAL_SKILL_ID: u32 = nebokrai_zone::skills::heal::SUPER_HEAL_SKILL_ID;
