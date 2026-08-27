//! Параметры исполнения `CNatural`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает общий с
//! `CAgility/CRapture` ход исполнения, сообщение навыка `0xBFE01`, отдельные
//! часы восстановления и строку нехватки MP `GS0288`. Само состояние насыщает
//! сопротивление стихиям до `i32::MAX`; его замена и сетевой результат
//! выполняются общим владельцем семейства в `CGame`.

pub(crate) const NATURAL_SKILL_ID: u32 = 220;
pub(crate) const SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN: u32 = 112;
