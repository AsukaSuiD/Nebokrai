//! Переходный путь Game к данным CManaShieldState в Zone.
//! Источник поведения: appserver/skills/manashieldstate.cpp/.h.

pub(crate) use nebokrai_zone::effects::{MANA_SHIELD_STATE_BYTES, ManaShieldState};

pub(crate) const MANA_SHIELD_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const MANA_SHIELD_STATE_END_MESSAGE: i32 = 0x000b_fe04;
