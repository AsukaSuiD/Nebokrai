//! Общий lifecycle периодического урона и его сериализация.
//! Источник: gameserver.exe/GameServer.pdb, периодические состояния appserver/skills.
//!
//! Состояние принадлежит существующей SlotMap-арене; callbacks работают с тем
//! же ключом и фактическими User/Sufferer. Отдельно передаются только параметры
//! удара, чтобы освободить заимствование перед вложенными игровыми действиями.
//! NULL-user restart сохраняет источник и начало срока. End отправляет visual,
//! заново разрешает Sufferer и удаляет запись только из исходной арены.
//! Общий префикс сохранения — ID4/Master40/remaining4/frequency4; хвост задаёт
//! конкретный тип. Load читает Master до часов, Save не изменяет payload,
//! а техническая запись при установке состояния не читает часы.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::{AppliedState, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower};
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut, timed_client_state_time, update_applied_state_end_visual,
    update_property_state_visual,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PeriodicAttackCore {
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    attack_count: u32,
}

impl PeriodicAttackCore {
    pub(crate) const fn new(master: MasterInfo, keep_time_ms: u32, frequency_ms: u32) -> Self {
        Self { master, started_at_ms: 0, keep_time_ms, frequency_ms, attack_count: 0 }
    }

    pub(crate) const fn master(&self) -> MasterInfo { self.master }

    pub(crate) fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn decode<'a>(
        payload: &'a [u8], offset: usize, id: u32, now: &mut dyn FnMut() -> u32,
    ) -> Result<(Self, LegacyReader<'a>), LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != id {
            return Err(LegacyReadBlock {
                offset, needed: 4, available: payload.len().saturating_sub(offset),
            });
        }
        let master = MasterInfo {
            master_type: reader.read_i32()?,
            master_id: reader.read_i32()?,
            master_guild_id: reader.read_i32()?,
            master_team_id: reader.read_i32()?,
            master_union_id: reader.read_i32()?,
            master_country_id: reader.read_i32()?,
            permitted_to_kill_player: reader.read_i32()?,
            permitted_to_kill_teammate: reader.read_i32()?,
            permitted_to_kill_guild_member: reader.read_i32()?,
            permitted_to_kill_criminal: reader.read_i32()?,
        };
        let started_at_ms = now();
        let core = Self {
            master, started_at_ms, keep_time_ms: reader.read_u32()?,
            frequency_ms: reader.read_u32()?, attack_count: 0,
        };
        Ok((core, reader))
    }

    fn encode(&self, writer: &mut LegacyWriter<'_>, remaining: u32) {
        for value in [
            self.master.master_type, self.master.master_id, self.master.master_guild_id,
            self.master.master_team_id, self.master.master_union_id, self.master.master_country_id,
            self.master.permitted_to_kill_player, self.master.permitted_to_kill_teammate,
            self.master.permitted_to_kill_guild_member, self.master.permitted_to_kill_criminal,
        ] { writer.write_i32(value); }
        writer.write_u32(remaining);
        writer.write_u32(self.frequency_ms);
    }
}

pub(crate) trait PeriodicAttackState: AppliedState {
    const STATE_ID: u32;
    const RECORD_BYTES: usize;
    const CHECK_LIFETIME_AFTER_ATTACK: bool = false;
    type AttackSeed;
    fn core(&self) -> &PeriodicAttackCore;
    fn core_mut(&mut self) -> &mut PeriodicAttackCore;
    fn encode_attack(&self, writer: &mut LegacyWriter<'_>);
    fn attack_seed(&self) -> Self::AttackSeed;
}

pub(crate) fn encode_periodic_state<T: PeriodicAttackState>(
    state: &T, now: impl FnMut() -> u32,
) -> Vec<u8> {
    encode_with_remaining(state, state.core().client_state_time(now))
}

pub(crate) fn encode_periodic_state_for_install<T: PeriodicAttackState>(state: &T) -> Vec<u8> {
    encode_with_remaining(state, state.core().keep_time_ms)
}

fn encode_with_remaining<T: PeriodicAttackState>(state: &T, remaining: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(T::RECORD_BYTES);
    let mut writer = LegacyWriter::new(&mut bytes);
    writer.write_u32(T::STATE_ID);
    state.core().encode(&mut writer, remaining);
    state.encode_attack(&mut writer);
    bytes
}

pub(crate) fn periodic_attack_information(
    master: MasterInfo, damage_factor: f32, critical: bool, power: AttackPower,
) -> AttackInformation {
    let player_master = master.master_type == 400;
    AttackInformation {
        skill_id: i32::MAX as u32,
        skill_level: 1,
        attacker_type: master.master_type,
        attacker_id: master.master_id,
        attacker_team_id: if player_master { master.master_team_id } else { 0 },
        attacker_faction_id: if player_master { master.master_guild_id } else { 0 },
        attacker_union_id: if player_master { master.master_union_id } else { 0 },
        hit_modifier: 0,
        damage_factor,
        damage_modifier: 0,
        critical,
        blast_attack: false,
        full_miss: 0,
        damages: vec![power],
    }
}

#[allow(clippy::too_many_arguments, reason = "раздельные User/Sufferer и место регистрации сохраняют native Begin")]
pub(crate) fn begin_primary_periodic_attack_state<T: PeriodicAttackState>(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: T,
    placement: Option<(usize, usize)>,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    resolve_state_move_shape(game, holder_region, holder)?;
    if user.is_some() { state.core_mut().started_at_ms = now(); }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = match sufferer { Some(sufferer) => Some(participant(sufferer)?), None => None };
    if let Some((region, identity)) = sufferer {
        if let Some(shape) = resolve_state_move_shape(game, region, identity) {
            let target_region = shape.shape().get_region_id();
            let target = shape.shape().identity();
            let mut message = CMessage::new(STATE_BEGIN_MESSAGE);
            message.add_long(target.object_type);
            message.add_long(target.id);
            message.add_ulong(T::STATE_ID);
            message.add_ulong(state.core().client_state_time(&mut *now));
            message.add_ulong(0);
            let _ = game.send_move_shape_around(target_region, target, &message);
        }
    }
    // После Update(0) у loop1 остаётся незавершённый visual; arena создаёт
    // именно этот остаток. Счётчик сбрасывается до передачи state владельцу.
    state.core_mut().attack_count = 0;
    let record = encode_periodic_state_for_install(&state);
    let shape = resolve_state_move_shape_mut(game, holder_region, holder)?;
    let key = match placement {
        Some(location) => shape.insert_replacement_state_record(state, &record, location)?,
        None => shape.append_applied_state_record(state, &record),
    };
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, sufferer);
    Some(key)
}

