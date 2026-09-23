//! Переходный End CLifeShieldState: Cure и обновление живой фигуры в Game.
//! Источник поведения: appserver/skills/lifeshieldstate.cpp/.h.

use super::curestate::{CURE_STATE_SKILL_ID, CureState, begin_primary_cure_state};
use super::lifeshield::SKILL_USAGE_STATE_PERSIST_TIME;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_applied_state_sufferer, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};

pub(crate) use nebokrai_zone::effects::{LIFE_SHIELD_STATE_BYTES, LifeShieldState};

/// Пролог LifeShield End; visual и Remove самого щита остаются общему End.
pub(crate) fn add_life_shield_cure(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, state: LifeShieldState, key: StateKey,
) {
    let Some(properties) = game.skill_base_properties(state.skill_id(), state.skill_level()).cloned()
    else { return; };
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return; };
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let previous = shape.find_state_position(|state| state.state_id() == CURE_STATE_SKILL_ID);
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    // Срок читается из сохранённой таблицы после полного End прежнего Cure.
    // Новый Begin использует actual S как U и S; сам LifeShield ещё в арене.
    let keep = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let _ = begin_primary_cure_state(
        game, target.0, target.1, Some(target), Some(target), CureState::new(keep),
        &mut game_tick_milliseconds,
    );
    let _ = game.update_move_shape_properties(target.0, target.1);
}
