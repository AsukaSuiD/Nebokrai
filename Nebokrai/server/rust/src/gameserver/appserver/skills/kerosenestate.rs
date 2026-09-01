//! Каноническое периодическое состояние горючей смеси `CKeroseneState` (`0xF1`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/kerosenestate.cpp`. Состояние хранит снимок владельца,
//! использует строгие границы срока и частоты и наносит один фиксированный
//! урон типа `Poison` без RNG. DB-запись длиной 56 байт принадлежит этому типу;
//! `CanonicalStateStorage` атомарно поддерживает её смещение и жизненный цикл,
//! включая извлечение и возврат перед межвладельческим применением удара.
//! Встроенная `tagAttackInformation` сохраняет конструкторские skill-id
//! `0x7fffffff` и уровень `1`: очистка между тиками уровень не перезаписывает.
//! Клиентский срок разделяет точное тело `0x00606320` с остальными
//! периодическими состояниями и использует два чтения wrapping clock.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

pub(crate) const KEROSENE_STATE_ID: u32 = 0xf1;
pub(crate) const KEROSENE_STATE_BYTES: usize = 56;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;
const DEFAULT_PERIODIC_SKILL_ID: u32 = i32::MAX as u32;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum KeroseneStateTick { Pending, Attack(AttackInformation), Ended }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KeroseneState {
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    hp_loss: u32,
    attack_count: u32,
    serialized_offset: Option<usize>,
}

impl KeroseneState {
    pub(crate) const fn new(master: MasterInfo, started_at_ms: u32, keep_time_ms: u32, frequency_ms: u32, hp_loss: u32) -> Self { Self { master, started_at_ms, keep_time_ms, frequency_ms, hp_loss, attack_count: 0, serialized_offset: None } }
    pub(crate) const fn skill_id(self) -> u32 { KEROSENE_STATE_ID }
    pub(crate) const fn master(self) -> MasterInfo { self.master }
    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> { let mut reader = LegacyReader::at(payload, offset)?; if reader.read_u32()? != KEROSENE_STATE_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); } let mut values = [0i32; 10]; for value in &mut values { *value = reader.read_i32()?; } Ok(Self { master: MasterInfo { master_type: values[0], master_id: values[1], master_guild_id: values[2], master_team_id: values[3], master_union_id: values[4], master_country_id: values[5], permitted_to_kill_player: values[6], permitted_to_kill_teammate: values[7], permitted_to_kill_guild_member: values[8], permitted_to_kill_criminal: values[9] }, started_at_ms: now_ms, keep_time_ms: reader.read_u32()?, frequency_ms: reader.read_u32()?, hp_loss: reader.read_u32()?, attack_count: 0, serialized_offset: Some(offset) }) }
    pub(crate) fn append_serialized(&mut self, payload: &mut Vec<u8>, now_ms: u32) { let offset = payload.len(); payload.extend_from_slice(&self.encoded(now_ms)); self.serialized_offset = Some(offset); }
    pub(crate) fn write_serialized_at(&mut self, payload: &mut [u8], offset: usize, now_ms: u32) -> bool { let Some(destination) = payload.get_mut(offset..offset.saturating_add(KEROSENE_STATE_BYTES)) else { return false }; destination.copy_from_slice(&self.encoded(now_ms)); self.serialized_offset = Some(offset); true }
    fn encoded(self, now_ms: u32) -> Vec<u8> { let mut record = Vec::with_capacity(KEROSENE_STATE_BYTES); let mut writer = LegacyWriter::new(&mut record); writer.write_u32(KEROSENE_STATE_ID); for value in [self.master.master_type, self.master.master_id, self.master.master_guild_id, self.master.master_team_id, self.master.master_union_id, self.master.master_country_id, self.master.permitted_to_kill_player, self.master.permitted_to_kill_teammate, self.master.permitted_to_kill_guild_member, self.master.permitted_to_kill_criminal] { writer.write_i32(value); } writer.write_u32(self.remaining_time(now_ms)); writer.write_u32(self.frequency_ms); writer.write_u32(self.hp_loss); record }
    pub(crate) fn update_serialized_runtime(self, payload: &mut [u8], now_ms: u32) { if let Some(offset) = self.serialized_offset { let _ = LegacyWriter::write_u32_at(payload, offset + 44, self.remaining_time(now_ms)); } }
    pub(crate) fn activate_loaded(&mut self, now_ms: u32) { self.started_at_ms = now_ms; self.attack_count = 0; }
    pub(crate) const fn serialized_span(self) -> Option<(usize, usize)> { match self.serialized_offset { Some(offset) => Some((offset, KEROSENE_STATE_BYTES)), None => None } }
    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) { if self.serialized_offset.is_some_and(|offset| removed_offset < offset) { self.serialized_offset = self.serialized_offset.map(|offset| offset - amount); } }
    fn remaining_time(self, now_ms: u32) -> u32 { let elapsed = now_ms.wrapping_sub(self.started_at_ms); if elapsed >= self.keep_time_ms { 0 } else { self.keep_time_ms.wrapping_sub(elapsed) } }
    pub(crate) fn ended(self, lifetime_now_ms: u32, target_dead: bool) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < lifetime_now_ms || target_dead }
    pub(crate) fn tick(&mut self, frequency_now_ms: u32) -> KeroseneStateTick { if self.started_at_ms.wrapping_add(self.frequency_ms.wrapping_mul(self.attack_count)) >= frequency_now_ms { return KeroseneStateTick::Pending; } self.attack_count = self.attack_count.wrapping_add(1); KeroseneStateTick::Attack(AttackInformation { skill_id: DEFAULT_PERIODIC_SKILL_ID, skill_level: 1, attacker_type: self.master.master_type, attacker_id: self.master.master_id, attacker_team_id: self.master.master_team_id, attacker_faction_id: self.master.master_guild_id, attacker_union_id: self.master.master_union_id, hit_modifier: 0, damage_factor: 1.0, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Poison, hp_damage: self.hp_loss as i32, mp_damage: 0 }] }) }
}