pub(crate) fn restart_periodic_attack_state<T: PeriodicAttackState>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<T>(key)).is_none()
        || !begin_base_applied_state(game, region_id, holder, key)
    {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<T>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.core().client_state_time(now),
        );
    }
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<T>(key))
    {
        state.core_mut().attack_count = 0;
    }
    true
}

pub(crate) fn end_periodic_attack_state<T: PeriodicAttackState>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<T>(key)).is_none()
    {
        return false;
    }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, T::RECORD_BYTES)
}

pub(crate) fn update_periodic_attack_state<T: PeriodicAttackState, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    runtime: &mut Runtime,
    calculate: impl FnOnce(&mut CGame, (i32, ShapeIdentity), MasterInfo, T::AttackSeed) -> AttackInformation,
) -> bool {
    if !T::CHECK_LIFETIME_AFTER_ATTACK {
        let lifetime_now_ms = runtime.now_milliseconds();
        let Some(expired) = resolve_state_move_shape(game, region_id, holder)
            .and_then(|shape| shape.applied_state::<T>(key))
            .map(|state| state.core().expired(lifetime_now_ms))
        else { return false; };
        if expired {
            end_periodic_attack_state::<T>(game, region_id, holder, key);
            return true;
        }
    }
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else {
        end_periodic_attack_state::<T>(game, region_id, holder, key);
        return true;
    };
    if game.move_shape_health(target_region, target).is_none_or(|health| health == 0) {
        end_periodic_attack_state::<T>(game, region_id, holder, key);
        return true;
    }
    let Some(count) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<T>(key))
        .map(|state| state.core().attack_count)
    else { return false; };
    let frequency_now_ms = runtime.now_milliseconds();
    let prepared = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<T>(key))
        .and_then(|state| {
            let core = state.core_mut();
            let deadline = core.started_at_ms.wrapping_add(core.frequency_ms.wrapping_mul(count));
            if deadline >= frequency_now_ms { return None; }
            core.attack_count = count.wrapping_add(1);
            Some((core.master, state.attack_seed()))
        });
    if let Some((master, seed)) = prepared {
        let attack = calculate(game, (target_region, target), master, seed);
        if master.master_type == MONSTER_TYPE {
            game.apply_monster_periodic_state_attack(master, target, target_region, attack, runtime);
        } else {
            match target.object_type {
                400 => game.apply_owned_skill_attack_to_player(master, target.id, target_region, attack, runtime),
                600 => game.apply_owned_skill_attack_to_monster(master, target.id, target_region, attack, runtime),
                1_100 | 1_200 if master.master_type == 400 => {
                    game.apply_owned_skill_attack_to_stationary_build(
                        master.master_id, target_region, target, attack, runtime,
                    );
                }
                _ => {}
            }
        }
    }
    if T::CHECK_LIFETIME_AFTER_ATTACK {
        let lifetime_now_ms = runtime.now_milliseconds();
        let expired = resolve_state_move_shape(game, region_id, holder)
            .and_then(|shape| shape.applied_state::<T>(key))
            .is_some_and(|state| state.core().expired(lifetime_now_ms));
        if expired {
            end_periodic_attack_state::<T>(game, region_id, holder, key);
        }
    }
    true
}
