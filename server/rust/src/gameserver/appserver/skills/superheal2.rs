//! CSuperHeal2: второе усиленное лечение, ID 0xE4.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/superheal2.cpp.
//! Общий cast принадлежит heal. Состояние хранится у выбранной S, но
//! primary Begin получает U/U; канонические holder и sufferer сохраняют
//! это различие. ID живёт в Zone `skills/heal.rs` (порция №6a).

pub(crate) const SUPER_HEAL_2_SKILL_ID: u32 = nebokrai_zone::skills::heal::SUPER_HEAL_2_SKILL_ID;
