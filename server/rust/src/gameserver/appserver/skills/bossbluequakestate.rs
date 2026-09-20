//! Блокировка землетрясения CBossBlueQuakeState (0x1f8).
//! Источник: gameserver.exe + GameServer.pdb, исходный владелец
//! appserver/skills/bossbluequakestate.cpp.
//!
//! Объектный Begin использует общий Blind-адаптер: timestamp
//! меняется только при U; loop1/Update0 предшествует move/fight-lock и записи
//! в арену. Restart передаёт NULL U и holder, сохраняя payload, срок и ключ;
//! после загрузки каждый экземпляр добавляет обе вложенные блокировки.
//! End: visual → S → fight-unlock → move-unlock → RemoveState
//! того же ключа, без чтения часов. Истечение использует строгий wrapping deadline.
//! GetRemainedTime читает часы второй раз при положительном остатке.
//! DB-запись ID/remaining занимает 8 байт; Load получает отдельный timestamp,
//! который Restart не заменяет. OnUpdateProperties возвращает 1
//! без изменения свойств, визуала или блокировок.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual, update_applied_state_visual_base,
};
use crate::gameserver::appserver::moveshape::StateKey;

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const BOSS_BLUE_QUAKE_STATE_ID: u32 = 0x1f8;
pub(crate) const BOSS_BLUE_QUAKE_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BossBlueQuakeState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl BossBlueQuakeState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != BOSS_BLUE_QUAKE_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }



    pub(crate) fn encoded_for_install(self) -> [u8; BOSS_BLUE_QUAKE_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; BOSS_BLUE_QUAKE_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(self, remaining: u32) -> [u8; BOSS_BLUE_QUAKE_STATE_BYTES] {
        let mut bytes = [0; BOSS_BLUE_QUAKE_STATE_BYTES];
        bytes[..4].copy_from_slice(&BOSS_BLUE_QUAKE_STATE_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&remaining.to_le_bytes());
        bytes
    }

    pub(crate) const fn skill_id(self) -> u32 { BOSS_BLUE_QUAKE_STATE_ID }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
}

impl super::blindstate::BlindStatePayload for BossBlueQuakeState {
    fn blind_state_id(&self) -> u32 { BOSS_BLUE_QUAKE_STATE_ID }
    fn begin_at(&mut self, now_ms: u32) { self.started_at_ms = now_ms; }
    fn remaining(&self, now: &mut dyn FnMut() -> u32) -> u32 { self.client_time(now) as u32 }
    fn install_record(&self) -> [u8; BOSS_BLUE_QUAKE_STATE_BYTES] { self.encoded_for_install() }
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
    now_milliseconds: impl FnMut() -> u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn restart_boss_blue_quake_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueQuakeState>(key)).copied()
        else { return false };
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.skill_id() as i32);
        message.add_long(state.client_time(now));
        message.add_long(0);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = update_applied_state_visual_base(game, region_id, holder, key);
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_moveable(false);
        shape.set_fightable(false);
    }
    true
}

pub(crate) fn update_boss_blue_quake_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueQuakeState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_boss_blue_quake_state(game, region_id, holder, key)
}

pub(crate) fn end_boss_blue_quake_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueQuakeState>(key)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| {
            shape.applied_state::<BossBlueQuakeState>(key)?;
            shape.set_fightable(true);
            shape.set_moveable(true);
            shape.remove_applied_state_record::<BossBlueQuakeState>(key, BOSS_BLUE_QUAKE_STATE_BYTES)
        }).is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}

pub(crate) fn finish_player_boss_blue_quake_state_on_cure(
    game: &mut CGame,
    player_id: i32,
    _now_ms: u32,
) -> bool {
    let Some((region_id, holder, key)) = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().identity(),
            player.move_shape().applied_state_key::<BossBlueQuakeState>()?))
    }) else { return false };
    end_boss_blue_quake_state(game, region_id, holder, key)
}
