//! Каноническое ослабление ядовитого тумана `CPoisonFogState` (`0xC9`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/poisonfogstate.cpp`. Сохранены 36-байтовая DB-запись,
//! строгая граница срока и legacy-преобразования через `float`. Поле
//! `dodge_loss` сохраняется в двоичной записи, хотя достигнутый русский вариант
//! использует для `CMonster` округлённую потерю defense, а для `CPlayer` это
//! поле не читает.
//! Собственный `GetRemainedTime` по `0x00607E00` сохраняет условное второе
//! чтение wrapping clock; DB-кодек по-прежнему принимает единый sampled tick.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::monster::MonsterCombatProperties;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const POISON_FOG_STATE_ID: u32 = 0xc9;
pub(crate) const POISON_FOG_STATE_BYTES: usize = 36;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PoisonFogState {
    skill_level: i32, started_at_ms: u32, keep_time_ms: u32,
    defense_loss: u32, defense_loss_coefficient: u32, dodge_loss: u32,
    element_resistance_loss: u32, element_resistance_loss_coefficient: u32,
    weapon_damage_level: u32, serialized_offset: Option<usize>,
}

impl PoisonFogState {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют состоянию EXE")]
    pub(crate) const fn new(skill_level: i32, started_at_ms: u32, keep_time_ms: u32, defense_loss: u32, defense_loss_coefficient: u32, dodge_loss: u32, element_resistance_loss: u32, element_resistance_loss_coefficient: u32, weapon_damage_level: u32) -> Self { Self { skill_level, started_at_ms, keep_time_ms, defense_loss, defense_loss_coefficient, dodge_loss, element_resistance_loss, element_resistance_loss_coefficient, weapon_damage_level, serialized_offset: None } }
    pub(crate) const fn skill_id(self) -> u32 { POISON_FOG_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32 }
    const fn sampled_remaining_time(self, now_ms: u32) -> u32 { let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms); if deadline <= now_ms { 0 } else { deadline.wrapping_sub(now_ms) } }
    pub(crate) const fn serialized_span(self) -> Option<(usize, usize)> { match self.serialized_offset { Some(offset) => Some((offset, POISON_FOG_STATE_BYTES)), None => None } }
    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) { if self.serialized_offset.is_some_and(|offset| removed_offset < offset) { self.serialized_offset = self.serialized_offset.map(|offset| offset - amount); } }
    pub(crate) fn activate_loaded(&mut self, now_ms: u32) { self.started_at_ms = now_ms; }
    fn scaled_loss(self, target_level: u8, coefficient: u32, maximum: u32) -> f32 { if coefficient == 0 { return 0.0; } maximum as f32 * ((self.weapon_damage_level as f32 - f32::from(target_level)) / coefficient as f32).clamp(0.0, 1.0) }
    pub(crate) fn apply_to_player(self, target_level: u8, mut properties: PlayerCombatProperties) -> PlayerCombatProperties { let defense = self.scaled_loss(target_level, self.defense_loss_coefficient, self.defense_loss) as u32 & 0xffff; let resistance = self.scaled_loss(target_level, self.element_resistance_loss_coefficient, self.element_resistance_loss) as u32 & 0xffff; properties.defense = properties.defense.wrapping_sub(defense).min(i32::MAX as u32); properties.element_resistance = properties.element_resistance.wrapping_sub(resistance).min(i32::MAX as u32); properties }
    pub(crate) fn apply_to_monster(self, mut properties: MonsterCombatProperties) -> MonsterCombatProperties { let loss = self.scaled_loss(properties.level, self.defense_loss_coefficient, self.defense_loss).round_ties_even() as u32; properties.defense = properties.defense.wrapping_sub(loss); properties.dodge = properties.dodge.wrapping_sub(loss); properties }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> { let mut reader = LegacyReader::at(payload, offset)?; if reader.read_u32()? != POISON_FOG_STATE_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); } Ok(Self { skill_level: reader.read_i32()?, started_at_ms: now_ms, keep_time_ms: reader.read_u32()?, defense_loss: reader.read_u32()?, defense_loss_coefficient: reader.read_u32()?, dodge_loss: reader.read_u32()?, element_resistance_loss: reader.read_u32()?, element_resistance_loss_coefficient: reader.read_u32()?, weapon_damage_level: reader.read_u32()?, serialized_offset: Some(offset) }) }
    fn encoded(self, now_ms: u32) -> Vec<u8> { let mut record = Vec::with_capacity(POISON_FOG_STATE_BYTES); let mut writer = LegacyWriter::new(&mut record); writer.write_u32(POISON_FOG_STATE_ID); writer.write_i32(self.skill_level); writer.write_u32(self.sampled_remaining_time(now_ms)); writer.write_u32(self.defense_loss); writer.write_u32(self.defense_loss_coefficient); writer.write_u32(self.dodge_loss); writer.write_u32(self.element_resistance_loss); writer.write_u32(self.element_resistance_loss_coefficient); writer.write_u32(self.weapon_damage_level); record }
    pub(crate) fn append_serialized(&mut self, payload: &mut Vec<u8>, now_ms: u32) { let offset = payload.len(); payload.extend_from_slice(&self.encoded(now_ms)); self.serialized_offset = Some(offset); }
    pub(crate) fn write_serialized_at(&mut self, payload: &mut [u8], offset: usize, now_ms: u32) -> bool { let Some(destination) = payload.get_mut(offset..offset.saturating_add(POISON_FOG_STATE_BYTES)) else { return false }; destination.copy_from_slice(&self.encoded(now_ms)); self.serialized_offset = Some(offset); true }
    pub(crate) fn update_serialized_runtime(self, payload: &mut [u8], now_ms: u32) { if let Some(offset) = self.serialized_offset { let _ = LegacyWriter::write_u32_at(payload, offset + 8, self.sampled_remaining_time(now_ms)); } }
}

