//! Каноническое состояние стойки `CPillarState` (`0x74`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/pillarstate.cpp`. Состояние хранит коэффициент поздней
//! защиты и строгий срок, публикует `0xBFE03/0xBFE04` и остаётся единственным
//! типизированным источником для проверки запрета рывков и `PostDefense`.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const PILLAR_STATE_ID: u32 = 0x74;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PillarState { started_at_ms: u32, keep_time_ms: u32, damage_factor_bits: u32 }

impl PillarState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, damage_factor: f32) -> Self {
        Self { started_at_ms, keep_time_ms, damage_factor_bits: damage_factor.to_bits() }
    }
    pub(crate) const fn skill_id(self) -> u32 { PILLAR_STATE_ID }
    pub(crate) const fn damage_factor(self) -> f32 { f32::from_bits(self.damage_factor_bits) }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32 }
}

pub(crate) fn send_pillar_state_visual(
    game: &mut CGame, region_id: i32, identity: ShapeIdentity,
    tile_x: i32, tile_y: i32, state: PillarState, begin: bool, now_ms: u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type); message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin { message.add_long(state.client_time(|| now_ms)); message.add_long(0); }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn replace_player_pillar_state(
    game: &mut CGame, player_id: i32, state: PillarState, now_ms: u32,
) -> bool {
    let installed = game.find_player_mut(player_id).and_then(|player| {
        let context = (player.server_region_id()?, player.shape().identity(),
            player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?);
        let old = player.replace_pillar_state(state);
        if old.is_some() { player.set_skill_moveable(true); }
        player.set_skill_moveable(false); Some((old, context))
    });
    let Some((old, (region_id, identity, tile_x, tile_y))) = installed else { return false };
    if let Some(old) = old { send_pillar_state_visual(game, region_id, identity, tile_x, tile_y, old, false, now_ms); }
    send_pillar_state_visual(game, region_id, identity, tile_x, tile_y, state, true, now_ms); true
}

pub(crate) fn expire_player_pillar_state(game: &mut CGame, player_id: i32, now_ms: u32) -> bool {
    let finished = game.find_player_mut(player_id).and_then(|player| {
        let state = player.take_expired_pillar_state(now_ms)?; player.set_skill_moveable(true);
        Some((state, player.server_region_id()?, player.shape().identity(),
            player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
    });
    let Some((state, region_id, identity, tile_x, tile_y)) = finished else { return false };
    send_pillar_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms); true
}
