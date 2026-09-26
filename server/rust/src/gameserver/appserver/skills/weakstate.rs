//! Живые операции ослабления CWeakState (0x12e).
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/weakstate.cpp/.h`.
//! Тела Begin/update/restart/ветвей AI/End и смены региона перенесены
//! буквально в `nebokrai_zone::skills::weakstate` порцией T5 «zonalcast-хаб»;
//! швы арены и визуализации реализованы над `CGame` в `skills/zonalcast.rs`.
//! Здесь — делегации с прежними сигнатурами; потребители (dispatch таблицы
//! `states/state.rs` и региональный обход `game/weak.rs`) не меняются.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::effects::{WEAK_STATE_ID, WeakState};

pub(crate) fn begin_primary_weak_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: WeakState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    nebokrai_zone::skills::weakstate::begin_primary_weak_state(
        game, holder_region, holder, user, sufferer, state, now,
    )
}

pub(crate) fn update_weak_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::weakstate::update_weak_state_properties(game, region_id, holder, key, now)
}

pub(crate) fn restart_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::weakstate::restart_weak_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn update_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::weakstate::update_weak_state(game, region_id, holder, key, now)
}

pub(crate) fn set_weak_state_region(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) {
    nebokrai_zone::skills::weakstate::set_weak_state_region(game, region_id, holder, key);
}

pub(crate) fn end_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    nebokrai_zone::skills::weakstate::end_weak_state(game, region_id, holder, key)
}