pub(crate) fn send_poison_fog_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: PoisonFogState, begin: bool, now_ms: u32) { let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(POISON_FOG_STATE_ID as i32); if begin { message.add_long(state.client_time(|| now_ms)); message.add_long(0); } let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message); }

pub(crate) fn expire_player_poison_fog_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
    _runtime: &mut Runtime,
) -> bool {
    let removed = game.find_player_mut(player_id).and_then(|player| {
        let region_id = player.server_region_id()?;
        let tile_x = player.shape().get_tile_x().ok()?;
        let tile_y = player.shape().get_tile_y().ok()?;
        let state = player.take_expired_poison_fog_state(now_ms)?;
        Some((region_id, tile_x, tile_y, state))
    });
    let Some((region_id, tile_x, tile_y, state)) = removed else {
        return false;
    };
    send_poison_fog_state_visual(
        game,
        region_id,
        ShapeIdentity {
            object_type: 400,
            id: player_id,
            ex_id: CGuid::GUID_INVALID,
        },
        tile_x,
        tile_y,
        state,
        false,
        now_ms,
    );
    let _ = game.update_player_properties(player_id);
    true
}

pub(crate) struct PoisonFogStateExpiration {
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: PoisonFogState,
}

impl PoisonFogStateExpiration {
    /// Доставка выполняется после возвращения region-owner-а в `CGame`, как в
    /// достигнутом общем такте состояний монстра.
    pub(crate) fn deliver(self, game: &mut CGame, region_id: i32, now_ms: u32) {
        send_poison_fog_state_visual(
            game,
            region_id,
            self.identity,
            self.tile_x,
            self.tile_y,
            self.state,
            false,
            now_ms,
        );
    }
}

pub(crate) fn take_expired_monster_poison_fog_state(
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> Option<PoisonFogStateExpiration> {
    region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let tile_x = monster.move_shape().shape().get_tile_x().ok()?;
        let tile_y = monster.move_shape().shape().get_tile_y().ok()?;
        let identity = monster.move_shape().shape().identity();
        let state = monster
            .move_shape_mut()
            .take_expired_poison_fog_state(now_ms)?;
        Some(PoisonFogStateExpiration {
            identity,
            tile_x,
            tile_y,
            state,
        })
    })
}
