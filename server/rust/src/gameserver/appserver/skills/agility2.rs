//! Временная ловкость CAgility2 (0x81).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/agility2.cpp.
//! Общие Begin, Check, AI и visual находятся в selfstatecast; создание по
//! таблице — zone rules `skills/selfstate.rs`, данные — zone
//! `effects/agility.rs`. В отличие от постоянных вариантов заменяется только
//! первый ID81; после его End читаются WORD full-miss и persist.
//! Создание, Begin(U,U), публикация и срок экземпляра принадлежат agilitystate2.

pub(crate) use nebokrai_zone::effects::AGILITY_2_SKILL_ID;
