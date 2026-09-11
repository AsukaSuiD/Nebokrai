//! Каноническое периодическое состояние `CLeafCutState` (`0x6B`).
//! Object Begin (0x005FC800) не требует user/sufferer: base Begin,
//! visual SetRun(1) → Update(0) → base visual tail, затем обнуление attack-count.
//! restart_leaf_cut_state переносит Begin(NULL, holder) по точному ключу:
//! timestamp из Unserialize и MasterInfo сохраняются; часы читает только visual.
//! Периодический AI изменяет payload по поколенческому ключу общей арены.
//! Чистый tick завершается до межвладельческого удара; состояние не вынимается
//! и остаётся доступным вложенному End/Clear. Удар использует независимый снимок.
//! Прямой End: vtable 0x006611F4, слот +0x1C → 0x005FD420: visual
//! с фазой 1 → GetSufferer (+0x18, 0x005DBFD0) → RemoveState (0x004CDAB0).
//! Это не CState::End: записи IsEnded и проверок времени/HP в нём нет.
//! Runtime Begin и StartAllStates связывают sufferer с holder; MasterInfo
//! остаётся источником атаки, а не владельцем удаляемого ключа. End работает
//! с опубликованной формой и точным ключом; ошибка доставки не отменяет удаление.
//! После фактического RemoveState общий virtual UpdateProperty вызывается
//! для живого держателя: player пересчитывает tagProperty, остальные формы
//! пересчитывают упорядоченные состояния и накопленные модификаторы.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/leafcutstate.cpp`. Состояние хранит снимок владельца и
//! атакующих свойств в момент применения, читает часы отдельно для срока жизни
//! и частоты, выполняет ровно два вызова MSVCRT RNG на атакующий тик и сохраняет
//! расширенные x87-цепочки с усечением к нулю. DB-запись длиной 68 байт
//! принадлежит этому типу;
//! `CanonicalStateStorage` атомарно поддерживает её смещение и жизненный цикл.
//! Tick меняет живое состояние по ключу до межвладельческого применения
//! уже рассчитанной атаки координатором `CGame`.
//! Клиентский срок разделяет exact-owner `0x00606320`: deadline-check и
//! положительный остаток читают wrapping clock отдельно.
//! Встроенный `tagAttackInformation` сохраняет конструкторные skill-id
//! `0x7fffffff` и уровень `1`; `Clear` между тиками их не переопределяет.
//! Базовый физический урон во всех трёх классах завершается через `__ftol2`:
//! в DWORD урона попадает младшая половина усечённого `i64`, после чего её
//! знаковое значение ограничивается снизу нулём. Критический пересчёт отдельно
//! использует `FISTP dword` и сохраняет его собственную overflow-семантику.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::fightdefense::truncate_original;
use crate::gameserver::appserver::skills::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

pub(crate) const LEAF_CUT_STATE_ID: u32 = 0x6b;
pub(crate) const LEAF_CUT_STATE_BYTES: usize = 68;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;
const DEFAULT_PERIODIC_SKILL_ID: u32 = i32::MAX as u32;

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
    pub(crate) fn reset_attack_count(&mut self) { self.attack_count = 0; }
    pub(crate) const fn serialized_span(self) -> Option<(usize, usize)> { match self.serialized_offset { Some(offset) => Some((offset, LEAF_CUT_STATE_BYTES)), None => None } }
    pub(crate) fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        if let Some(offset) = &mut self.serialized_offset {
            if *offset >= inserted_offset {
                *offset += amount;
            }
        }
    }

    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) { if self.serialized_offset.is_some_and(|offset| removed_offset < offset) { self.serialized_offset = self.serialized_offset.map(|offset| offset - amount); } }
    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) }

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

    pub(crate) fn encoded(self, now_ms: u32) -> Vec<u8> {
        let mut record = Vec::with_capacity(LEAF_CUT_STATE_BYTES); let mut writer = LegacyWriter::new(&mut record); writer.write_u32(self.state_id);
        for value in [self.master.master_type, self.master.master_id, self.master.master_guild_id, self.master.master_team_id, self.master.master_union_id, self.master.master_country_id, self.master.permitted_to_kill_player, self.master.permitted_to_kill_teammate, self.master.permitted_to_kill_guild_member, self.master.permitted_to_kill_criminal] { writer.write_i32(value); }
        writer.write_u32(self.remaining_time(now_ms)); writer.write_u32(self.frequency_ms); writer.write_u32(self.damage_factor_bits); writer.write_u32(self.damage_modifier_bits); writer.write_u16(self.minimum_attack); writer.write_u16(self.maximum_attack); writer.write_u16(self.element_attack); writer.write_u16(self.soul_attack); record
    }

    pub(crate) fn encoded_for_install(self) -> Vec<u8> { self.encoded(self.started_at_ms) }

    pub(crate) fn update_serialized_runtime(self, payload: &mut [u8], now_ms: u32) { if let Some(offset) = self.serialized_offset { let _ = LegacyWriter::write_u32_at(payload, offset + 44, self.remaining_time(now_ms)); } }
    fn remaining_time(self, now_ms: u32) -> u32 { let elapsed = now_ms.wrapping_sub(self.started_at_ms); if elapsed >= self.keep_time_ms { 0 } else { self.keep_time_ms.wrapping_sub(elapsed) } }

    pub(crate) fn tick(&mut self, lifetime_now_ms: u32, frequency_now_ms: u32, target_dead: bool, critical_chance: u16, critical_rate: f32, random: &mut dyn FnMut(i32) -> i32) -> LeafCutStateTick {
        if self.started_at_ms.wrapping_add(self.keep_time_ms) < lifetime_now_ms || target_dead { return LeafCutStateTick::Ended; }
        if self.started_at_ms.wrapping_add(self.frequency_ms.wrapping_mul(self.attack_count)) >= frequency_now_ms { return LeafCutStateTick::Pending; }
        self.attack_count = self.attack_count.wrapping_add(1); LeafCutStateTick::Attack(self.attack(critical_chance, critical_rate, random))
    }

    fn attack(self, critical_chance: u16, critical_rate: f32, random: &mut dyn FnMut(i32) -> i32) -> AttackInformation {
        let minimum = i32::from(self.minimum_attack); let maximum = i32::from(self.maximum_attack); let span = minimum.abs_diff(maximum).wrapping_add(1) as i32; let mut physical = truncate_original_i64_low(f64::from(random(span)) + f64::from(f32::from_bits(self.damage_modifier_bits)) + f64::from(minimum)); if physical < 0 { physical = 0; }
        let critical = random(100) < i32::from(critical_chance); let mut damages = vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: i32::from(self.element_attack), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(self.soul_attack), mp_damage: 0 }]; if critical { for damage in &mut damages { damage.hp_damage = truncate_original(f64::from(damage.hp_damage) * f64::from(critical_rate)); } }
        AttackInformation { skill_id: DEFAULT_PERIODIC_SKILL_ID, skill_level: 1, attacker_type: self.master.master_type, attacker_id: self.master.master_id, attacker_team_id: self.master.master_team_id, attacker_faction_id: self.master.master_guild_id, attacker_union_id: self.master.master_union_id, hit_modifier: 0, damage_factor: f32::from_bits(self.damage_factor_bits), damage_modifier: 0, critical, blast_attack: false, full_miss: 0, damages }
    }
}

