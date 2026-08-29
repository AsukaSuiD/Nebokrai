//! Каноническое периодическое состояние `CLeafCutState` (`0x6B`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/leafcutstate.cpp`. Состояние хранит снимок владельца и
//! атакующих свойств в момент применения, читает часы отдельно для срока жизни
//! и частоты, выполняет ровно два вызова MSVCRT RNG на атакующий тик и сохраняет
//! усечение `float` к нулю. DB-запись длиной 68 байт принадлежит этому типу;
//! `CanonicalStateStorage` атомарно поддерживает её смещение и жизненный цикл.
//! Tick извлекается и возвращается здесь же до межвладельческого применения
//! уже рассчитанной атаки координатором `CGame`.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

pub(crate) const LEAF_CUT_STATE_ID: u32 = 0x6b;
pub(crate) const LEAF_CUT_STATE_BYTES: usize = 68;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;
const LEGACY_UNKNOWN_SKILL_ID: u32 = i32::MAX as u32;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LeafCutStateTick { Pending, Attack(AttackInformation), Ended }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeafCutState {
    state_id: u32,
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    damage_factor_bits: u32,
    damage_modifier_bits: u32,
    minimum_attack: u16,
    maximum_attack: u16,
    element_attack: u16,
    soul_attack: u16,
    attack_count: u32,
    serialized_offset: Option<usize>,
}

impl LeafCutState {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют состоянию EXE")]
    pub(crate) const fn new(master: MasterInfo, started_at_ms: u32, keep_time_ms: u32, frequency_ms: u32, damage_factor: f32, damage_modifier: f32, minimum_attack: u16, maximum_attack: u16, element_attack: u16, soul_attack: u16) -> Self {
        Self::new_with_id(LEAF_CUT_STATE_ID, master, started_at_ms, keep_time_ms, frequency_ms, damage_factor, damage_modifier, minimum_attack, maximum_attack, element_attack, soul_attack)
    }

    #[allow(clippy::too_many_arguments, reason = "общий layout двух подтверждённых классов EXE")]
    pub(crate) const fn new_with_id(state_id: u32, master: MasterInfo, started_at_ms: u32, keep_time_ms: u32, frequency_ms: u32, damage_factor: f32, damage_modifier: f32, minimum_attack: u16, maximum_attack: u16, element_attack: u16, soul_attack: u16) -> Self {
        Self { state_id, master, started_at_ms, keep_time_ms, frequency_ms, damage_factor_bits: damage_factor.to_bits(), damage_modifier_bits: damage_modifier.to_bits(), minimum_attack, maximum_attack, element_attack, soul_attack, attack_count: 0, serialized_offset: None }
    }

