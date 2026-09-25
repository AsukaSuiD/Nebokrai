//! Источник COrigin (0x130).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/origin.cpp/.h.
//! Весь класс делегирован: общий зарегистрированный Begin/AI/End —
//! `immediatestate.rs`, ветка установки и таблица прибавок — zone rules
//! `skills/immediate.rs`, данные и формула состояния — zone `effects/element.rs`,
//! живой property/Begin/End — `originstate.rs`.
//! MATCH по машинной разведке порции №4: `COrigin::AI` VA `0x005AEA70` — тот же
//! цикл CTaiJi с ID 0x130: создание и Begin до End первого старого ID, замена
//! в прежней позиции, UpdateProperty и End(1). Check общий: только исходный U
//! и свойства, без reuse, visual и изменения движения.
