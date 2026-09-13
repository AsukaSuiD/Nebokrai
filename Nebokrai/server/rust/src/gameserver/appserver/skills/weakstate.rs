//! Состояние ослабления: прямоугольник действия и снижение границ атаки.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/weakstate.cpp.
//!
//! Владение, visual и сохранённая запись принадлежат общей арене. Объектный
//! Begin требует S, но допускает NULL U без обновления времени; пакет появляется
//! только при пересчёте свойств. End обновляет visual и удаляет тот же объект
//! через актуальную S. Игрок сужает ограниченное снижение до WORD, монстр
//! вычитает полный DWORD из обоих модификаторов.
//!
//! Тип 1 истекает по строгому сроку, тип 2 — при выходе S из прямоугольника
//! или смене региона; прочие типы завершаются сразу. Координаты в DB не входят.
//! Непредставимая координата даёт исходный FISTP INT_MIN и участвует в
//! проверке прямоугольника, а не сохраняет состояние ранним отказом.
//! Load читает часы после типа, Save — после ID и типа, не меняя live-состояние.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, remove_applied_state_from,
    resolve_applied_state_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
    timed_client_state_time, update_applied_state_end_visual, update_property_state_visual,
    StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

pub(crate) const WEAK_STATE_ID: u32 = 0x12e;
pub(crate) const WEAK_STATE_BYTES: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WeakState {
    state_type: u32,
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_loss: u32,
    center_x: i32,
    center_y: i32,
    length: i32,
    height: i32,
}

impl WeakState {
    pub(crate) const fn new(attack_loss: u32, center_x: i32, center_y: i32, length: i32, height: i32) -> Self {
        Self { state_type: 2, started_at_ms: 0, keep_time_ms: 0, attack_loss, center_x, center_y, length, height }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != WEAK_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let state_type = reader.read_u32()?;
        let started_at_ms = now();
        let keep_time_ms = reader.read_u32()?;
        Ok(Self {
            state_type,
            started_at_ms,
            keep_time_ms,
            ..Self::new(reader.read_u32()?, 0, 0, 0, 0)
        })
    }

    pub(crate) fn encoded(&self, now: impl FnMut() -> u32) -> [u8; WEAK_STATE_BYTES] {
        self.encode_record(|| self.client_time(now) as u32)
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; WEAK_STATE_BYTES] {
        self.encode_record(|| self.keep_time_ms)
    }

    fn encode_record(&self, remaining: impl FnOnce() -> u32) -> [u8; WEAK_STATE_BYTES] {
        let mut bytes = [0; WEAK_STATE_BYTES];
        bytes[..4].copy_from_slice(&WEAK_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.state_type.to_le_bytes());
        bytes[8..12].copy_from_slice(&remaining().to_le_bytes());
        bytes[12..].copy_from_slice(&self.attack_loss.to_le_bytes());
        bytes
    }

    pub(crate) const fn skill_id(&self) -> u32 { WEAK_STATE_ID }

    pub(crate) fn client_time(&self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) const fn contains(&self, tile_x: i32, tile_y: i32) -> bool {
        let start_x = self.center_x.wrapping_sub(self.length / 2);
        let start_y = self.center_y.wrapping_sub(self.height / 2);
        tile_x >= start_x
            && tile_x < start_x.wrapping_add(self.length)
            && tile_y >= start_y
            && tile_y < start_y.wrapping_add(self.height)
    }

    pub(crate) fn apply_to_player(&self, mut properties: PlayerCombatProperties) -> PlayerCombatProperties {
        let minimum_loss = properties.minimum_attack.min(self.attack_loss) & 0xffff;
        let maximum_loss = properties.maximum_attack.min(self.attack_loss) & 0xffff;
        properties.minimum_attack = properties.minimum_attack.wrapping_sub(minimum_loss).min(i32::MAX as u32);
        properties.maximum_attack = properties.maximum_attack.wrapping_sub(maximum_loss).min(i32::MAX as u32);
        properties
    }
}

#[allow(clippy::too_many_arguments, reason = "User, Sufferer и держатель арены независимы")]
pub(crate) fn begin_primary_weak_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: WeakState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    resolve_state_move_shape(game, holder_region, holder)?;
    resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
    if user.is_some() { state.started_at_ms = now(); }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, holder_region, holder)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

pub(crate) fn update_weak_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none()
    { return false; }
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<WeakState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key))
    else { return false; };
    match target.object_type {
        600 => {
            let loss = state.attack_loss as i32;
            if let Some(monster) = game.find_region_mut(target_region)
                .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id))
            {
                let modifiers = monster.move_shape_mut().property_modifiers_mut();
                modifiers.minimum_attack = modifiers.minimum_attack.wrapping_sub(loss);
                modifiers.maximum_attack = modifiers.maximum_attack.wrapping_sub(loss);
            }
        }
        400 => {
            let Some(properties) = game.find_player(target.id).map(|player| player.combat_properties())
            else { return false; };
            let updated = state.apply_to_player(properties);
            if let Some(player) = game.find_player_mut(target.id) {
                player.update_state_combat_properties(|_| updated);
            }
        }
        _ => {}
    }
    true
}

pub(crate) fn restart_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none() {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(shape) = resolve_state_move_shape(game, region_id, holder) else { return false };
    let Some(state) = shape.applied_state::<WeakState>(key) else { return false };
    match state.state_type {
        1 => {
            if state.started_at_ms.wrapping_add(state.keep_time_ms) >= now() {
                return false;
            }
        }
        2 => {
            let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
            else { return false; };
            let Some(target) = resolve_state_move_shape(game, target_region, target)
            else { return false; };
            let x = target.shape().get_tile_x().unwrap_or(i32::MIN);
            let y = target.shape().get_tile_y().unwrap_or(i32::MIN);
            if state.contains(x, y) {
                return false;
            }
        }
        _ => {}
    }
    end_weak_state(game, region_id, holder, key)
}

pub(crate) fn set_weak_state_region(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) {
    let Some(state_type) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key))
        .map(|state| state.state_type)
    else { return; };
    if state_type == 2 {
        let _ = end_weak_state(game, region_id, holder, key);
    } else if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        let _ = shape.set_applied_state_sufferer_region(key, region_id);
    }
}

pub(crate) fn end_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(
        game, region_id, holder, key, StatePropertyTarget::Sufferer,
    );
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(
        game, region_id, holder, key, (target_region, target), WEAK_STATE_BYTES,
    )
}
