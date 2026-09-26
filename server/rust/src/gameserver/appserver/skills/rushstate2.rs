//! Оглушение вторым рывком: 0x7c, общий timed payload и lifecycle Blind.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/rushstate2.cpp.
//! ID 0x7C, 8-байтная запись и тип состояния перенесены в Zone
//! `skills/rush.rs` (порция №5 «player melee»); OnAction здесь пустой,
//! поэтому Defense не снимает состояние, в отличие от собственно Blind.
//! Начало состояния остаётся общим lifecycle Blind семейства
//! (`blindstate.rs`); безпотребительный реэкспорт `RUSH_2_STATE_BYTES`
//! со старого пути снят порцией №5 (размер читают из Zone напрямую).

pub(crate) use nebokrai_zone::skills::rush::{RUSH_2_STATE_ID, Rush2State};

pub(crate) use super::blindstate::begin_primary_blind_state as begin_primary_rush_2_state;
