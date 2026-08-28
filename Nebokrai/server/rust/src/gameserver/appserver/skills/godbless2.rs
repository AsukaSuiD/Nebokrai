//! Вторая ветвь божественного благословения `CGodBless2` (`0x145`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godbless2.cpp`. Совпадающий pipeline и формулы исполняет
//! семейный owner `godbless.rs`; эта ветвь отдельно сохраняет обязательную
//! monster-цель и ошибку `GS0305` для остальных запросов.

pub(crate) const GOD_BLESS_2_SKILL_ID: u32 = super::godblessstate2::GOD_BLESS_STATE_2_ID;