    pub(crate) const fn skill_id(self) -> u32 { self.state_id }
    pub(crate) const fn master(self) -> MasterInfo { self.master }
    pub(crate) const fn serialized_span(self) -> Option<(usize, usize)> { match self.serialized_offset { Some(offset) => Some((offset, LEAF_CUT_STATE_BYTES)), None => None } }
    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) { if self.serialized_offset.is_some_and(|offset| removed_offset < offset) { self.serialized_offset = self.serialized_offset.map(|offset| offset - amount); } }
    pub(crate) const fn client_time(self, now_ms: u32) -> i32 { let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms); if deadline <= now_ms { 0 } else { deadline.wrapping_sub(now_ms) as i32 } }
    pub(crate) fn activate_loaded(&mut self, now_ms: u32) { self.started_at_ms = now_ms; self.attack_count = 0; }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        Self::decode_with_id(payload, offset, now_ms, LEAF_CUT_STATE_ID)
    }

    pub(crate) fn decode_with_id(payload: &[u8], offset: usize, now_ms: u32, state_id: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != state_id { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); }
        let mut values = [0i32; 10]; for value in &mut values { *value = reader.read_i32()?; }
        Ok(Self { state_id, master: MasterInfo { master_type: values[0], master_id: values[1], master_guild_id: values[2], master_team_id: values[3], master_union_id: values[4], master_country_id: values[5], permitted_to_kill_player: values[6], permitted_to_kill_teammate: values[7], permitted_to_kill_guild_member: values[8], permitted_to_kill_criminal: values[9] }, started_at_ms: now_ms, keep_time_ms: reader.read_u32()?, frequency_ms: reader.read_u32()?, damage_factor_bits: reader.read_u32()?, damage_modifier_bits: reader.read_u32()?, minimum_attack: reader.read_u16()?, maximum_attack: reader.read_u16()?, element_attack: reader.read_u16()?, soul_attack: reader.read_u16()?, attack_count: 0, serialized_offset: Some(offset) })
    }

    pub(crate) fn append_serialized(&mut self, payload: &mut Vec<u8>, now_ms: u32) {
        let offset = payload.len(); let record = self.encoded(now_ms); payload.extend_from_slice(&record); self.serialized_offset = Some(offset);
    }

    pub(crate) fn write_serialized_at(&mut self, payload: &mut [u8], offset: usize, now_ms: u32) -> bool {
        let Some(destination) = payload.get_mut(offset..offset.saturating_add(LEAF_CUT_STATE_BYTES)) else { return false }; destination.copy_from_slice(&self.encoded(now_ms)); self.serialized_offset = Some(offset); true
    }

    fn encoded(self, now_ms: u32) -> Vec<u8> {
        let mut record = Vec::with_capacity(LEAF_CUT_STATE_BYTES); let mut writer = LegacyWriter::new(&mut record); writer.write_u32(self.state_id);
        for value in [self.master.master_type, self.master.master_id, self.master.master_guild_id, self.master.master_team_id, self.master.master_union_id, self.master.master_country_id, self.master.permitted_to_kill_player, self.master.permitted_to_kill_teammate, self.master.permitted_to_kill_guild_member, self.master.permitted_to_kill_criminal] { writer.write_i32(value); }
        writer.write_u32(self.remaining_time(now_ms)); writer.write_u32(self.frequency_ms); writer.write_u32(self.damage_factor_bits); writer.write_u32(self.damage_modifier_bits); writer.write_u16(self.minimum_attack); writer.write_u16(self.maximum_attack); writer.write_u16(self.element_attack); writer.write_u16(self.soul_attack); record
    }

    pub(crate) fn update_serialized_runtime(self, payload: &mut [u8], now_ms: u32) { if let Some(offset) = self.serialized_offset { let _ = LegacyWriter::write_u32_at(payload, offset + 44, self.remaining_time(now_ms)); } }
    fn remaining_time(self, now_ms: u32) -> u32 { let elapsed = now_ms.wrapping_sub(self.started_at_ms); if elapsed >= self.keep_time_ms { 0 } else { self.keep_time_ms.wrapping_sub(elapsed) } }

    pub(crate) fn tick(&mut self, lifetime_now_ms: u32, frequency_now_ms: u32, target_dead: bool, critical_chance: u16, critical_rate: f32, random: &mut dyn FnMut(i32) -> i32) -> LeafCutStateTick {
        if self.started_at_ms.wrapping_add(self.keep_time_ms) < lifetime_now_ms || target_dead { return LeafCutStateTick::Ended; }
        if self.started_at_ms.wrapping_add(self.frequency_ms.wrapping_mul(self.attack_count)) >= frequency_now_ms { return LeafCutStateTick::Pending; }
        self.attack_count = self.attack_count.wrapping_add(1); LeafCutStateTick::Attack(self.attack(critical_chance, critical_rate, random))
    }

    fn attack(self, critical_chance: u16, critical_rate: f32, random: &mut dyn FnMut(i32) -> i32) -> AttackInformation {
        let minimum = i32::from(self.minimum_attack); let maximum = i32::from(self.maximum_attack); let span = minimum.abs_diff(maximum).wrapping_add(1) as i32; let mut physical = (random(span) as f32 + f32::from_bits(self.damage_modifier_bits) + minimum as f32) as i32; if physical < 0 { physical = 0; }
        let critical = random(100) < i32::from(critical_chance); let mut damages = vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: i32::from(self.element_attack), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(self.soul_attack), mp_damage: 0 }]; if critical { for damage in &mut damages { damage.hp_damage = (damage.hp_damage as f32 * critical_rate) as i32; } }
        AttackInformation { skill_id: LEGACY_UNKNOWN_SKILL_ID, skill_level: 0, attacker_type: self.master.master_type, attacker_id: self.master.master_id, attacker_team_id: self.master.master_team_id, attacker_faction_id: self.master.master_guild_id, attacker_union_id: self.master.master_union_id, hit_modifier: 0, damage_factor: f32::from_bits(self.damage_factor_bits), damage_modifier: 0, critical, blast_attack: false, full_miss: 0, damages }
    }
}

