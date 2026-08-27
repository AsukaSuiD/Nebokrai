//! Параметры исполнения `CRapture`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает общий с
//! `CAgility/CNatural` ход исполнения, сообщение навыка `0xBFE01`, отдельные
//! часы восстановления и строку нехватки MP `GS0279`. Состояние складывает
//! `blast_attack` как `u16` с переполнением; его замена и сетевой результат
//! выполняются общим владельцем семейства в `CGame`.

pub(crate) const RAPTURE_SKILL_ID: u32 = 219;
pub(crate) const SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN: u32 = 125;
