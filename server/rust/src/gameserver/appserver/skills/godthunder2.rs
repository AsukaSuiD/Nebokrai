//! Идентичность второго призыва CGodThunder2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godthunder2.cpp.
//! Общий Begin/Check/AI/visual/End — в Zone `skills/zonalcast.rs`, тело
//! Summon — в Zone `skills/godthunder.rs` (порция T5 «zonalcast-хаб»).
//! Собственная маска и WarSoul-проход выбираются владельцем формы по skill ID.

pub(crate) use nebokrai_zone::skills::GOD_THUNDER_2_SKILL_ID;
