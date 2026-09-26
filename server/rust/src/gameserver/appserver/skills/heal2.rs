//! CHeal2: второй вариант периодического лечения, ID навыка/состояния 0xE3.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/heal2.cpp.
//! Зарегистрированный cast и visual принадлежат heal; primary-состояние —
//! healstate. ID живёт в Zone `skills/heal.rs` (порция №6a).

pub(crate) const HEAL_2_SKILL_ID: u32 = nebokrai_zone::skills::heal::HEAL_2_SKILL_ID;
