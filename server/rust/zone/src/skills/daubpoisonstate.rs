//! Живые Begin/restart/update/End `CDaubPoisonState` (`0xDF`).
//! Данные, срок и сохраняемая запись — Zone `effects/daubpoison.rs`
//! (кодек и правило срока там; здесь только обращения к живой арене, без
//! дублирования данных и записи).
//!
//! Машинные quirks: Object Begin при null U — тихий ret 0; публикация
//! `0x000BFE03` (type/id/0xDF/remaining/0) идёт до append без отдельного
//! UpdateProperty после Begin; End снимает состояние через
//! RemoveState(pointer) без записи state.ended — чужой или отсутствующий S
//! оставляет запись у держателя; истечение — строгое `now > start + keep`.
//!
//! Швы: hub `statecast::StateCastGame`/`StateCastMoveShape` (visual,
//! base Begin, `remove_applied_state_from`); обход состояний тика — общий
//! callback-реестр `states/state.rs` прежнего пакета, вызывающий
//! `update_daub_poison_state`. Первичное Begin(U,U) и append принадлежат
//! захваченному CMoveShape, не только игроку.
//!
//! Исходный владелец PDB: `appserver/skills/daubpoisonstate.cpp/.h`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#daubpoison--cdaubpoison-0xdf-и-cdaubpoisonstate

use crate::app::game_message::CMessage;
use crate::effects::{DAUB_POISON_STATE_BYTES, DaubPoisonState};
use crate::regions::ShapeIdentity;

use super::state::StateKey;
use super::statecast::{
    StateCastGame, StateCastMoveShape, StateCastPropertyTarget, state_cast_storage_participant,
};

const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;

/// Первичный Begin(U,U) после ветки применения `CDaubPoison`: U разрешается
/// до часов, часы и стороны фиксируются, visual `0x000BFE03` публикуется до
/// append; возвращённый ключ однозначен внутри одной арены. `None` — отказ
/// разрешения U либо append (Begin false → deleting-dtor нового у native).
pub fn begin_primary_daub_poison_state<Game: StateCastGame>(
    game: &mut Game,
    source: (i32, ShapeIdentity),
    keep_time_ms: u32,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let mut state = DaubPoisonState::new(keep_time_ms);
    game.resolve_state_move_shape(source.0, source.1)?;
    state.begin_at(now());
    let user = state_cast_storage_participant(game, source)?;
    let sufferer = state_cast_storage_participant(game, source)?;
    if let Some(shape) = game.resolve_state_move_shape(sufferer.0, sufferer.1) {
        let target = (shape.shape().get_region_id(), shape.shape().identity());
        let mut message = CMessage::new(STATE_BEGIN_MESSAGE);
        message.add_long(target.1.object_type);
        message.add_long(target.1.id);
        message.add_ulong(state.skill_id());
        message.add_long(state.client_state_time(&mut *now) as i32);
        message.add_ulong(0);
        game.send_move_shape_around(target.0, target.1, &message);
    }
    let record = state.encoded_for_install();
    let shape = game.resolve_state_move_shape_mut(source.0, source.1)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(user));
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

/// Повторный вход: NULL-user restart сохраняет timestamp и User; visual
/// loop1 перезапускается, property-visual `0x000BFE03` читает actual S.
pub fn restart_daub_poison_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key)).is_none()
    {
        return false;
    }
    if !game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_begun(key)) { return false; }
    if game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.begin_applied_state_visual(key, 1))
    {
        game.update_property_state_visual::<DaubPoisonState>(
            region_id, holder, key, StateCastPropertyTarget::Sufferer,
            now, |state, now| state.client_state_time(now),
        );
    }
    true
}

/// Тик AI `0x5D5BA0`: истечение срока (строгое неравенство) → полный End.
pub fn update_daub_poison_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key))
        .is_some_and(|state| state.expired(now_ms))
    {
        return false;
    }
    end_daub_poison_state(game, region_id, holder, key)
}

/// End `0x5FD420`: optional Update(1) (`0x000BFE04`) → свежий S →
/// RemoveState(pointer), без записи state.ended; отсутствующий S подавляет
/// удаление, но не visual-хвост ресурса.
pub fn end_daub_poison_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key)).is_none()
    {
        return false;
    }
    game.update_applied_state_end_visual(region_id, holder, key, StateCastPropertyTarget::Sufferer);
    let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    game.remove_applied_state_from(region_id, holder, key, target, DAUB_POISON_STATE_BYTES)
}
