//! Сопротивление стихиям CNatural (0xDC).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/natural.cpp.
//! Общий caller находится в agility; ошибка MP отличается строкой GS0288.
//! После удаления всех постоянных состояний семейства читается WORD gain
//! сопротивления. Формула и запись в Zone; Begin/End — в agilitystate.

pub(crate) use nebokrai_zone::effects::NATURAL_SKILL_ID;
pub(crate) const SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN: u32 = 112;
