//! Параметры немедленного навыка `CEnlargeMaxMp`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает навык `602`:
//! он проверяет только наличие свойств, всегда выбирает владельца, заменяет
//! только прежнее состояние `602`, публикует `OnChangeStates` и завершает
//! базовый навык одним снятием запрета движения.

pub(crate) const ENLARGE_MAX_MP_SKILL_ID: u32 = 602;
pub(crate) const SKILL_USAGE_MAX_MP_GAIN: u32 = 115;
