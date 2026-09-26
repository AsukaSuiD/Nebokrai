//! Призыв трупной свечи `CSummonCorpseCandle` (`0x19A`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/summoncorpsecandle.cpp`. Константа перенесена в
//! `nebokrai_zone::skills::summoncreatureskill` (общий путь семьи и статусы
//! MATCH см. там); до их переноса точечный, объектный и self-входы
//! использовали общий `summoncreatureskill`, а `CGame` разрешал владельцев
//! и выполнял dispatch.

pub(crate) use nebokrai_zone::skills::SUMMON_CORPSE_CANDLE_SKILL_ID;
