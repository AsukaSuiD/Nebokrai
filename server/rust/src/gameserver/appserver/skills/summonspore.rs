//! Призыв споры `CSummonSpore` (`0x19C`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/summonspore.cpp`. Константа перенесена в
//! `nebokrai_zone::skills::summoncreatureskill` (общий путь семьи и статусы
//! MATCH см. там); до их переноса точечный, объектный и self-входы
//! использовали общий `summoncreatureskill`, а `CGame` разрешал владельцев
//! и выполнял dispatch.

pub(crate) use nebokrai_zone::skills::SUMMON_SPORE_SKILL_ID;
