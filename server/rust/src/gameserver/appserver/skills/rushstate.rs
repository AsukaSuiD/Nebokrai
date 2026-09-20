//! Оглушение первым рывком: 0x73, общий timed payload и lifecycle Blind.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/rushstate.cpp.
//! Отдельный ID сохраняется в арене и codec; OnAction здесь пустой,
//! поэтому Defense не снимает состояние, в отличие от собственно Blind.

pub(crate) const RUSH_STATE_ID: u32 = 0x73;
pub(crate) const RUSH_STATE_BYTES: usize = super::blindstate::BLIND_STATE_BYTES;
pub(crate) type RushState = super::blindstate::BlindState<RUSH_STATE_ID>;

pub(crate) use super::blindstate::begin_primary_blind_state as begin_primary_rush_state;