pub(crate) fn send_leaf_cut_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: LeafCutState, begin: bool, now_ms: u32) {
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(state.skill_id() as i32); if begin { message.add_long(state.client_time(now_ms)); message.add_long(0); } let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn update_player_leaf_cut_state<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, runtime: &mut Runtime) -> bool {
    let target = game.find_player_mut(player_id).and_then(|player| {
        let identity = player.shape().identity(); let x = player.shape().get_tile_x().ok()?; let y = player.shape().get_tile_y().ok()?;
        let region_id = player.server_region_id()?; let dead = player.is_dead(); let critical_chance = player.combat_properties().cch;
        let state = player.take_leaf_cut_state_for_ai()?; Some((state, identity, x, y, region_id, dead, critical_chance))
    });
    let Some((mut state, identity, x, y, region_id, dead, critical_chance)) = target else { return false };
    let lifetime_now = runtime.now_milliseconds(); let frequency_now = runtime.now_milliseconds(); let critical_rate = game.globe_setup().critical_rate();
    let tick = state.tick(lifetime_now, frequency_now, dead, critical_chance, critical_rate, &mut |maximum| game.skill_random_below(maximum));
    match tick {
        LeafCutStateTick::Pending => if let Some(player) = game.find_player_mut(player_id) { player.restore_leaf_cut_state_after_ai(state); },
        LeafCutStateTick::Attack(attack) => { let master = state.master(); if let Some(player) = game.find_player_mut(player_id) { player.restore_leaf_cut_state_after_ai(state); } game.apply_owned_skill_attack_to_player(master, player_id, region_id, attack, runtime); }
        LeafCutStateTick::Ended => { if let Some(player) = game.find_player_mut(player_id) { player.finish_leaf_cut_state(state); } send_leaf_cut_state_visual(game, region_id, identity, x, y, state, false, lifetime_now); let _ = game.publish_player_states(player_id); }
    }
    true
}

pub(crate) fn update_monster_leaf_cut_state<Runtime: GameMainLoopRuntime>(game: &mut CGame, region_id: i32, monster_id: i32, runtime: &mut Runtime) -> bool {
    let Some(mut owner) = game.take_region_owner(region_id) else { return false };
    let target = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
        let shape = monster.move_shape().shape(); let identity = shape.identity(); let x = shape.get_tile_x().ok()?; let y = shape.get_tile_y().ok()?;
        let dead = monster.hit_points() == 0; let state = monster.move_shape_mut().take_leaf_cut_state_for_ai()?; Some((state, identity, x, y, dead))
    });
    game.restore_region_owner(owner);
    let Some((mut state, identity, x, y, dead)) = target else { return false };
    let lifetime_now = runtime.now_milliseconds(); let frequency_now = runtime.now_milliseconds(); let critical_rate = game.globe_setup().critical_rate();
    let tick = state.tick(lifetime_now, frequency_now, dead, 0, critical_rate, &mut |maximum| game.skill_random_below(maximum));
    match tick {
        LeafCutStateTick::Pending => if let Some(mut owner) = game.take_region_owner(region_id) { if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().restore_leaf_cut_state_after_ai(state); } game.restore_region_owner(owner); },
        LeafCutStateTick::Attack(attack) => { let master = state.master(); if let Some(mut owner) = game.take_region_owner(region_id) { if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().restore_leaf_cut_state_after_ai(state); } game.restore_region_owner(owner); } game.apply_owned_skill_attack_to_monster(master, monster_id, region_id, attack, runtime); }
        LeafCutStateTick::Ended => { if let Some(mut owner) = game.take_region_owner(region_id) { if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().finish_leaf_cut_state(state); } game.restore_region_owner(owner); } send_leaf_cut_state_visual(game, region_id, identity, x, y, state, false, lifetime_now); }
    }
    true
}
