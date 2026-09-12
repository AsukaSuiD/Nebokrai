//! Фронтальный рубящий удар CJuCut (0x6C).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/jucut.cpp.
//! Оружие категории 2, отказ GS0292. Зарегистрированный Begin/AI/End и visual
//! принадлежат frontcellswordcast/frontcellswordvisual; frontcellsword атакует
//! только первую CMoveShape лицевой клетки, не пропуская её при отказе.

pub(crate) const JU_CUT_SKILL_ID: u32 = 0x6c;
