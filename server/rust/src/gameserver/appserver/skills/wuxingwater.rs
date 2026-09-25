//! Вода CWuXingWater (0x355).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/wuxingwater.cpp/.h.
//! Весь класс делегирован: общий зарегистрированный цикл — `immediatestate.rs`,
//! ID-карта и подготовка 24 параметров — zone rules `skills/wuxing.rs`, данные
//! и формула состояния — zone `effects/wuxing.rs`, живой property/Begin/End —
//! `wuxingstate.rs`.
//! В отличие от четырёх соседних элементов общий WuXing сохраняет полный
//! signed MAX_HP без сужения до short. UNKNOWN по машинной разведке порции №4:
//! тело `CWuXingWater::AI` дизассемблом не снято.
