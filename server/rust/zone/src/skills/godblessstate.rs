//! Живые callbacks CGodBlessState/CGodBlessState2.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/godblessstate{,2}.cpp/.h`. Прежний
//! переходный владелец — `src/gameserver/appserver/skills/godblessstate.rs`;
//! тела перенесены буквально порцией №6a. Данные, срок, codec и числовые
//! правила — Zone `effects/godbless.rs` (Serialize `0x005EE310`,
//! Unserialize `0x00601830`, GetRemainedTime `0x00601480`, AI
//! `0x00601640`, OnUpdateProperties `0x00601690`).
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::*`
//! реализован у прежнего владельца; прибавки живого монстра разрешаются
//! у владельца одним швом (`god_bless_monster_gains`), чужой layout не
//! имитируется; visual/property и End ветки остаются у `states/state.rs`
//! и объявлены швами.

use crate::effects::{GOD_BLESS_STATE_BYTES, GOD_BLESS_STATE_ID, GodBlessState};
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::state::StateKey;
use super::statecast::{StateCastGame, StateCastMoveShape, StateCastPlayer, StateCastPropertyTarget};

/// Property callback: стандартный visual Update(0), затем прибавки
/// монстру (три wrapping-сложения модификаторов) или игроку
/// (пересчёт снимка боевых свойств).
pub fn update_god_bless_state_properties<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    let _ = game.update_property_state_visual::<GodBlessState>(
        region_id, holder, key, StateCastPropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key)).copied()
    else { return false; };
    if target.object_type == MONSTER_TYPE {
        game.god_bless_monster_gains(target_region, target.id, state.monster_gains());
    } else if target.object_type == PLAYER_TYPE {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                (properties.minimum_attack, properties.maximum_attack, properties.element_modify) =
                    state.player_gains(
                        properties.minimum_attack, properties.maximum_attack, properties.element_modify,
                    );
                properties
            });
        }
    }
    true
}

/// Повторный вход: только state12F (GodBless2 restart не вызывается);
/// базовый Begin(NULL, holder) и visual loop1 без пакета.
pub fn restart_god_bless_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key)).copied()
    else { return false };
    if state.skill_id() != GOD_BLESS_STATE_ID {
        return false;
    }
    if !game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_begun(key))
    { return false; }
    let _ = game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.begin_applied_state_visual(key, 1));
    true
}

/// Истечение состояния: общий End ветки.
pub fn update_god_bless_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_god_bless_state(game, region_id, holder, key)
}

/// Полный End: End-pakет адресату только у state12F, запись ended и
/// снятие с держателя через сохранённого S.
pub fn end_god_bless_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<GodBlessState>(key)).copied()
    else { return false };
    if state.skill_id() == GOD_BLESS_STATE_ID {
        game.update_applied_state_end_visual(region_id, holder, key, StateCastPropertyTarget::Sufferer);
    }
    let Some(shape) = game.resolve_state_move_shape_mut(region_id, holder) else { return false; };
    shape.mark_applied_state_ended(key);
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false };
    game.remove_applied_state_from(region_id, holder, key, (target_region, target), GOD_BLESS_STATE_BYTES)
}
