//! Живой Begin/End и visual `CRideState` GameServer в переходном Game.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходные owners
//! `appserver/other states/ridestate.h/.cpp`. Данные, wire-кодек и gate
//! проверки товара перенесены в Zone `effects/ride.rs` (там же адреса
//! Serialize/Unserialize/AI).
//! Первичный object Begin0x004F8D60 требует nonnull sufferer, затем выполняет
//! base Begin(self,self), loop1 visual Update(0), SetFightable(false), check=0.
//! Единственный base clock поглощается на своём месте; base timestamp в Ride
//! не имеет достигнутых потребителей и не подменяет check timestamp.
//! Helper не регистрирует payload: caller после успеха сохраняет actual User/S
//! и остаточное loop1 visual-состояние в общей арене без повторного пакета.
//! Visual0x004F8E20 публикует BFE03(type,id,100004,0,type<<16|level)
//! либо BFE04(type,id,100004) через actual sufferer. Pure message-helper
//! не подменяет optional/ended/target gates и base visual tail владельца.
//! Общий CGame-вход visual выполняет base tail даже при NULL sufferer/ended,
//! но не при отсутствующем ресурсе. Повторный Begin(NULL,holder) сохраняет User
//! и не читает base clock: после visual ставит fight-lock и check=0.
//! End0x004F8D10: optional visual Update(1), свежий GetSufferer,
//! SetFightable(true), RemoveState у этой цели. Base End, запись ended,
//! принудительное удаление из holder и отдельный внешний Update отсутствуют.
//! Только фактическое RemoveState выполняет обычный property callback.
//! Destructor0x004F8FC0 освобождает имя и безусловно переходит в CState dtor
//! 0x005DBD40 (tail-jump0x004F8FED): освобождение visual, без End и пакетов.
//! Rust освобождает ресурсы безопасно; отказ native allocator не эмулируется,
//! исчезнувший после доставки target даёт отказ Begin перед fight-lock вместо
//! доступа по stale pointer; уже отправленный визуал не откатывается.
//! OnUpdateProperties0x004F9000 разрешает sufferer как CPlayer и ищет первый
//! live packet goods через original-name index именно этого экземпляра.
//! Он не использует goods-список AI и не вызывает visual/часы.
//! CPlayer::apply_ride_state_properties заимствует state и goods, затем
//! выполняет общий MountEquipRide(true) → MountEquipRide(false)0x0043C5E0.
//! Недостигнутые coordinate/typed-target перегрузки Begin сохранены только в локальном исследовательском корпусе.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{RIDE_STATE_ID, RideState};

pub(crate) fn ride_state_visual_message(
    target: ShapeIdentity,
    state: &RideState,
    begin: bool,
) -> CMessage {
    let mut message = CMessage::new(if begin { 0x0b_fe03 } else { 0x0b_fe04 });
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_ulong(RIDE_STATE_ID);
    if begin {
        message.add_long(state.client_state_time());
        message.add_ulong(state.additional_data());
    }
    message
}

pub(crate) fn begin_primary_ride_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state: &mut RideState,
    now: &mut dyn FnMut() -> u32,
) -> Option<(i32, ShapeIdentity)> {
    resolve_state_move_shape(game, region_id, holder)?;
    let _base_timestamp = now();
    let shape = resolve_state_move_shape(game, region_id, holder)?.shape();
    let participant = (
        shape.get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() },
    );
    let message = ride_state_visual_message(participant.1, state, true);
    let _ = game.send_move_shape_around(participant.0, participant.1, &message);
    let sufferer = resolve_state_move_shape_mut(game, participant.0, participant.1)?;
    sufferer.set_fightable(false);
    state.reset_goods_check();
    Some(participant)
}
