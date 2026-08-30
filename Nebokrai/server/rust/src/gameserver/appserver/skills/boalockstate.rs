//! Каноническое состояние связывания `CBoaLockState` (`0xD2`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/boalockstate.cpp`. Состояние запрещает только движение,
//! заменяет прежний экземпляр того же ID с end-пакетом перед begin-пакетом,
//! использует строгую беззнаковую проверку срока и публикует точные
//! `0xBFE03/0xBFE04`. Единственный экземпляр принадлежит
//! `CanonicalStateStorage`; сырой state-вектор не дублируется. Vtable exact
//! EXE подтверждает общий с `CBlindState` клиентский срок по `0x005F2CD0`.

use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;

pub(crate) const BOA_LOCK_STATE_ID: u32 = 0xd2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BoaLockState { started_at_ms: u32, keep_time_ms: u32 }
impl BoaLockState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self { Self { started_at_ms, keep_time_ms } }
    pub(crate) const fn skill_id(self) -> u32 { BOA_LOCK_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32 }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
fn send_player_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, x: i32, y: i32, state: BoaLockState, begin: bool, now_milliseconds: impl FnMut() -> u32) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(state.skill_id() as i32); if begin { message.add_long(state.client_time(now_milliseconds)); message.add_long(0); } let _ = game.send_shape_position_around(region_id, x, y, &message);
}
fn send_monster_visual(game: &CGame, region: &CServerRegion, shape: &crate::gameserver::appserver::shape::CShape, state: BoaLockState, begin: bool, now_milliseconds: impl FnMut() -> u32) {
    let identity = shape.identity(); let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(state.skill_id() as i32); if begin { message.add_long(state.client_time(now_milliseconds)); message.add_long(0); } let _ = game.send_game_shape_around(region, shape, None, &message);
}
pub(crate) fn replace_player_boa_lock_state(game: &mut CGame, player_id: i32, state: BoaLockState, now_ms: u32) -> bool {
    let installed = game.find_player_mut(player_id).and_then(|player| { let old = player.replace_boa_lock_state(state); if old.is_some() { player.set_skill_moveable(true); } player.set_skill_moveable(false); Some((old, player.server_region_id()?, player.shape().identity(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?)) }); let Some((old, region, identity, x, y)) = installed else { return false }; if let Some(old) = old { send_player_visual(game, region, identity, x, y, old, false, || now_ms); } send_player_visual(game, region, identity, x, y, state, true, game_tick_milliseconds); let _ = game.publish_player_states(player_id); true
}
pub(crate) fn replace_monster_boa_lock_state(game: &CGame, region: &mut CServerRegion, monster_id: i32, state: BoaLockState, now_ms: u32) -> bool {
    let installed = region.find_monster_by_id_mut(monster_id).map(|monster| { let shape = monster.move_shape().shape().clone(); let old = monster.move_shape_mut().replace_boa_lock_state(state); if old.is_some() { monster.move_shape_mut().set_moveable(true); } monster.move_shape_mut().set_moveable(false); (old, shape) }); let Some((old, shape)) = installed else { return false }; if let Some(old) = old { send_monster_visual(game, region, &shape, old, false, || now_ms); } send_monster_visual(game, region, &shape, state, true, game_tick_milliseconds); true
}
pub(crate) fn expire_player_boa_lock_state(game: &mut CGame, player_id: i32, now_ms: u32) -> bool {
    let finished = game.find_player_mut(player_id).and_then(|player| { let state = player.take_expired_boa_lock_state(now_ms)?; player.set_skill_moveable(true); Some((state, player.server_region_id()?, player.shape().identity(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?)) }); let Some((state, region, identity, x, y)) = finished else { return false }; send_player_visual(game, region, identity, x, y, state, false, || now_ms); true
}
pub(crate) fn expire_monster_boa_lock_state(game: &CGame, region: &mut CServerRegion, monster_id: i32, now_ms: u32) -> bool {
    let finished = region.find_monster_by_id_mut(monster_id).and_then(|monster| { let state = monster.move_shape_mut().take_expired_boa_lock_state(now_ms)?; monster.move_shape_mut().set_moveable(true); Some((state, monster.move_shape().shape().clone())) }); let Some((state, shape)) = finished else { return false }; send_monster_visual(game, region, &shape, state, false, || now_ms); true
}
