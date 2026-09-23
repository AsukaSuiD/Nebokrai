//! Идентификатор CWuXingWater; общий WuXing сохраняет полный MAX_HP_GAIN
//! без сужения до short, в отличие от четырёх соседних элементов.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/wuxingwater.cpp/.h`; данные — `zone/effects/wuxing.rs`.

pub(crate) use nebokrai_zone::effects::WUXING_WATER_STATE_ID as WUXING_WATER_SKILL_ID;
