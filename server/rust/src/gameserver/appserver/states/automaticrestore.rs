//! Переходный адаптер живой фигуры к состоянию восстановления Zone.
//! Проекция `PlayerCombatProperties` → `AutomaticRestoreProperties`
//! перенесена в Zone `effects/automaticrestore.rs` (оба типа локальны Zone).

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::effects::{
    AUTOMATIC_RESTORE_STATE_BYTES, AutomaticRestoreMutation, AutomaticRestoreState,
};

pub(crate) fn restart_automatic_restore_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AutomaticRestoreState>(key))
        .is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}
