//! Временная ловкость CAgility2 (0x81).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/agility2.cpp.
//! Общие Begin, Check, AI и visual находятся в agility/selfcastvisual.
//! В отличие от постоянных вариантов заменяется только первый ID81;
//! после его End читаются WORD full-miss и persist. Создание, Begin(U,U),
//! публикация и срок экземпляра принадлежат agilitystate2.

pub(crate) const AGILITY_2_SKILL_ID: u32 = 0x81;
