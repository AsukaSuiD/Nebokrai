//! Живые Begin, restart, update и End CDaubPoisonState в переходном Game.
//! Данные, срок и сохраняемая запись перенесены в Zone
//! `effects/daubpoison.rs` (происхождение и адреса указаны там).
//! Источник поведения: appserver/skills/daubpoisonstate.cpp/.h.
//! Общая арена хранит один экземпляр, стороны Begin, serialized span и
//! visual; заимствования заменяют CState* без копирования payload между
//! End и callback. Object Begin0x005F1A50 требует S: base clock при
//! non-NULL U до getters → visual loop1/Update0 → append у caller.
//! NULL-user restart сохраняет timestamp и User. Visual0x005F1B00 читает
//! actual S; BFE03 содержит type/id/DF/remaining/0, BFE04 — type/id/DF.
//! End0x005FD420: optional visual1 → свежий S → общий RemoveState(pointer),
//! без записи state.ended. Missing/ended visual подавляет только пакет;
//! существующий ресурс получает base tail и при отсутствующем S.
//! Список/свойства после удаления обслуживает общий lifecycle; первичный
//! append сам UpdateProperty не вызывает. Первичное Begin(U,U) и append
//! принадлежат захваченному CMoveShape, не только игроку. После часов
//! отдельно сохраняются стороны U/S; свежий S обслуживает visual.
//! Player-only проверка переноса яда принадлежит ударам.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut, update_applied_state_end_visual,
    update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{
    DAUB_POISON_STATE_BYTES, DaubPoisonState,
};

const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;

pub(crate) fn begin_primary_daub_poison_state(
    game: &mut CGame,
    source: (i32, ShapeIdentity),
    keep_time_ms: u32,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let mut state = DaubPoisonState::new(keep_time_ms);
    resolve_state_move_shape(game, source.0, source.1)?;
    state.begin_at(now());
    let participant = |source: (i32, ShapeIdentity)| {
        let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
        Some((shape.get_region_id(), ShapeIdentity {
            ex_id: CGuid::GUID_INVALID, ..shape.identity()
        }))
    };
    let user = participant(source)?;
    let sufferer = participant(source)?;
    if let Some(shape) = resolve_state_move_shape(game, sufferer.0, sufferer.1) {
        let target = (shape.shape().get_region_id(), shape.shape().identity());
        let mut message = CMessage::new(STATE_BEGIN_MESSAGE);
        message.add_long(target.1.object_type);
        message.add_long(target.1.id);
        message.add_ulong(state.skill_id());
        message.add_long(state.client_state_time(now) as i32);
        message.add_ulong(0);
        let _ = game.send_move_shape_around(target.0, target.1, &message);
    }
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, source.0, source.1)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(user));
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

pub(crate) fn restart_daub_poison_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<DaubPoisonState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer,
            now, |state, now| state.client_state_time(now),
        );
    }
    true
}

pub(crate) fn update_daub_poison_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key))
        .is_some_and(|state| state.expired(now_ms))
    {
        return false;
    }
    end_daub_poison_state(game, region_id, holder, key)
}

pub(crate) fn end_daub_poison_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<DaubPoisonState>(key)).is_none()
    {
        return false;
    }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, DAUB_POISON_STATE_BYTES)
}
