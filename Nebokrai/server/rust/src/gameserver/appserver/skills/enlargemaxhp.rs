//! Параметры немедленного навыка `CEnlargeMaxHp`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает навык `601`:
//! он проверяет только наличие свойств, всегда выбирает владельца, заменяет
//! только прежнее состояние `601`, публикует `OnChangeStates` и завершает
//! базовый навык одним снятием запрета движения.

pub(crate) const ENLARGE_MAX_HP_SKILL_ID: u32 = 601;
pub(crate) const SKILL_USAGE_MAX_HP_GAIN: u32 = 118;
