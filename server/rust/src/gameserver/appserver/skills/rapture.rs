//! Модификатор blast_attack CRapture (0xDB).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/rapture.cpp.
//! Общий caller находится в agility, строка ошибки MP — GS0279. После
//! удаления всех постоянных состояний семейства читается WORD blast gain.
//! Формула и запись в Zone; Begin/End — в agilitystate.

pub(crate) use nebokrai_zone::effects::RAPTURE_SKILL_ID;
pub(crate) const SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN: u32 = 125;
