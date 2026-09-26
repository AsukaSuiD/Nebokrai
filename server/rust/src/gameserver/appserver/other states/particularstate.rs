//! Живые Begin/restart/AI/End CParticularState в переходном Game.
//! Источник: gameserver.exe + GameServer.pdb, исходный owner
//! appserver/other states/particularstate.cpp. Данные и 8-байтная запись
//! перенесены в Zone `effects/particular.rs` (там же адреса конструктора
//! и кодека). Экземпляры принадлежат общей арене CMoveShape; Vec/SlotMap
//! и заимствования заменяют CState*/ручной lifetime.
//! Object Begin0x004F9710 требует non-NULL sufferer:
//! base self/self clock → visual loop1/BFE03 → base tail → checkstamp=0,
//! затем append. NULL-user restart сохраняет User и не читает часы.
//! Base timestamp не потребляется AI/codec и не хранится фиктивным таймером.
//! Visual0x004F97C0 использует actual sufferer; absent/ended gates подавляют
//! пакет, но существующий ресурс всегда получает base tail. Общая арена
//! материализует residual visual первичного Begin без повторной отправки.
//! AI0x004F9900 сначала читает clock: после границы 2000 проверяет Player
//! sufferer и ненулевые GAP_EXCEPTION_STATE packet → equipment по listener.
//! Checkstamp всегда нулевой и не продвигается; death-gate отсутствует.
//! End0x005FD420: optional visual/BFE04 → свежий sufferer → RemoveState,
//! без state.ended и принудительного удаления из другой арены.
//! Vtable SetRegion меняет только sufferer.
//! Drop освобождает ресурс без End/пакетов; отказ native allocator не эмулируется.
//! Координатный/typed-target Begin0x004F9540/0x004F9610 остаются только в локальном исследовательском корпусе.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    update_applied_state_end_visual, update_applied_state_visual_base,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{PARTICULAR_STATE_BYTES, ParticularState};

pub(crate) fn begin_primary_particular_state(
    player: &mut CPlayer,
    additional: u32,
    now: &mut dyn FnMut() -> u32,
    send: &mut dyn FnMut(&CPlayer, &CMessage),
) -> StateKey {
    let state = ParticularState::new(additional);
    let _ = now();
    let participant = (
        player.shape().get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..player.shape().identity() },
    );
    let message = particular_state_begin_message(participant.1, &state);
    send(player, &message);
    let record = state.encoded();
    let shape = player.move_shape_mut();
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(participant));
    shape.set_applied_state_sufferer(key, Some(participant));
    key
}

pub(crate) fn restart_particular_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ParticularState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_particular_state_begin_visual(game, region_id, holder, key);
    }
    true
}

fn particular_state_begin_message(target: ShapeIdentity, state: &ParticularState) -> CMessage {
    let mut message = CMessage::new(0x000b_fe03);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(state.state_id());
    message.add_long(state.client_state_time());
    message.add_ulong(state.additional_data());
    message
}

fn update_particular_state_begin_visual(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    let Some(ended) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_visual_ended(key))
    else { return false };
    if !ended {
        if let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key) {
            let message = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<ParticularState>(key))
                .map(|state| particular_state_begin_message(target, state));
            if let Some(message) = message {
                let _ = game.send_move_shape_around(target_region, target, &message);
            }
        }
    }
    update_applied_state_visual_base(game, region_id, holder, key);
    true
}

pub(crate) fn update_particular_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    let checked_at_ms = runtime.now_milliseconds();
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ParticularState>(key))
    else { return false };
    if !state.due(checked_at_ms) {
        return false;
    }
    let additional = state.additional_data();
    if let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key) {
        if target.object_type == 400
            && game.find_player(target.id).is_some_and(|player| {
                player.particular_state_goods_present(additional, game.goods_factory())
            })
        {
            return false;
        }
    }
    end_particular_state(game, region_id, holder, key)
}

pub(crate) fn end_particular_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ParticularState>(key)).is_none()
    {
        return false;
    }
    update_applied_state_end_visual(
        game, region_id, holder, key, StatePropertyTarget::Sufferer,
    );
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false };
    remove_applied_state_from(game, region_id, holder, key, target, PARTICULAR_STATE_BYTES)
}
