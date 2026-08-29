//! Призыв споры `CSummonSpore` (`0x19C`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/summonspore.cpp`. Точечный, объектный и
//! self-входы используют общий `summoncreatureskill`: skill-owner сохраняет
//! reuse, cast-delay, пакеты `0xBFE01`, количество создаваемых существ и их
//! lifetime, а `CGame` только разрешает владельцев и выполняет dispatch.

pub(crate) const SUMMON_SPORE_SKILL_ID: u32 = 0x19c;
