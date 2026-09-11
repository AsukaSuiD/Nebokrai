//! Общий payload и lifecycle CPoisonArrowState (0x21E) и CSpiderPoisonState
//! (0x191), GameServer.exe/GameServer.pdb, owners skills/poisonarrowstate.cpp
//! и skills/spiderpoisonstate.cpp. Их Begin/AI отличаются адресами, но сохраняют
//! одинаковый порядок: условный base clock → actual U/S → loop1 visual →
//! count reset → регистрация; AI: clock → absolute deadline → actual S/IsDied →
//! count → clock → absolute frequency → count++ → независимый удар.
//! Различие CalculateAttackPower явно сохранено: Arrow 0x005E3690 обнуляет
//! отрицательный signed HP, Spider 0x005E9610 передаёт его без ограничения.
//! MasterInfo заполняет team/guild/union и PK-права атаки только для player.
//! End 0x005FD420: optional visual1 → свежий S → RemoveState; state.ended
//! не меняется. SetRegion 0x005E3B30 меняет S.region, NULL restart сохраняет U.
//! Общие Save 0x005E93C0 и Load 0x005E3500 используют 56 байт:
//! ID/Master40/remaining/frequency/HP. Save не меняет payload; Load читает
//! Master до clock. Remaining 0x00606320 при положительном остатке читает
//! часы повторно. Install-cache часов не читает и не заменяет Serialize.
//! Два типовых ID сохраняют существующие варианты единой SlotMap-арены,
//! без второго хранилища и копий живых состояний. Межвладельческие callbacks
//! получают только независимые MasterInfo/AttackInformation; достигнутые цели
//! player/monster обслуживают существующие adapters. Неизвестные overload
//! Begin остаются в исходных owner-файлах.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::{AppliedState, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
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
const DEFAULT_PERIODIC_SKILL_ID: u32 = i32::MAX as u32;
const MONSTER_TYPE: i32 = 600;
pub(crate) const POISON_STATE_BYTES: usize = 56;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PoisonState<const ID: u32> {
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    hp_loss: u32,
    attack_count: u32,
}

impl<const ID: u32> PoisonState<ID> {
    pub(crate) const fn new(
        master: MasterInfo,
        keep_time_ms: u32,
        frequency_ms: u32,
        hp_loss: u32,
    ) -> Self {
        Self {
            master, started_at_ms: 0, keep_time_ms, frequency_ms, hp_loss, attack_count: 0,
        }
    }

    pub(crate) fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ID {
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
        let mut state = Self::new(master, reader.read_u32()?, reader.read_u32()?, reader.read_u32()?);
        state.started_at_ms = started_at_ms;
        Ok(state)
    }
    pub(crate) fn encoded(&self, now_milliseconds: impl FnMut() -> u32) -> [u8; POISON_STATE_BYTES] {
        self.encoded_with_remaining(self.client_state_time(now_milliseconds))
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; POISON_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    fn encoded_with_remaining(&self, remaining: u32) -> [u8; POISON_STATE_BYTES] {
        let mut record = Vec::with_capacity(POISON_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut record);
        writer.write_u32(ID);
        for value in [
            self.master.master_type, self.master.master_id, self.master.master_guild_id,
            self.master.master_team_id, self.master.master_union_id, self.master.master_country_id,
            self.master.permitted_to_kill_player, self.master.permitted_to_kill_teammate,
            self.master.permitted_to_kill_guild_member, self.master.permitted_to_kill_criminal,
        ] {
            writer.write_i32(value);
        }
        writer.write_u32(remaining);
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.hp_loss);
        record.try_into().expect("размер периодического состояния яда фиксирован")
    }

