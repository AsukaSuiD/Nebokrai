//! Тайцзи CTaiJi (0x12D).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/taiji.cpp/.h.
//! Весь класс делегирован: общий зарегистрированный Begin/AI/End —
//! `immediatestate.rs`, ветка установки и таблица прибавок — zone rules
//! `skills/immediate.rs`, данные и формула состояния — zone `effects/element.rs`,
//! живой property/Begin/End — `taijistate.rs`.
//! MATCH по машинной разведке порции №4: `CTaiJi::AI` VA `0x005AF770` —
//! создание и первичный Begin нового состояния до поиска первого старого
//! ID 0x12D, затем End старого, destroy свежего остатка, установка в ту же
//! позицию, UpdateProperty и End(1). Check общий: только исходный U и свойства,
//! без reuse, visual и изменения движения.
