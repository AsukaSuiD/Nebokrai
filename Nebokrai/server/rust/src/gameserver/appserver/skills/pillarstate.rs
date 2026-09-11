//! Каноническое состояние стойки `CPillarState` (`0x74`).
//! AI из vtable `0x00660834 +0x0c` (`0x005d60b0`) сравнивает абсолютный
//! wrapping deadline строго с now, без особого исключения для нулевого срока.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/pillarstate.cpp`. Состояние хранит коэффициент поздней
//! защиты и строгий срок, публикует `0xBFE03/0xBFE04`. Проверка запрета рывков
//! и `PostDefense` читают типизированный экземпляр из общей арены владельца.
//! Общая с `CBossBlueFuryState` serializer-пара `0x005E7330/0x005D6190`
//! сохраняет 12 байт: `ID + remaining time + IEEE-754 factor bits`;
//! spatial login восстанавливает срок и вложенный запрет движения.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! AI/End разрешают общий CMoveShape по region/type/id; RTTI-ограничения
//! формул игрока не запрещают жизненный цикл региональных держателей.
//! После visual владелец перечитывается; UpdateProperty вызывается только
//! для игрока и только при фактическом удалении этой записи.
//! Exact End 0x005FB800: visual → GetSufferer → SetMoveable(true) → RemoveState.
//! Загрузка добавляет вложенный запрет движения для каждого экземпляра.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const PILLAR_STATE_ID: u32 = 0x74;
pub(crate) const PILLAR_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PillarState { started_at_ms: u32, keep_time_ms: u32, damage_factor_bits: u32 }

impl PillarState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, damage_factor: f32) -> Self {
        Self { started_at_ms, keep_time_ms, damage_factor_bits: damage_factor.to_bits() }
    }
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != PILLAR_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let remaining = reader.read_u32()?;
        Ok(Self { started_at_ms: 0, keep_time_ms: remaining, damage_factor_bits: reader.read_u32()? })
    }
    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self { self.started_at_ms = now_ms; self }
    pub(crate) fn encoded_for_install(self) -> [u8; PILLAR_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; PILLAR_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; PILLAR_STATE_BYTES] {
        let mut bytes = [0; PILLAR_STATE_BYTES];
        bytes[..4].copy_from_slice(&PILLAR_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining.to_le_bytes());
        bytes[8..].copy_from_slice(&self.damage_factor_bits.to_le_bytes());
        bytes
    }
    pub(crate) const fn skill_id(self) -> u32 { PILLAR_STATE_ID }
    pub(crate) const fn damage_factor(self) -> f32 { f32::from_bits(self.damage_factor_bits) }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
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

pub(crate) fn update_pillar_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key))
        .filter(|state| state.expired(now_ms)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder).and_then(|shape| {
        shape.set_moveable(true);
        shape.remove_applied_state_record::<PillarState>(key, PILLAR_STATE_BYTES)
    }).is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    true
}
