//! Сопротивление стихиям CNatural (0xDC).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/natural.cpp.
//! Общий caller находится в selfstatecast/agility; usage 112 прибавки — zone
//! rules `skills/selfstate.rs`, данные и формула — zone `effects/agility.rs`,
//! Begin/End — agilitystate. Строка MP-отказа — GS0288.
//! UNKNOWN по машинной разведке порции №4: тело `CNatural::AI` дизассемблом
//! не снято; разделяемые цепочки Check/AI подтверждены по ManaShield.

pub(crate) use nebokrai_zone::effects::NATURAL_SKILL_ID;
