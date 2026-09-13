//! Максимум HP CEnlargeMaxHp.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/enlargemaxhp.cpp/.h.
//! Общий зарегистрированный Begin/AI/End — в immediatestate; Check требует
//! исходного U и свойств, без reuse, visual и изменения движения.
//! AI выбирает GetU, при NULL — GetS; завершает первый прежний ID до создания
//! состояния и добавляет новое в конец.
//! После установки — UpdateProperty и End1. Формула и DB8 остаются у состояния.

pub(crate) const ENLARGE_MAX_HP_SKILL_ID: u32 = 601;
pub(crate) const SKILL_USAGE_MAX_HP_GAIN: u32 = 118;
