//! Металл CWuXingMetal (0x353).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/wuxingmetal.cpp/.h.
//! Весь класс делегирован: общий зарегистрированный цикл — `immediatestate.rs`,
//! ID-карта и подготовка 24 параметров — zone rules `skills/wuxing.rs`, данные
//! и формула состояния — zone `effects/wuxing.rs`, живой property/Begin/End —
//! `wuxingstate.rs`.
//! MATCH по машинной разведке порции №4: только этот элемент общего WuXing
//! читает и применяет дополнительный MAX_MP (usage 119) после общего порядка
//! остальных запросов.
