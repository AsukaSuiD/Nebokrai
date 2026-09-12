//! Модификатор blast_attack CRapture (0xDB).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/rapture.cpp.
//! Общий caller находится в agility, строка ошибки MP — GS0279. После
//! удаления всех постоянных состояний семейства читается WORD blast gain.
//! Формула, Begin/End и wire экземпляра принадлежат agilitystate.

pub(crate) const RAPTURE_SKILL_ID: u32 = 0xdb;
pub(crate) const SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN: u32 = 125;
