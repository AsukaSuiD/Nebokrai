//! Числовое правило смазки оружия ядом CDaubPoison.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/daubpoison.cpp/.h.

use crate::effects::DAUB_POISON_STATE_ID;

pub const DAUB_POISON_SKILL_ID: u32 = DAUB_POISON_STATE_ID;
const STATE_PERSIST_TIME: u32 = 10_002;

/// Единственный запрос срока нового состояния выполняется после завершения
/// прежнего слота; сам запрос остаётся у живого Game.
pub fn daub_poison_keep_time_ms(mut query_property: impl FnMut(u32) -> u32) -> u32 {
    query_property(STATE_PERSIST_TIME)
}
