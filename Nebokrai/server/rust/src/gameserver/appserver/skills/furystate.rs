//! CFuryState и общий payload усиления атаки из gameserver.exe/GameServer.pdb,
//! appserver/skills/furystate.cpp и ragebreakstate.cpp.
//!
//! Совпадающие таймер, 12-байтовый codec и OnUpdateProperties принадлежат
//! одной реализации; разные ID сохраняют отдельные типы и ключи общей арены.
//! Begin Fury требует U, читает часы в базе и создаёт loop1 без Update.
//! End обновляет visual actual S, затем базовый End удаляет запись у actual U.
//! Поэтому Begin(NULL, holder) при загрузке не меняет часы, ended и visual.
//! SetRegion меняет только регион User.
//! Загрузка читает часы перед keep/gain; Save пишет ID, живой остаток и gain.
//! Положительный остаток требует двух чтений часов; истечение строгое.
//! Signed gain × 0.01_f32 × полный unsigned maximum усекается через __ftol2.
//! Игрок дважды сужает gain до WORD вокруг ограничения суммы 0xFFFF;
//! монстр прибавляет полный signed delta к modifier максимальной атаки.
//! Временный расчётный delta не является сериализуемым полем.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::{AppliedState, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, end_base_applied_state, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
    update_applied_state_end_visual, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

pub(crate) const FURY_STATE_SKILL_ID: u32 = 0x1a3;
pub(crate) const FURY_STATE_BYTES: usize = 12;
pub(crate) type FuryState = AttackGainState<FURY_STATE_SKILL_ID>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AttackGainState<const ID: u32> {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_gain_percent: i32,
}

impl<const ID: u32> AttackGainState<ID> {
    pub(crate) const fn new(keep_time_ms: u32, attack_gain_percent: i32) -> Self {
        Self { started_at_ms: 0, keep_time_ms, attack_gain_percent }
    }

    pub(crate) const fn skill_id(&self) -> u32 { ID }

    pub(crate) fn decode(
        payload: &[u8], offset: usize, now_ms: u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ID {
            return Err(LegacyReadBlock {
                offset, needed: 4, available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self {
            started_at_ms: now_ms,
            keep_time_ms: reader.read_u32()?,
            attack_gain_percent: reader.read_i32()?,
        })
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; FURY_STATE_BYTES] {
        let mut bytes = [0; FURY_STATE_BYTES];
        bytes[..4].copy_from_slice(&ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.keep_time_ms.to_le_bytes());
        bytes[8..].copy_from_slice(&self.attack_gain_percent.to_le_bytes());
        bytes
    }

    pub(crate) fn encoded(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; FURY_STATE_BYTES] {
        let mut bytes = [0; FURY_STATE_BYTES];
        bytes[..4].copy_from_slice(&ID.to_le_bytes());
        let remaining = self.client_time(now_milliseconds) as u32;
        bytes[4..8].copy_from_slice(&remaining.to_le_bytes());
        bytes[8..].copy_from_slice(&self.attack_gain_percent.to_le_bytes());
        bytes
    }

    pub(crate) const fn restart_timer(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub(crate) const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(&self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    fn truncated_gain(&self, maximum: u32) -> i32 {
        truncate_original_i64_low(
            f64::from(self.attack_gain_percent)
                * f64::from(0.01_f32)
                * f64::from(maximum),
        )
    }

    pub(crate) fn apply_to_player_maximum_attack(&self, maximum: u32) -> u32 {
        let mut gain = self.truncated_gain(maximum) as u16 as u32;
        if maximum.wrapping_add(gain) > u16::MAX as u32 {
            gain = (u16::MAX as u32).wrapping_sub(maximum);
        }
        maximum.wrapping_add(gain as u16 as u32).min(i32::MAX as u32)
    }
}

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub(crate) fn begin_primary_fury_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: FuryState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    user?;
    begin_primary_attack_gain_state(game, holder_region, holder, user, sufferer, state, now)
}

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub(super) fn begin_primary_attack_gain_state<const ID: u32>(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: AttackGainState<ID>,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey>
where
    AttackGainState<ID>: AppliedState,
{
    if user.is_some() { state.started_at_ms = now(); }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity {
            ex_id: CGuid::GUID_INVALID, ..shape.identity()
        }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = match sufferer {
        Some(sufferer) => Some(participant(sufferer)?), None => None,
    };
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, holder_region, holder)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, sufferer);
    // Между base Begin, созданием loop1 и append нет внешнего callback.
    // Ресурс создаёт каталог арены; первый пакет принадлежит UpdateProperty.
    Some(key)
}

pub(crate) fn update_fury_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
    key: StateKey, now: &mut dyn FnMut() -> u32,
) -> bool {
    update_attack_gain_state_properties::<FURY_STATE_SKILL_ID>(game, region_id, holder, key, now)
}

pub(super) fn update_attack_gain_state_properties<const ID: u32>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool
where
    AttackGainState<ID>: AppliedState,
{
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<AttackGainState<ID>>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AttackGainState<ID>>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        let maximum = game.find_region(target_region)
            .and_then(|region| region.base().find_monster_by_id(target.id))
            .and_then(|monster| {
                let property = game.find_monster_property_by_origin_name(monster.original_name())?;
                Some(monster.state_attack_bounds(property.minimum_attack, property.maximum_attack).1)
            });
        if let Some(maximum) = maximum {
            let gain = state.truncated_gain(maximum);
            if let Some(monster) = game.find_region_mut(target_region)
                .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id))
            {
                let modifiers = monster.move_shape_mut().property_modifiers_mut();
                modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(gain);
            }
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                properties.maximum_attack = state.apply_to_player_maximum_attack(properties.maximum_attack);
                properties
            });
        }
    }
    true
}

pub(crate) fn restart_fury_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    false
}

pub(crate) fn update_fury_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<FuryState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_fury_state(game, region_id, holder, key)
}

pub(crate) fn end_fury_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<FuryState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    end_base_applied_state(game, region_id, holder, key, FURY_STATE_BYTES)
}