pub(crate) fn send_kerosene_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: KeroseneState, begin: bool, now_ms: u32) { let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(KEROSENE_STATE_ID as i32); if begin { message.add_ulong(state.client_state_time(|| now_ms)); message.add_long(0); } let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message); }

pub(crate) fn update_player_kerosene_state<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, runtime: &mut Runtime) -> bool {
    let target = game.find_player_mut(player_id).and_then(|player| { let identity = player.shape().identity(); let x = player.shape().get_tile_x().ok()?; let y = player.shape().get_tile_y().ok()?; let region_id = player.server_region_id()?; let dead = player.is_dead(); let state = player.take_kerosene_state_for_ai()?; Some((state, identity, x, y, region_id, dead)) });
    let Some((mut state, identity, x, y, region_id, dead)) = target else { return false }; let lifetime_now = runtime.now_milliseconds();
    let tick = if state.ended(lifetime_now, dead) { KeroseneStateTick::Ended } else { state.tick(runtime.now_milliseconds()) };
    match tick {
        KeroseneStateTick::Pending => if let Some(player) = game.find_player_mut(player_id) { player.restore_kerosene_state_after_ai(state); },
        KeroseneStateTick::Attack(attack) => { let master = state.master(); if let Some(player) = game.find_player_mut(player_id) { player.restore_kerosene_state_after_ai(state); } game.apply_owned_skill_attack_to_player(master, player_id, region_id, attack, runtime); }
        KeroseneStateTick::Ended => { if let Some(player) = game.find_player_mut(player_id) { player.finish_kerosene_state(state); } send_kerosene_state_visual(game, region_id, identity, x, y, state, false, lifetime_now); let _ = game.publish_player_states(player_id); }
    }
    true
}

pub(crate) fn update_monster_kerosene_state<Runtime: GameMainLoopRuntime>(game: &mut CGame, region_id: i32, monster_id: i32, runtime: &mut Runtime) -> bool {
    let Some(mut owner) = game.take_region_owner(region_id) else { return false }; let target = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| { let identity = monster.move_shape().shape().identity(); let x = monster.move_shape().shape().get_tile_x().ok()?; let y = monster.move_shape().shape().get_tile_y().ok()?; let dead = monster.hit_points() == 0; let state = monster.move_shape_mut().take_kerosene_state_for_ai()?; Some((state, identity, x, y, dead)) }); game.restore_region_owner(owner);
    let Some((mut state, identity, x, y, dead)) = target else { return false }; let lifetime_now = runtime.now_milliseconds(); let tick = if state.ended(lifetime_now, dead) { KeroseneStateTick::Ended } else { state.tick(runtime.now_milliseconds()) };
    match tick {
        KeroseneStateTick::Pending => if let Some(mut owner) = game.take_region_owner(region_id) { if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().restore_kerosene_state_after_ai(state); } game.restore_region_owner(owner); },
        KeroseneStateTick::Attack(attack) => { let master = state.master(); if let Some(mut owner) = game.take_region_owner(region_id) { if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().restore_kerosene_state_after_ai(state); } game.restore_region_owner(owner); } game.apply_owned_skill_attack_to_monster(master, monster_id, region_id, attack, runtime); }
        KeroseneStateTick::Ended => { if let Some(mut owner) = game.take_region_owner(region_id) { if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().finish_kerosene_state(state); } game.restore_region_owner(owner); } send_kerosene_state_visual(game, region_id, identity, x, y, state, false, lifetime_now); }
    }
    true
}
