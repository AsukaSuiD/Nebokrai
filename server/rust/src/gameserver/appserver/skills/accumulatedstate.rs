//! Общий цикл накопления EnergyHolding и SoulCollect.
//! Источник: gameserver.exe/GameServer.pdb, energyholdingstate.cpp и
//! soulcollectstate.cpp. Данные, signedness лимита и участник Add остаются
//! у владельцев; хранение и идентичность обеспечивает существующая SlotMap.
//!
//! Поиск останавливается на первом ID, включая ended и чужой тип. Typed
//! экземпляр сохраняет исходные параметры. Новый Begin создаёт loop1 без
//! публикации, Add увеличивает счётчик перед парой 2→1, и только затем
//! экземпляр попадает в список. Каждый visual заново разрешает Sufferer.

use crate::gameserver::appserver::moveshape::{AppliedState, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_applied_state_sufferer, resolve_applied_state_user,
    resolve_state_move_shape, resolve_state_move_shape_mut, update_applied_state_visual_base,
};
use crate::gameserver::appserver::states::visualeffect::CVisualEffect;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(super) enum AccumulationParticipant { User, Sufferer }

pub(super) trait AccumulatedState: AppliedState + Copy {
    const ID: u32;
    const PARTICIPANT: AccumulationParticipant;
    fn increment(&mut self) -> bool;
    fn record(self) -> [u8; 12];
    fn client_fields(self) -> (u32, u32);
}

fn publish_visual<T: AccumulatedState>(
    game: &mut CGame, target: (i32, ShapeIdentity), state: T, mode: u32,
) {
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let region = shape.shape().get_region_id();
    let identity = shape.shape().identity();
    let mut message = CMessage::new(if mode == 1 { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_ulong(T::ID);
    if mode == 1 {
        let (remaining, additional) = state.client_fields();
        message.add_ulong(remaining);
        message.add_ulong(additional);
    }
    let _ = game.send_move_shape_around(region, identity, &message);
}

pub(super) fn update_accumulated_visual<T: AccumulatedState>(
    game: &mut CGame, source: (i32, ShapeIdentity), key: StateKey, mode: u32,
) {
    let Some((state, ended)) = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| Some((*shape.applied_state::<T>(key)?, shape.applied_state_visual_ended(key)?)))
    else { return; };
    if !ended && let Some(target) = resolve_applied_state_sufferer(game, source.0, source.1, key) {
        publish_visual(game, target, state, mode);
    }
    update_applied_state_visual_base(game, source.0, source.1, key);
}

pub(super) fn add_accumulated_state<T: AccumulatedState>(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce(&CGame) -> Option<T>, now: &mut dyn FnMut() -> u32,
) -> bool {
    if let Some((_, key)) = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == T::ID))
    {
        if resolve_state_move_shape(game, source.0, source.1)
            .and_then(|shape| shape.applied_state::<T>(key)).is_some()
        {
            let participant = match T::PARTICIPANT {
                AccumulationParticipant::User => resolve_applied_state_user(game, source.0, source.1, key),
                AccumulationParticipant::Sufferer => resolve_applied_state_sufferer(game, source.0, source.1, key),
            };
            if participant.is_some() && resolve_state_move_shape_mut(game, source.0, source.1)
                .and_then(|shape| shape.applied_state_mut::<T>(key)).is_some_and(T::increment)
            {
                update_accumulated_visual::<T>(game, source, key, 2);
                update_accumulated_visual::<T>(game, source, key, 1);
            }
            return true;
        }
    }
    let Some(mut state) = create(game) else { return false; };
    // Базовый Begin с ненулевым user читает часы и для состояния без таймера.
    let _ = now();
    let Some(shape) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
    let participant = (shape.shape().get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..shape.shape().identity()
    });
    let mut visual = CVisualEffect::new();
    visual.begin_visual_effect(1);
    if resolve_state_move_shape(game, participant.0, participant.1).is_some() && state.increment() {
        publish_visual(game, participant, state, 2);
        visual.update_visual_effect();
        publish_visual(game, participant, state, 1);
        visual.update_visual_effect();
    }
    let Some(shape) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    let record = state.record();
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(participant));
    shape.set_applied_state_sufferer(key, Some(participant));
    true
}
