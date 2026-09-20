//! Сопротивление стихиям CNatural (0xDC).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/natural.cpp.
//! Общий caller находится в agility; ошибка MP отличается строкой GS0288.
//! После удаления всех постоянных состояний семейства читается WORD gain
//! сопротивления. Формула, Begin/End и wire экземпляра принадлежат agilitystate.

pub(crate) const NATURAL_SKILL_ID: u32 = 0xdc;
pub(crate) const SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN: u32 = 112;
