//! Применение и снятие Hearten у живой фигуры Game.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/heartenstate.cpp` и `heartenstate.h`. Тела перенесены
//! буквально в Zone `skills/heartenstate.rs` (порция №6a «state-касты
//! пятёрки + heal-квартет»; основание и статусы см. там). Здесь —
//! делегации с прежними сигнатурами: обход состояний и потребители не
//! меняются.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_zone::effects::HeartenState;

pub(crate) fn begin_primary_hearten_state(
    game: &mut CGame,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: (i32, ShapeIdentity),
    state: HeartenState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    nebokrai_zone::skills::heartenstate::begin_primary_hearten_state(game, user, sufferer, state, now)
}

pub(crate) fn update_hearten_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
    key: StateKey, now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::heartenstate::update_hearten_state_properties(game, region_id, holder, key, now)
}

pub(crate) fn restart_hearten_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::heartenstate::restart_hearten_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn update_hearten_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    nebokrai_zone::skills::heartenstate::update_hearten_state(game, region_id, holder, key, now_ms)
}

/// Полный End: visual End-пакет адресату до снятия с держателя.
pub(crate) fn end_hearten_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    nebokrai_zone::skills::heartenstate::end_hearten_state(game, region_id, holder, key)
}
