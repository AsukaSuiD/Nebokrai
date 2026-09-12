//! Подавление атаки CRoarState и его первичная установка.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/roarstate.cpp.
//!
//! Замена завершает первый ID83 и уничтожает остаток той же позиции до
//! чтения параметров нового состояния. Begin(U,S) читает часы и публикует
//! visual до append; UpdateProperty вызывается после публикации в арене.
//! NULL-user restart сохраняет timestamp/user и обновляет Sufferer.
//! End выполняет visual → свежий Sufferer → RemoveState без state.ended;
//! отсутствие S не удаляет запись из другого держателя.
//!
//! AI проверяет строгий absolute wrapping deadline, затем смерть S.
//! Пересчёт сначала обновляет visual, затем снижает MIN/MAX и element_modify
//! игрока либо модификаторы монстра. Физические losses сравниваются unsigned,
//! элементный loss игрока — signed. DB хранит ID/remaining/два loss (16 байт);
//! Unserialize читает часы после трёх полей. SlotMap владеет экземплярами,
//! общий visual сохраняет loop1, сообщения BFE03/BFE04 и порядок callbacks.

use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, end_and_destroy_state_at,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut, timed_client_state_time, update_applied_state_end_visual,
    update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const ROAR_STATE_ID: u32 = 0x83;
pub(crate) const ROAR_STATE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RoarState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_loss: i32,
    element_attack_loss: i32,
}

impl RoarState {
    pub(crate) const fn new(keep_time_ms: u32, attack_loss: i32, element_attack_loss: i32) -> Self {
        Self { started_at_ms: 0, keep_time_ms, attack_loss, element_attack_loss }
    }

    pub(crate) fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ROAR_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let keep_time_ms = reader.read_u32()?;
        let attack_loss = reader.read_i32()?;
        let element_attack_loss = reader.read_i32()?;
        Ok(Self { started_at_ms: now(), keep_time_ms, attack_loss, element_attack_loss })
    }

    pub(crate) fn encoded_for_install(self) -> [u8; ROAR_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now: impl FnMut() -> u32) -> [u8; ROAR_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now) as u32)
    }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; ROAR_STATE_BYTES] {
        let mut bytes = [0; ROAR_STATE_BYTES];
        bytes[..4].copy_from_slice(&ROAR_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.attack_loss.to_le_bytes());
        bytes[12..].copy_from_slice(&self.element_attack_loss.to_le_bytes());
        bytes
    }

    pub(crate) const fn skill_id(self) -> u32 { ROAR_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }
    pub(crate) fn client_time(self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }
    pub(crate) fn apply_to_player(self, mut properties: PlayerCombatProperties) -> PlayerCombatProperties {
        let attack_loss = self.attack_loss as u32;
        let minimum_loss = properties.minimum_attack.min(attack_loss);
        let maximum_loss = properties.maximum_attack.min(attack_loss);
        let element_loss = properties.element_modify.min(self.element_attack_loss);
        properties.minimum_attack = properties.minimum_attack.wrapping_sub(minimum_loss).min(i32::MAX as u32);
        properties.maximum_attack = properties.maximum_attack.wrapping_sub(maximum_loss).min(i32::MAX as u32);
        properties.element_modify = properties.element_modify.wrapping_sub(element_loss);
        properties
    }
}

fn participant(game: &CGame, target: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, target.0, target.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
}

pub(super) fn replace_roar_state(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return false; };
    let previous = shape.find_state_position(|state| state.state_id() == ROAR_STATE_ID);
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    let element_loss = properties.query_property(215) as i32;
    let attack_loss = properties.query_property(205) as i32;
    let keep_time = properties.query_property(10_002);
    let mut state = RoarState::new(keep_time, attack_loss, element_loss);
    state.started_at_ms = now();
    let Some(user) = participant(game, source) else { return false; };
    let Some(sufferer) = participant(game, target) else { return false; };
    if resolve_state_move_shape(game, sufferer.0, sufferer.1).is_some() {
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(sufferer.1.object_type);
        message.add_long(sufferer.1.id);
        message.add_ulong(ROAR_STATE_ID);
        message.add_long(state.client_time(&mut *now));
        message.add_ulong(0);
        let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
    }
    let record = state.encoded_for_install();
    let Some(shape) = resolve_state_move_shape_mut(game, target.0, target.1) else { return false; };
    let key = shape.append_applied_state_record(state, &record);
    shape.begin_applied_state_visual(key, 1);
    shape.update_applied_state_visual_base(key);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(user));
    shape.set_applied_state_sufferer(key, Some(sufferer));
    let _ = game.update_move_shape_properties(target.0, target.1);
    true
}

pub(crate) fn update_roar_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<RoarState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key)).copied()
    else { return false; };
    if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|properties| state.apply_to_player(properties));
        }
    } else if matches!(target.object_type, 600 | 602) {
        let losses = game.find_region(target_region)
            .and_then(|region| region.base().find_monster_by_id(target.id))
            .and_then(|monster| {
                let property = game.find_monster_property_by_origin_name(monster.original_name())?;
                let (minimum, maximum) = monster.state_attack_bounds(
                    property.minimum_attack, property.maximum_attack,
                );
                Some((
                    minimum.min(state.attack_loss as u32) as i32,
                    maximum.min(state.attack_loss as u32) as i32,
                    monster.element_modifier().min(state.element_attack_loss as u32) as i32,
                ))
            });
        if let Some((minimum_loss, maximum_loss, element_loss)) = losses {
            if let Some(monster) = game.find_region_mut(target_region)
                .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
                let modifiers = monster.move_shape_mut().property_modifiers_mut();
                modifiers.minimum_attack = modifiers.minimum_attack.wrapping_sub(minimum_loss);
                modifiers.maximum_attack = modifiers.maximum_attack.wrapping_sub(maximum_loss);
                modifiers.element_modify = modifiers.element_modify.wrapping_sub(element_loss);
            }
        }
    }
    true
}

pub(crate) fn restart_roar_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key)).is_none()
    { return false; }
    let Some(sufferer) = participant(game, (region_id, holder)) else { return false; };
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_applied_state_sufferer(key, Some(sufferer));
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<RoarState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.client_time(now) as u32,
        );
    }
    true
}

pub(crate) fn update_roar_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key))
    else { return false; };
    if state.expired(now_ms) { return end_roar_state(game, region_id, holder, key); }
    let dead = resolve_applied_state_sufferer(game, region_id, holder, key)
        .and_then(|target| game.move_shape_health(target.0, target.1))
        .is_some_and(|health| health == 0);
    dead && end_roar_state(game, region_id, holder, key)
}

pub(crate) fn end_roar_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, ROAR_STATE_BYTES)
}
