//! Каноническое состояние оглушения рывком `CRushState` (`0x73`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/rushstate.cpp`. Состояние запрещает движение и бой,
//! использует строгую беззнаковую проверку срока и публикует точные
//! `0xBFE03/0xBFE04`. Игрок и монстр хранят один типизированный экземпляр в
//! `CanonicalStateStorage`; пространственный владелец остаётся у `CGame` и
//! `CServerRegion`. Vtable exact EXE подтверждает общий с `CBlindState`
//! клиентский срок по `0x005F2CD0` и serializer-пару
//! `0x005F51E0/0x005EAAC0`: persisted-запись `ID + remaining time` занимает
//! 8 байт. Spatial login восстанавливает оба вложенных запрета.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;

pub(crate) const RUSH_STATE_ID: u32 = 0x73;
pub(crate) const RUSH_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RushState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl RushState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != RUSH_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(0, reader.read_u32()?))
    }

    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self {
        self.started_at_ms = now_ms;
        self
    }

    pub(crate) fn encoded_for_install(self) -> [u8; RUSH_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; RUSH_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(self, remaining: u32) -> [u8; RUSH_STATE_BYTES] {
        let mut bytes = [0; RUSH_STATE_BYTES];
        bytes[..4].copy_from_slice(&RUSH_STATE_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&remaining.to_le_bytes());
        bytes
    }

    pub(crate) const fn skill_id(self) -> u32 { RUSH_STATE_ID }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
}

fn state_message(
    identity: ShapeIdentity,
    state: RushState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) -> CMessage {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(0);
    }
    message
}

#[allow(clippy::too_many_arguments, reason = "поля задают точную точку круговой доставки состояния")]
pub(crate) fn send_rush_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: RushState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let message = state_message(identity, state, begin, now_milliseconds);
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn replace_player_rush_state(
    game: &mut CGame,
    player_id: i32,
    state: RushState,
    now_ms: u32,
) -> bool {
    let installed = game.find_player_mut(player_id).and_then(|player| {
        let context = (
            player.server_region_id()?,
            player.shape().identity(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        );
        let old = player.replace_rush_state(state);
        if old.is_some() {
            player.set_skill_moveable(true);
            player.set_skill_fightable(true);
        }
        player.set_skill_moveable(false);
        player.set_skill_fightable(false);
        Some((old, context))
    });
    let Some((old, (region_id, identity, tile_x, tile_y))) = installed else {
        return false;
    };
    if let Some(old) = old {
        let message = state_message(identity, old, false, || now_ms);
        let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
    }
    let message = state_message(identity, state, true, game_tick_milliseconds);
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
    true
}

pub(crate) fn replace_monster_rush_state(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    state: RushState,
    now_ms: u32,
) -> bool {
    let installed = region.find_monster_by_id_mut(monster_id).map(|monster| {
        let shape = monster.move_shape().shape().clone();
        let old = monster.move_shape_mut().replace_rush_state(state);
        if old.is_some() {
            monster.move_shape_mut().set_moveable(true);
            monster.move_shape_mut().set_fightable(true);
        }
        monster.move_shape_mut().set_moveable(false);
        monster.move_shape_mut().set_fightable(false);
        (old, shape)
    });
    let Some((old, shape)) = installed else { return false };
    if let Some(old) = old {
        let message = state_message(shape.identity(), old, false, || now_ms);
        let _ = game.send_game_shape_around(region, &shape, None, &message);
    }
    let message = state_message(shape.identity(), state, true, game_tick_milliseconds);
    let _ = game.send_game_shape_around(region, &shape, None, &message);
    true
}

pub(crate) fn expire_player_rush_state(game: &mut CGame, player_id: i32, now_ms: u32) -> bool {
    let finished = game.find_player_mut(player_id).and_then(|player| {
        let state = player.take_expired_rush_state(now_ms)?;
        player.set_skill_moveable(true);
        player.set_skill_fightable(true);
        Some((
            state,
            player.server_region_id()?,
            player.shape().identity(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        ))
    });
    let Some((state, region_id, identity, tile_x, tile_y)) = finished else { return false };
    let message = state_message(identity, state, false, || now_ms);
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
    true
}

pub(crate) fn expire_monster_rush_state(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    let finished = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_expired_rush_state(now_ms)?;
        monster.move_shape_mut().set_moveable(true);
        monster.move_shape_mut().set_fightable(true);
        Some((state, monster.move_shape().shape().clone()))
    });
    let Some((state, shape)) = finished else { return false };
    let message = state_message(shape.identity(), state, false, || now_ms);
    let _ = game.send_game_shape_around(region, &shape, None, &message);
    true
}
