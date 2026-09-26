//! Вторая ветвь божественного благословения `CGodBless2` (`0x145`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godbless2.cpp`. Общий цикл и формулы — Zone
//! `skills/godbless.rs` (порция №6a; основание см. там). ID навыка — тот
//! же, что ID состояния 2 (`effects::GOD_BLESS_STATE_2_ID`). Каждый AI
//! требует свежую S типа Monster до расхода MP; Begin принимает обычные
//! формы цели без такого предварительного запрета. Ошибка Player —
//! visual10/GS0305, затем End0. Замена состояния, в отличие от GodBless,
//! не вызывает DelExStateByType.

pub(crate) const GOD_BLESS_2_SKILL_ID: u32 = nebokrai_zone::skills::godbless::GOD_BLESS_2_SKILL_ID;