pub(crate) fn send_leaf_cut_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: LeafCutState, begin: bool, now_ms: u32) {
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(state.skill_id() as i32); if begin { message.add_ulong(state.client_state_time(|| now_ms)); message.add_long(0); } let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn restart_leaf_cut_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<LeafCutState>(key)).copied()
    else { return false };
    if !crate::gameserver::appserver::states::state::begin_base_applied_state(
        game, region_id, holder, key,
    ) { return false }
    if crate::gameserver::appserver::states::state::begin_applied_state_visual(
        game, region_id, holder, key, 1,
    ) {
        let mut message = CMessage::new(STATE_BEGIN_MESSAGE);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.skill_id() as i32);
        message.add_ulong(state.client_state_time(&mut *now));
        message.add_long(0);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = crate::gameserver::appserver::states::state::update_applied_state_visual_base(
            game, region_id, holder, key,
        );
    }
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<LeafCutState>(key))
    {
        state.reset_attack_count();
    }
    true
}

pub(crate) fn end_leaf_cut_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state_id) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<LeafCutState>(key))
        .map(|state| state.skill_id())
    else {
        return false;
    };
    let mut message = CMessage::new(STATE_END_MESSAGE);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state_id as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<LeafCutState>(key, LEAF_CUT_STATE_BYTES))
        .is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}

pub(crate) fn update_player_leaf_cut_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    let target = game.find_player(player_id).and_then(|player| {
        player.move_shape().applied_state::<LeafCutState>(key)?;
        let shape = player.move_shape().shape();
        Some((
            shape.identity(),
            player.server_region_id()?,
            player.is_dead(),
            player.combat_properties().cch,
        ))
    });
    let Some((identity, region_id, dead, critical_chance)) = target else { return false };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    let critical_rate = game.globe_setup().critical_rate();
    let prepared = game.with_player_state_random::<LeafCutState, _>(player_id, key, |state, random| {
        let tick = state.tick(lifetime_now_ms, frequency_now_ms, dead, critical_chance, critical_rate, random);
        (tick, *state)
    });
    let Some((tick, state)) = prepared else { return false };
    match tick {
        LeafCutStateTick::Pending => {}
        LeafCutStateTick::Attack(attack) => {
            let master = state.master();
            game.apply_owned_skill_attack_to_player(master, player_id, region_id, attack, runtime);
        }
        LeafCutStateTick::Ended => {
            let _ = end_leaf_cut_state(game, region_id, identity, key);
            let _ = game.publish_player_states(player_id);
        }
    }
    true
}

pub(crate) fn update_monster_leaf_cut_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    let Some(owner) = game.take_region_owner(region_id) else { return false };
    let target = owner.base().find_monster_by_id(monster_id).and_then(|monster| {
        monster.move_shape().applied_state::<LeafCutState>(key)?;
        let shape = monster.move_shape().shape();
        Some((shape.identity(), monster.hit_points() == 0))
    });
    game.restore_region_owner(owner);
    let Some((identity, dead)) = target else { return false };
    let lifetime_now_ms = runtime.now_milliseconds();
    let frequency_now_ms = runtime.now_milliseconds();
    let critical_rate = game.globe_setup().critical_rate();
    let Some(mut owner) = game.take_region_owner(region_id) else { return false };
    let prepared = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().applied_state_mut::<LeafCutState>(key)?;
        let tick = state.tick(lifetime_now_ms, frequency_now_ms, dead, 0, critical_rate, &mut |maximum| game.skill_random_below(maximum));
        Some((tick, *state))
    });
    game.restore_region_owner(owner);
    let Some((tick, state)) = prepared else { return false };
    match tick {
        LeafCutStateTick::Pending => {}
        LeafCutStateTick::Attack(attack) => {
            let master = state.master();
            game.apply_owned_skill_attack_to_monster(master, monster_id, region_id, attack, runtime);
        }
        LeafCutStateTick::Ended => {
            let _ = end_leaf_cut_state(game, region_id, identity, key);
        }
    }
    true
}
