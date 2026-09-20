//! Призыв скелета `CSummonSkeleton` (`0x19B`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/summonskeleton.cpp`. Точечный, объектный и
//! self-входы используют общий `summoncreatureskill`: skill-owner сохраняет
//! reuse, cast-delay, пакеты `0xBFE01`, количество создаваемых существ и их
//! lifetime, а `CGame` только разрешает владельцев и выполняет dispatch.

pub(crate) const SUMMON_SKELETON_SKILL_ID: u32 = 0x19b;
