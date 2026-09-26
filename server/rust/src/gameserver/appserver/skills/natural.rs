//! Сопротивление стихиям CNatural (0xDC).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/natural.cpp.
//! Общий caller находится в selfstatecast/agility; usage 112 прибавки — zone
//! rules `skills/selfstate.rs`, данные и формула — zone `effects/agility.rs`,
//! Begin/End — agilitystate. Строка MP-отказа — GS0288.
//! Тело `CNatural::AI` (`0x16C0B0`) досверено VERIFIED-MATCH: точный
//! клон-шаблон `CManaShield::AI`; итоговый статус — zone `skills/selfstate.rs`.

pub(crate) use nebokrai_zone::effects::NATURAL_SKILL_ID;
