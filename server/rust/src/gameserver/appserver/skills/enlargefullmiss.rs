//! Полное уклонение CEnlargeFullMiss.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/enlargefullmiss.cpp/.h.
//! Общий зарегистрированный Begin/AI/End — в immediatestate; Check требует
//! исходного U и свойств, без reuse, visual и изменения движения.
//! AI выбирает GetU, при NULL — GetS; завершает первый прежний ID до создания
//! состояния и добавляет новое в конец.
//! После установки — UpdateProperty и End1. Формула и DB8 остаются у состояния.

pub(crate) use nebokrai_zone::effects::ENLARGE_FULL_MISS_STATE_ID as ENLARGE_FULL_MISS_SKILL_ID;
pub(crate) const SKILL_USAGE_FULL_MISS_GAIN: u32 = 115;
