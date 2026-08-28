//! Вторая идентичность семейного состояния `CGodBlessState2` (`0x145`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godblessstate2.cpp`. Layout, формулы, срок и визуальные
//! пакеты совпадают с `GodBlessState`; обе идентичности занимают один
//! канонический replacement-slot и взаимно вытесняют друг друга.

pub(crate) const GOD_BLESS_STATE_2_ID: u32 = 0x145;
