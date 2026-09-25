//! Земля CWuXingEarth (0x357).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/wuxingearth.cpp/.h.
//! Весь класс делегирован: общий зарегистрированный цикл — `immediatestate.rs`,
//! ID-карта и подготовка 24 параметров — zone rules `skills/wuxing.rs`, данные
//! и формула состояния — zone `effects/wuxing.rs`, живой property/Begin/End —
//! `wuxingstate.rs`.
//! MATCH по машинной разведке порции №4: элемент разделяет общий порядок
//! двадцати четырёх запросов без дополнительных чтений, как и Metal до своего
//! usage 119.
