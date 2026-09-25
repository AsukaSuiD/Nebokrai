//! Дерево CWuXingWood (0x354).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/wuxingwood.cpp/.h.
//! Весь класс делегирован: общий зарегистрированный цикл — `immediatestate.rs`,
//! ID-карта и подготовка 24 параметров — zone rules `skills/wuxing.rs`, данные
//! и формула состояния — zone `effects/wuxing.rs`, живой property/Begin/End —
//! `wuxingstate.rs`.
//! UNKNOWN по машинной разведке порции №4: тело `CWuXingWood::AI` дизассемблом
//! не снято; общий с Metal/Earth порядок подготовки принят по общему циклу.
