//! Модификатор blast_attack CRapture (0xDB).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/rapture.cpp.
//! Общий caller находится в selfstatecast/agility; usage 125 прибавки — zone
//! rules `skills/selfstate.rs`, данные и формула — zone `effects/agility.rs`,
//! Begin/End — agilitystate. Строка MP-отказа — GS0279.
//! UNKNOWN по машинной разведке порции №4: тело `CRapture::AI` дизассемблом
//! не снято; разделяемые цепочки Check/AI подтверждены по ManaShield.

pub(crate) use nebokrai_zone::effects::RAPTURE_SKILL_ID;
