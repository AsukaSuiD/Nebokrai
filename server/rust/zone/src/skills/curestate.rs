//! Живые Begin, restart и End CCureState.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/curestate.cpp/.h`; тела перенесены
//! буквально. Данные, срок и 8-байтный codec fold
//! (Serialize `0x1F51E0`, Unserialize `0x1E9AC0`, AI fold `0x1D5BA0`)
//! принадлежат Zone `effects/cure.rs`.
//!
//! Wire-тип Begin `0x000BFE03` общий с family ManaShield (его константа —
//! отдельный владелец старого пакета); поля кадра — исходные.
//! Объявленные швы переноса (не расхождения): hub
//! `statecast::StateCastGame` реализован у прежнего владельца; общие
//! хелперы обхода арены (`end_and_destroy_state_at`,
//! `end_move_shape_state`, `remove_applied_state_from`,
//! `update_property_state_visual`, `update_applied_state_end_visual`)
//! остаются у `states/state.rs` и объявлены швами.

use crate::app::game_message::CMessage;
use crate::effects::{CURE_STATE_BYTES, CURE_STATE_SKILL_ID, CureState};
use crate::regions::ShapeIdentity;

use super::state::StateKey;
use super::statecast::{
    StateCastGame, StateCastMoveShape, StateCastPropertyTarget, state_cast_storage_participant,
};

/// Wire-тип Begin состояния: пакет тот же, что у family щитов
/// (`MANA_SHIELD_STATE_BEGIN_MESSAGE` в `manashieldstate.rs` старого пакета).
pub const CURE_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;

/// Здесь Begin нового состояния предшествует поиску и End старого:
/// во время его visual новый экземпляр ещё не принадлежит вектору состояний.
pub fn begin_and_replace_cure_state<Game: StateCastGame>(
    game: &mut Game,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: CureState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let target = sufferer?;
    begin_cure_state(game, target.0, target.1, user, sufferer, state, true, now)
}

/// Полный Begin и append без неявного поиска/End прежнего Cure.
pub fn begin_primary_cure_state<Game: StateCastGame>(
    game: &mut Game,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: CureState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    begin_cure_state(game, holder_region, holder, user, sufferer, state, false, now)
}

#[allow(clippy::too_many_arguments, reason = "место регистрации и порядок замены задаются конкретным producer-ом")]
fn begin_cure_state<Game: StateCastGame>(
    game: &mut Game,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: CureState,
    replace_previous: bool,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    game.resolve_state_move_shape(holder_region, holder)?;
    game.resolve_state_move_shape(sufferer.0, sufferer.1)?;
    if user.is_some() { state.begin_at(now()); }
    let user = match user {
        Some(user) => Some(state_cast_storage_participant(game, user)?),
        None => None,
    };
    let sufferer = state_cast_storage_participant(game, sufferer)?;
    let mut message = CMessage::new(CURE_STATE_BEGIN_MESSAGE);
    message.add_long(sufferer.1.object_type);
    message.add_long(sufferer.1.id);
    message.add_long(CURE_STATE_SKILL_ID as i32);
    message.add_ulong(state.client_state_time(&mut *now));
    message.add_long(0);
    game.send_move_shape_around(sufferer.0, sufferer.1, &message);
    let placement = if replace_previous {
        let shape = game.resolve_state_move_shape(holder_region, holder)?;
        let previous = shape.find_state_position(|state| state.state_id() == CURE_STATE_SKILL_ID);
        let placement = previous.and_then(|(_, key)| shape.applied_state_replacement_location(key));
        if let Some((index, _)) = previous {
            if !game.end_and_destroy_state_at(holder_region, holder, index) { return None; }
        }
        placement
    } else { None };
    let record = state.encoded_for_install();
    let shape = game.resolve_state_move_shape_mut(holder_region, holder)?;
    let key = match placement {
        Some(location) => shape.insert_replacement_state_record(state, &record, location)?,
        None => shape.append_applied_state_record(state, &record),
    };
    shape.begin_applied_state_visual(key, 1);
    shape.update_applied_state_visual_base(key);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

/// Базовый Begin повторного входа и visual loop1 с property-пакетом —
/// прежний `restart_cure_state` общего обхода состояний.
pub fn restart_cure_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<CureState>(key)).is_none()
    { return false; }
    if !game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_begun(key))
    { return false; }
    if game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.begin_applied_state_visual(key, 1))
    {
        game.update_property_state_visual::<CureState>(
            region_id, holder, key, StateCastPropertyTarget::Sufferer, now,
            |state, now| state.client_state_time(now),
        );
    }
    true
}

/// Первый Cure-слот живого игрока: ключ арены берётся у его MoveShape.
pub fn end_player_cure_state<Game: StateCastGame>(game: &mut Game, player_id: i32) -> bool {
    let Some(key) = game.player_cure_state_key(player_id) else { return false };
    end_player_cure_state_key(game, player_id, key)
}

/// End по уже найденному ключу первого Cure игрока: фигура игрока
/// разрешается заново перед общим End ветки.
pub fn end_player_cure_state_key<Game: StateCastGame>(game: &mut Game, player_id: i32, key: StateKey) -> bool {
    let Some((region_id, holder)) = game.player_shape_participant(player_id) else { return false };
    end_cure_state_key(game, region_id, holder, key)
}

/// Истечение конкретного Cure держателя: часы конкретного ключа арены,
/// результат — только End этого ключа.
pub fn update_cure_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.cure_state_by_key(key))
        .is_some_and(|state| state.expired(now_ms));
    expired && end_cure_state_key(game, region_id, holder, key)
}

/// Полный End состояния по ключу: visual End-пакет до снятия с держателя.
pub fn end_cure_state_key<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<CureState>(key)).is_none()
    { return false; }
    game.update_applied_state_end_visual(region_id, holder, key, StateCastPropertyTarget::Sufferer);
    let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key) else { return false; };
    game.remove_applied_state_from(region_id, holder, key, target, CURE_STATE_BYTES)
}
