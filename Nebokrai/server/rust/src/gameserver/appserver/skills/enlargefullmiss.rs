//! Параметры немедленного навыка `CEnlargeFullMiss`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает навык `603`:
//! он проверяет только наличие свойств, всегда выбирает владельца, заменяет
//! только прежнее состояние `603`, публикует `OnChangeStates` и завершает
//! базовый навык одним снятием запрета движения.

pub(crate) const ENLARGE_FULL_MISS_SKILL_ID: u32 = 603;
pub(crate) const SKILL_USAGE_FULL_MISS_GAIN: u32 = 115;
