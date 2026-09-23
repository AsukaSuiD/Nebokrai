//! Параметры нового состояния CHearten.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/hearten.cpp/.h;
//! CHearten::AI VA 0x00551CAE–0x00551CC7.

use crate::effects::HeartenState;

const MAX_HP_GAIN: u32 = 118;
const PERSIST: u32 = 10_002;

/// Вызывать после удаления прежнего состояния: прибавка читается перед сроком.
pub fn hearten_state(mut query_property: impl FnMut(u32) -> u32) -> HeartenState {
    let gain = query_property(MAX_HP_GAIN) as i32;
    let keep_time_ms = query_property(PERSIST);
    HeartenState::new(0, keep_time_ms, gain)
}