    pub(crate) const fn skill_id(&self) -> u32 { ID }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }

    pub(crate) fn client_state_time(&self, now_milliseconds: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds)
    }

    fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    fn take_due_attack(&mut self, now_ms: u32, count: u32) -> Option<(MasterInfo, AttackInformation)> {
        let deadline = self.started_at_ms.wrapping_add(self.frequency_ms.wrapping_mul(count));
        if deadline >= now_ms { return None; }
        self.attack_count = count.wrapping_add(1);
        let player_master = self.master.master_type == 400;
        Some((self.master, AttackInformation {
            skill_id: DEFAULT_PERIODIC_SKILL_ID,
            skill_level: 1,
            attacker_type: self.master.master_type,
            attacker_id: self.master.master_id,
            attacker_team_id: if player_master { self.master.master_team_id } else { 0 },
            attacker_faction_id: if player_master { self.master.master_guild_id } else { 0 },
            attacker_union_id: if player_master { self.master.master_union_id } else { 0 },
            hit_modifier: 0,
            damage_factor: 1.0,
            damage_modifier: 0,
            critical: false,
            blast_attack: false,
            full_miss: 0,
            damages: vec![AttackPower {
                kind: AttackPowerType::Poison,
                // 0x21E обнуляет отрицательный signed HP, 0x191 сохраняет его.
                hp_damage: if ID == 0x21e {
                    (self.hp_loss as i32).max(0)
                } else {
                    self.hp_loss as i32
                },
                mp_damage: 0,
            }],
        }))
    }
}

#[allow(clippy::too_many_arguments, reason = "раздельные User/Sufferer и место регистрации сохраняют native Begin")]
pub(crate) fn begin_primary_poison_state<const ID: u32>(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: PoisonState<ID>,
    placement: Option<(usize, usize)>,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey>
where
    PoisonState<ID>: AppliedState,
{
    resolve_state_move_shape(game, holder_region, holder)?;
    if user.is_some() { state.started_at_ms = now(); }
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
            message.add_ulong(ID);
            message.add_ulong(state.client_state_time(&mut *now));
            message.add_ulong(0);
            let _ = game.send_move_shape_around(target_region, target, &message);
        }
    }
    // После Update(0) у loop1 остаётся незавершённый visual; arena создаёт
    // именно этот остаток. Счётчик сбрасывается до передачи state владельцу.
    state.attack_count = 0;
    let record = state.encoded_for_install();
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

pub(crate) fn restart_poison_state<const ID: u32>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool
where
    PoisonState<ID>: AppliedState,
{
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonState<ID>>(key)).is_none()
        || !begin_base_applied_state(game, region_id, holder, key)
    {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<PoisonState<ID>>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.client_state_time(now),
        );
    }
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<PoisonState<ID>>(key))
    {
        state.attack_count = 0;
    }
    true
}

pub(crate) fn end_poison_state<const ID: u32>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool
where
    PoisonState<ID>: AppliedState,
{
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonState<ID>>(key)).is_none()
    {
        return false;
    }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, POISON_STATE_BYTES)
}

pub(crate) fn update_poison_state<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool
where
    PoisonState<ID>: AppliedState,
{
    let lifetime_now_ms = runtime.now_milliseconds();
    let Some(expired) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonState<ID>>(key))
        .map(|state| state.expired(lifetime_now_ms))
    else { return false; };
    if expired {
        end_poison_state::<ID>(game, region_id, holder, key);
        return true;
    }
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else {
        end_poison_state::<ID>(game, region_id, holder, key);
        return true;
    };
    if game.move_shape_health(target_region, target).is_none_or(|health| health == 0) {
        end_poison_state::<ID>(game, region_id, holder, key);
        return true;
    }
    let Some(count) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonState<ID>>(key))
        .map(|state| state.attack_count)
    else { return false; };
    let frequency_now_ms = runtime.now_milliseconds();
    let Some((master, attack)) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<PoisonState<ID>>(key))
        .and_then(|state| state.take_due_attack(frequency_now_ms, count))
    else { return true; };
    if master.master_type == MONSTER_TYPE {
        game.apply_monster_periodic_state_attack(master, target, target_region, attack, runtime);
    } else {
        match target.object_type {
            400 => game.apply_owned_skill_attack_to_player(master, target.id, target_region, attack, runtime),
            600 => game.apply_owned_skill_attack_to_monster(master, target.id, target_region, attack, runtime),
            _ => {}
        }
    }
    true
}
