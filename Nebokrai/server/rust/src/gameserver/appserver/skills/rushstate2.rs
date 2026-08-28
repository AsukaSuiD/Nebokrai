//! Каноническое состояние оглушения вторым рывком `CRushState2` (`0x7c`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/rushstate2.cpp`. Состояние запрещает движение и бой,
//! заменяет прежний экземпляр того же ID и публикует точные сообщения
//! `0xBFE03/0xBFE04`. Игрок и монстр хранят один типизированный экземпляр в
//! `CanonicalStateStorage`; пространственную и сетевую координацию выполняет
//! `CGame`.

use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const RUSH_2_STATE_ID: u32 = 0x7c;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Rush2State {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl Rush2State {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) const fn skill_id(self) -> u32 { RUSH_2_STATE_ID }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let elapsed = now_ms.wrapping_sub(self.started_at_ms);
        if elapsed >= self.keep_time_ms { 0 } else { self.keep_time_ms.wrapping_sub(elapsed) as i32 }
    }
}

fn state_message(identity: ShapeIdentity, state: Rush2State, begin: bool, now_ms: u32) -> CMessage {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_ms));
        message.add_long(0);
    }
    message
}

pub(crate) fn replace_player_rush_2_state(game: &mut CGame, player_id: i32, state: Rush2State, now_ms: u32) -> bool {
    let installed = game.find_player_mut(player_id).and_then(|player| {
        let context = (player.server_region_id()?, player.shape().identity(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?);
        let old = player.replace_rush_2_state(state);
        if old.is_some() { player.set_skill_moveable(true); player.set_skill_fightable(true); }
        player.set_skill_moveable(false);
        player.set_skill_fightable(false);
        Some((old, context))
    });
    let Some((old, (region_id, identity, tile_x, tile_y))) = installed else { return false };
    if let Some(old) = old {
        let message = state_message(identity, old, false, now_ms);
        let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
    }
    let message = state_message(identity, state, true, now_ms);
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
    true
}

pub(crate) fn replace_monster_rush_2_state(game: &CGame, region: &mut CServerRegion, monster_id: i32, state: Rush2State, now_ms: u32) -> bool {
    let installed = region.find_monster_by_id_mut(monster_id).map(|monster| {
        let shape = monster.move_shape().shape().clone();
        let old = monster.move_shape_mut().replace_rush_2_state(state);
        if old.is_some() { monster.move_shape_mut().set_moveable(true); monster.move_shape_mut().set_fightable(true); }
        monster.move_shape_mut().set_moveable(false);
        monster.move_shape_mut().set_fightable(false);
        (old, shape)
    });
    let Some((old, shape)) = installed else { return false };
    if let Some(old) = old {
        let message = state_message(shape.identity(), old, false, now_ms);
        let _ = game.send_game_shape_around(region, &shape, None, &message);
    }
    let message = state_message(shape.identity(), state, true, now_ms);
    let _ = game.send_game_shape_around(region, &shape, None, &message);
    true
}

pub(crate) fn expire_player_rush_2_state(game: &mut CGame, player_id: i32, now_ms: u32) -> bool {
    let finished = game.find_player_mut(player_id).and_then(|player| {
        let state = player.take_expired_rush_2_state(now_ms)?;
        player.set_skill_moveable(true);
        player.set_skill_fightable(true);
        Some((state, player.server_region_id()?, player.shape().identity(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
    });
    let Some((state, region_id, identity, tile_x, tile_y)) = finished else { return false };
    let message = state_message(identity, state, false, now_ms);
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
    true
}

pub(crate) fn expire_monster_rush_2_state(game: &CGame, region: &mut CServerRegion, monster_id: i32, now_ms: u32) -> bool {
    let finished = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_expired_rush_2_state(now_ms)?;
        monster.move_shape_mut().set_moveable(true);
        monster.move_shape_mut().set_fightable(true);
        Some((state, monster.move_shape().shape().clone()))
    });
    let Some((state, shape)) = finished else { return false };
    let message = state_message(shape.identity(), state, false, now_ms);
    let _ = game.send_game_shape_around(region, &shape, None, &message);
    true
}
