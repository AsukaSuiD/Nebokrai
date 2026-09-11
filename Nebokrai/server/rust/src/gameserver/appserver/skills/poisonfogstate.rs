//! Каноническое ослабление ядовитого тумана `CPoisonFogState` (`0xC9`).
//! Истечение получает ключ конкретного экземпляра общей арены; проверка
//! срока и End не подменяют его первым состоянием с тем же ID.
//! Direct End (vtable 0x006622D4 +0x1C, тело 0x005FD420) отправляет
//! visual до точного RemoveState и последующего UpdateProperty игрока.
//! Timer и Cure используют ту же опубликованную generic holder границу;
//! удаление использует фактический serialized_span выбранного экземпляра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/poisonfogstate.cpp`. Сохранены 36-байтовая DB-запись,
//! строгая граница срока и разные legacy-преобразования свойств игрока и
//! монстра. Поле `dodge_loss` сохраняется в двоичной записи, хотя достигнутый
//! русский вариант не читает его ни в одной ветви: обе цели теряют defense и
//! element resistance.
//! Для монстра разница уровней сначала сохраняется во `float`, а произведение
//! defense остаётся в x87 до усечения; произведение сопротивления сохраняется
//! во `float`. Для игрока разница остаётся в x87, оба произведения сохраняются
//! во `float`, ограничиваются текущим свойством и после усечения сужаются до
//! младших 16 бит.
//! Собственный `GetRemainedTime` по `0x00607E00` сохраняет условное второе
//! чтение wrapping clock; DB-кодек по-прежнему принимает единый sampled tick.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::monster::MonsterCombatProperties;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::fightdefense::truncate_original;
use crate::gameserver::appserver::states::state::{
    timed_client_state_time, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

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
    pub(crate) fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        if let Some(offset) = &mut self.serialized_offset {
            if *offset >= inserted_offset {
                *offset += amount;
            }
        }
    }

    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) { if self.serialized_offset.is_some_and(|offset| removed_offset < offset) { self.serialized_offset = self.serialized_offset.map(|offset| offset - amount); } }
    pub(crate) fn activate_loaded(&mut self, now_ms: u32) { self.started_at_ms = now_ms; }
    fn player_loss(self, target_level: u8, coefficient: u32, maximum: u32, current: u32) -> u32 {
        let scaled = if coefficient == 0 {
            0.0
        } else {
            let difference = f64::from(self.weapon_damage_level) - f64::from(target_level);
            let ratio = (difference / f64::from(coefficient)).clamp(0.0, 1.0);
            (f64::from(maximum) * ratio) as f32
        };
        let capped = if f64::from(current) < f64::from(scaled) {
            f64::from(current) as f32
        } else {
            scaled
        };
        truncate_original(f64::from(capped)) as u32 & 0xffff
    }

    fn monster_losses(self, target_level: u8) -> (u32, u32) {
        let difference = (f64::from(self.weapon_damage_level) - f64::from(target_level)) as f32;
        let ratio = |coefficient: u32| {
            if coefficient == 0 {
                0.0
            } else {
                (f64::from(difference) / f64::from(coefficient)).clamp(0.0, 1.0)
            }
        };
        let defense = truncate_original(
            f64::from(self.defense_loss) * ratio(self.defense_loss_coefficient),
        ) as u32;
        let resistance = (f64::from(self.element_resistance_loss)
            * ratio(self.element_resistance_loss_coefficient)) as f32;
        (defense, truncate_original(f64::from(resistance)) as u32)
    }

    pub(crate) fn apply_to_player(self, target_level: u8, mut properties: PlayerCombatProperties) -> PlayerCombatProperties {
        let defense = self.player_loss(
            target_level,
            self.defense_loss_coefficient,
            self.defense_loss,
            properties.defense,
        );
        let resistance = self.player_loss(
            target_level,
            self.element_resistance_loss_coefficient,
            self.element_resistance_loss,
            properties.element_resistance,
        );
        properties.defense = properties.defense.wrapping_sub(defense).min(i32::MAX as u32);
        properties.element_resistance = properties.element_resistance.wrapping_sub(resistance).min(i32::MAX as u32);
        properties
    }

    pub(crate) fn apply_to_monster(self, mut properties: MonsterCombatProperties) -> MonsterCombatProperties {
        let (defense, resistance) = self.monster_losses(properties.level);
        properties.defense = properties.defense.wrapping_sub(defense);
        properties.element_resistance = properties.element_resistance.wrapping_sub(resistance);
        properties
    }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> { let mut reader = LegacyReader::at(payload, offset)?; if reader.read_u32()? != POISON_FOG_STATE_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); } Ok(Self { skill_level: reader.read_i32()?, started_at_ms: now_ms, keep_time_ms: reader.read_u32()?, defense_loss: reader.read_u32()?, defense_loss_coefficient: reader.read_u32()?, dodge_loss: reader.read_u32()?, element_resistance_loss: reader.read_u32()?, element_resistance_loss_coefficient: reader.read_u32()?, weapon_damage_level: reader.read_u32()?, serialized_offset: Some(offset) }) }
    fn encoded(self, now_ms: u32) -> Vec<u8> { let mut record = Vec::with_capacity(POISON_FOG_STATE_BYTES); let mut writer = LegacyWriter::new(&mut record); writer.write_u32(POISON_FOG_STATE_ID); writer.write_i32(self.skill_level); writer.write_u32(self.sampled_remaining_time(now_ms)); writer.write_u32(self.defense_loss); writer.write_u32(self.defense_loss_coefficient); writer.write_u32(self.dodge_loss); writer.write_u32(self.element_resistance_loss); writer.write_u32(self.element_resistance_loss_coefficient); writer.write_u32(self.weapon_damage_level); record }
    pub(crate) fn append_serialized(&mut self, payload: &mut Vec<u8>, now_ms: u32) { let offset = payload.len(); payload.extend_from_slice(&self.encoded(now_ms)); self.serialized_offset = Some(offset); }
    pub(crate) fn write_serialized_at(&mut self, payload: &mut [u8], offset: usize, now_ms: u32) -> bool { let Some(destination) = payload.get_mut(offset..offset.saturating_add(POISON_FOG_STATE_BYTES)) else { return false }; destination.copy_from_slice(&self.encoded(now_ms)); self.serialized_offset = Some(offset); true }
    pub(crate) fn update_serialized_runtime(self, payload: &mut [u8], now_ms: u32) { if let Some(offset) = self.serialized_offset { let _ = LegacyWriter::write_u32_at(payload, offset + 8, self.sampled_remaining_time(now_ms)); } }
}

pub(crate) fn send_poison_fog_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: PoisonFogState, begin: bool, now_ms: u32) { let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(POISON_FOG_STATE_ID as i32); if begin { message.add_long(state.client_time(|| now_ms)); message.add_long(0); } let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message); }

pub(crate) fn update_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now_ms: u32,
) -> bool {
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key))
        .is_some_and(|state| state.expired(now_ms));
    if !expired { return false }
    end_poison_fog_state(game, region_id, holder, key)
}

pub(crate) fn end_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    let mut message = CMessage::new(STATE_END_MESSAGE);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(POISON_FOG_STATE_ID as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<PoisonFogState>(
            key, POISON_FOG_STATE_BYTES,
        )).is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    removed
}
