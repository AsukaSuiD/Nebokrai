//! Идентификатор CWuXingMetal; только этот элемент общего WuXing читает
//! и применяет дополнительный MAX_MP_GAIN.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/wuxingmetal.cpp/.h`; данные — `zone/effects/wuxing.rs`.

pub(crate) use nebokrai_zone::effects::WUXING_METAL_STATE_ID as WUXING_METAL_SKILL_ID;
