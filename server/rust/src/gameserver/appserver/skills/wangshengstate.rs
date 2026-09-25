//! Живые callbacks сохранённого состояния CWangshengState.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/wangshengstate.cpp/.h;
//! данные, срок, codec и правило HP перенесены в Zone effects.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, end_base_applied_state, resolve_applied_state_user,
    resolve_state_move_shape, update_applied_state_end_visual, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
pub(crate) use nebokrai_zone::effects::{WANGSHENG_STATE_BYTES, WangshengState};
pub(crate) fn update_wangsheng_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, target)) = resolve_applied_state_user(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<WangshengState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now),
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WangshengState>(key)).copied()
    else { return false; };
    if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            if let Some(health) = state.capped_health(player.health(), player.maximum_health()) {
                player.set_health(health);
            }
        }
    }
    true
}

pub(crate) fn restart_wangsheng_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    false
}

pub(crate) fn update_wangsheng_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WangshengState>(key))
        .is_some_and(|state| state.expired(now_ms))
    {
        return false;
    }
    update_applied_state_end_visual(
        game, region_id, holder, key, StatePropertyTarget::Sufferer,
    );
    end_wangsheng_state(game, region_id, holder, key)
}

pub(crate) fn end_wangsheng_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WangshengState>(key)).is_none() {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, WANGSHENG_STATE_BYTES)
}
