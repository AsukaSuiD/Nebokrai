//! Формат области `CYinYangPhalanx2` (`0x146`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/yinyangphalanx2.cpp`. Lifecycle, replacement и формула
//! побайтно совпадают с `CYinYangPhalanx`; общий типизированный владелец не
//! дублируется. Отличие — маска 1×1 на каждом уровне, подтверждённая размерами
//! `0x006A4E00..0x006A4E14` и байтами `0x006A4DFC..0x006A4DFE`.

pub(crate) const YIN_YANG_2_SCOPE: (i32, i32, &[bool]) = (1, 1, &[true]);
