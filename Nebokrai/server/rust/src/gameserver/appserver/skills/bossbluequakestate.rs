//! Каноническое состояние землетрясения синего босса `CBossBlueQuakeState` (`0x1f8`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluequakestate.cpp`. Состояние хранится только в
//! `CanonicalStateStorage`: замена сначала завершает прежнюю блокировку, затем
//! запрещает движение и бой до строгой границы срока. Пакеты начала и завершения
//! сохраняют `0xBFE03/04`; истечение для игрока и монстра, а также снятие
//! очищением проходят через того же канонического владельца.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const BOSS_BLUE_QUAKE_STATE_ID: u32 = 0x1f8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BossBlueQuakeState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl BossBlueQuakeState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) const fn skill_id(self) -> u32 { BOSS_BLUE_QUAKE_STATE_ID }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms);
        if deadline <= now_ms { 0 } else { deadline.wrapping_sub(now_ms) as i32 }
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точную точку круговой доставки состояния")]
pub(crate) fn send_boss_blue_quake_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BossBlueQuakeState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn expire_player_boss_blue_quake_state(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    let Some((region_id, identity, tile_x, tile_y, state)) = game.find_player_mut(player_id).and_then(|player| {
        let state = player.take_expired_boss_blue_quake_state(now_ms)?;
        Some((player.server_region_id()?, player.shape().identity(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, state))
    }) else { return false };
    send_boss_blue_quake_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_skill_fightable(true);
    }
    true
}

pub(crate) fn finish_player_boss_blue_quake_state_on_cure(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    let Some((region_id, identity, tile_x, tile_y, state)) = game.find_player_mut(player_id).and_then(|player| {
        let state = player.take_boss_blue_quake_state()?;
        Some((player.server_region_id()?, player.shape().identity(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, state))
    }) else { return false };
    send_boss_blue_quake_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_skill_fightable(true);
    }
    true
}

pub(crate) fn expire_monster_boss_blue_quake_state(
    game: &mut CGame,
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    let Some((identity, tile_x, tile_y, state)) = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_expired_boss_blue_quake_state(now_ms)?;
        Some((monster.move_shape().shape().identity(), monster.move_shape().shape().get_tile_x().ok()?, monster.move_shape().shape().get_tile_y().ok()?, state))
    }) else { return false };
    send_boss_blue_quake_state_visual(game, region.id, identity, tile_x, tile_y, state, false, now_ms);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(true);
        monster.move_shape_mut().set_fightable(true);
    }
    true
}
