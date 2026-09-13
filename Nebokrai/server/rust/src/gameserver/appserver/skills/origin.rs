//! Источник COrigin.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/origin.cpp/.h.
//! Общий зарегистрированный Begin/AI/End — в immediatestate; Check требует
//! исходного U и свойств, без reuse, visual и изменения движения.
//! AI выбирает GetU, при NULL — GetS; создаёт состояние до End первого
//! прежнего ID и сохраняет его позицию.
//! После установки — UpdateProperty и End1. Формула и DB8 остаются у состояния.

pub(crate) const ORIGIN_SKILL_ID: u32 = 304;
pub(crate) const SKILL_USAGE_ELEMENT_MODIFY_GAIN: u32 = 115;
