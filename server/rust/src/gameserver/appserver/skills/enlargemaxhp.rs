//! Максимум HP CEnlargeMaxHp (0x259).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/enlargemaxhp.cpp/.h.
//! Весь класс делегирован: общий зарегистрированный Begin/AI/End —
//! `immediatestate.rs`, ветка установки и usage 118 прибавки — zone rules
//! `skills/immediate.rs`, данные и формула состояния — zone
//! `effects/maxresource.rs`, живой property/Begin/End — `enlargemaxhpstate.rs`.
//! MATCH по машинной разведке порции №4: AI семейства Enlarge завершает первый
//! прежний ID до чтения прибавки и добавляет новое состояние в конец;
//! UpdateProperty безусловен даже при отказе Begin, успешный AI — End(1).
