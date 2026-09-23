//! Применение постоянной прибавки максимума HP к живому игроку Game.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/enlargemaxhpstate.cpp` и `enlargemaxhpstate.h`.
//! Данные и формула находятся в `zone/effects/maxresource.rs`.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::effects::EnlargeMaxHpState;
pub(crate) use nebokrai_zone::effects::MAX_RESOURCE_STATE_BYTES as ENLARGE_MAX_HP_STATE_BYTES;

/// OnUpdateProperties 0x005E2420: GetSufferer, затем только player-формула.
/// Visual, IsEnded-gate и чтения часов у этого override отсутствуют.
pub(crate) fn update_enlarge_max_hp_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    crate::gameserver::appserver::states::state::update_player_state_properties::<EnlargeMaxHpState>(
        game, region_id, holder, key, |state, player| {
            player.update_state_combat_properties(|mut properties| { properties.maximum_hp = state.apply(properties.maximum_hp); properties });
        },
    )
}

pub(crate) fn restart_enlarge_max_hp_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnlargeMaxHpState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
}

pub(crate) fn end_enlarge_max_hp_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnlargeMaxHpState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, ENLARGE_MAX_HP_STATE_BYTES)
}
