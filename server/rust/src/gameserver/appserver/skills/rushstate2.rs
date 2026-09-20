//! Оглушение вторым рывком: 0x7c, общий timed payload и lifecycle Blind.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/rushstate2.cpp.
//! Отдельный ID сохраняется в арене и codec; OnAction здесь пустой,
//! поэтому Defense не снимает состояние, в отличие от собственно Blind.

pub(crate) const RUSH_2_STATE_ID: u32 = 0x7c;
pub(crate) const RUSH_2_STATE_BYTES: usize = super::blindstate::BLIND_STATE_BYTES;
pub(crate) type Rush2State = super::blindstate::BlindState<RUSH_2_STATE_ID>;

pub(crate) use super::blindstate::begin_primary_blind_state as begin_primary_rush_2_state;
