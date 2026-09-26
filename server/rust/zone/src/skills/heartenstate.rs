//! Применение и снятие Hearten у живой фигуры Game.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb (точная
//! пара `4F5C98E0…` + RSDS match), `appserver/skills/heartenstate.cpp` и
//! `heartenstate.h`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/heartenstate.rs`; тела перенесены
//! буквально порцией №6a. Данные, срок и формула прибавки — Zone
//! `effects/hearten.rs` (ctor `0x005EE500`, vtable `0x006600D4`, writer
//! `0x005D4D10`, reader `0x004F9D80`, property callback `0x005EE740`).
//!
//! Loop1 visual создаёт общий каталог арены. Между Begin и append нет
//! внешнего callback; первый UpdateVisualEffect принадлежит последующему
//! UpdateProperty. Объявленные швы переноса (не расхождения): hub
//! `statecast::*` реализован у прежнего владельца; property-визуалы и End
//! ветки остаются у `states/state.rs` и объявлены швами.

use crate::effects::{HEARTEN_STATE_BYTES, HeartenState};
use crate::regions::ShapeIdentity;

use super::state::StateKey;
use super::statecast::{
    StateCastGame, StateCastMoveShape, StateCastPlayer, StateCastPropertyTarget,
    state_cast_storage_participant,
};

/// Primary Begin(U,S) → append: часы только при ненулевом user,
/// канонические участники с обнулённым ex_id.
pub fn begin_primary_hearten_state<Game: StateCastGame>(
    game: &mut Game,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: (i32, ShapeIdentity),
    mut state: HeartenState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    if user.is_some() { state.begin_at(now()); }
    let user = match user {
        Some(user) => Some(state_cast_storage_participant(game, user)?),
        None => None,
    };
    let sufferer = state_cast_storage_participant(game, sufferer)?;
    let record = state.encoded_for_install();
    let shape = game.resolve_state_move_shape_mut(sufferer.0, sufferer.1)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

/// Property callback: стандартный visual Update(0) адресату, затем
/// прибавка MAX_HP только при sufferer-игроке.
pub fn update_hearten_state_properties<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, sufferer)) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    let _ = game.update_property_state_visual::<HeartenState>(
        region_id, holder, key, StateCastPropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    if sufferer.object_type != 400 { return true; }
    game.update_player_state_properties::<HeartenState>(region_id, holder, key, |state, player| {
        player.update_state_combat_properties(|mut properties| {
            properties.maximum_hp = state.apply(properties.maximum_hp);
            properties
        });
    })
}

/// Повторный вход: базовый Begin(NULL, holder) и visual loop1 без пакета.
pub fn restart_hearten_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<HeartenState>(key)).is_none()
    { return false; }
    if !game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_begun(key))
    { return false; }
    let _ = game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.begin_applied_state_visual(key, 1));
    true
}

/// Истечение конкретного Hearten: общий End ветки.
pub fn update_hearten_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<HeartenState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_hearten_state(game, region_id, holder, key)
}

/// Полный End: visual End-пакет адресату до снятия с держателя.
pub fn end_hearten_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<HeartenState>(key)).is_none()
    { return false; }
    game.update_applied_state_end_visual(region_id, holder, key, StateCastPropertyTarget::Sufferer);
    let Some(sufferer) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    game.remove_applied_state_from(region_id, holder, key, sufferer, HEARTEN_STATE_BYTES)
}
