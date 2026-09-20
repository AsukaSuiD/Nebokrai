//! Состояния Po/Yu (0x212..0x219): gameserver.exe/GameServer.pdb,
//! appserver/skills/{pojia,pobing,pomo,pofa,yujia,yubing,yumo,yufa}state.cpp.
//!
//! User всегда держатель арены; Sufferer у Po — caster, у Yu — сам держатель.
//! Object Begin требует User, читает собственные часы и создаёт silent loop1.
//! OnUpdateProperties требует User: Po публикует ему, Yu — Sufferer,
//! а формула всегда меняет User. Среди монстровых формул существуют только
//! снижение атаки Pobing и element modifier Pomo; Yu меняют лишь игрока.
//! Повторные состояния применяются в живом порядке общей арены, без второго
//! агрегата. Замена завершает прежний экземпляр и добавляет новый в хвост.
//! Прямой End — базовый ended→RemoveState у User без visual. Таймер после
//! строгого unsigned deadline сначала обновляет существующий visual,
//! затем выполняет тот же End. NULL цели подавляет пакет, но не visual tail.
//! DB codec: ID→живой remaining→signed value; положительный remaining читает
//! часы дважды. Load читает собственный clock перед keep/value. Техническая
//! запись первичного наложения часов не читает. SetRegion меняет только User.
//! Begin(NULL, holder) после загрузки отказывает до базы: сохраняет timestamp,
//! ended и visual. Пустой User не заменяется holder; общий Clear удаляет остаток.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    timed_client_state_time, resolve_state_move_shape, end_base_applied_state,
    resolve_state_move_shape_mut, resolve_applied_state_user,
    update_applied_state_end_visual, update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;

pub(crate) const BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyAttributeKind {
    AttackAvoidLoss,
    AttackLoss,
    ElementModifyLoss,
    ElementAvoidLoss,
    AttackAvoidGain,
    AttackGain,
    ElementModifyGain,
    ElementAvoidGain,
}

impl BattleFairyAttributeKind {
    pub(crate) const fn targets_self(self) -> bool {
        matches!(
            self,
            Self::AttackAvoidGain
                | Self::AttackGain
                | Self::ElementModifyGain
                | Self::ElementAvoidGain
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyAttributeState {
    skill_id: u32,
    kind: BattleFairyAttributeKind,
    started_at_ms: u32,
    keep_time_ms: u32,
    value: i32,
}

impl BattleFairyAttributeState {
    pub(crate) const fn new(
        skill_id: u32,
        kind: BattleFairyAttributeKind,
        started_at_ms: u32,
        keep_time_ms: u32,
        value: i32,
    ) -> Self {
        Self { skill_id, kind, started_at_ms, keep_time_ms, value }
    }

    pub(crate) const fn skill_id(self) -> u32 { self.skill_id }
    pub(crate) const fn kind(self) -> BattleFairyAttributeKind { self.kind }
    pub(crate) const fn started_at_ms(self) -> u32 { self.started_at_ms }
    pub(crate) const fn keep_time_ms(self) -> u32 { self.keep_time_ms }
    pub(crate) const fn value(self) -> i32 { self.value }
    pub(crate) fn decode(
        payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let Some(definition) = super::battlefairyattribute::definition(skill_id) else { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); };
        let started_at_ms = now();
        Ok(Self::new(skill_id, definition.kind, started_at_ms, reader.read_u32()?, reader.read_i32()?))
    }
    pub(crate) fn encoded(
        &self, now: &mut dyn FnMut() -> u32,
    ) -> [u8; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES] {
        self.encoded_with_remaining(|| self.client_state_time(now))
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES] {
        self.encoded_with_remaining(|| self.keep_time_ms)
    }

    fn encoded_with_remaining(
        &self, remaining: impl FnOnce() -> u32,
    ) -> [u8; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES] {
        let mut bytes = [0; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining().to_le_bytes());
        bytes[8..].copy_from_slice(&self.value.to_le_bytes());
        bytes
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds)
    }

    pub(crate) fn apply_to_player(self, mut properties: PlayerCombatProperties) -> PlayerCombatProperties {
        match self.kind {
            BattleFairyAttributeKind::AttackAvoidLoss => {
                let value = i32::from(properties.attack_avoid).wrapping_sub(self.value);
                properties.attack_avoid = if value < 1 { 0 } else { value as u16 };
            }
            BattleFairyAttributeKind::AttackLoss => {
                properties.minimum_attack = subtract_player_attack(properties.minimum_attack, self.value);
                properties.maximum_attack = subtract_player_attack(properties.maximum_attack, self.value);
            }
            BattleFairyAttributeKind::ElementModifyLoss => {
                let value = properties.element_modify.wrapping_sub(self.value);
                properties.element_modify = if value < 1 { 0 } else { value };
            }
            BattleFairyAttributeKind::ElementAvoidLoss => {
                let value = i32::from(properties.element_avoid).wrapping_sub(self.value);
                properties.element_avoid = if value < 1 { 0 } else { value as u16 };
            }
            BattleFairyAttributeKind::AttackAvoidGain => {
                properties.attack_avoid = properties.attack_avoid.wrapping_add(self.value as u16);
            }
            BattleFairyAttributeKind::AttackGain => {
                properties.minimum_attack = add_player_attack(properties.minimum_attack, self.value);
                properties.maximum_attack = add_player_attack(properties.maximum_attack, self.value);
            }
            BattleFairyAttributeKind::ElementModifyGain => {
                properties.element_modify = properties.element_modify.wrapping_add(self.value);
            }
            BattleFairyAttributeKind::ElementAvoidGain => {
                properties.element_avoid = properties.element_avoid.wrapping_add(self.value as u16);
            }
        }
        properties
    }

    pub(crate) const fn apply_to_monster_element(self, value: i32) -> i32 {
        match self.kind {
            BattleFairyAttributeKind::ElementModifyLoss => {
                let next = value.wrapping_sub(self.value);
                if next < 1 { 0 } else { next }
            }
            _ => value,
        }
    }
}

const fn subtract_player_attack(value: u32, amount: i32) -> u32 {
    let next = (value as i32).wrapping_sub(amount);
    if next < 1 { 0 } else { next as u32 }
}

const fn add_player_attack(value: u32, amount: i32) -> u32 {
    let next = value.wrapping_add(amount as u32);
    if next > i32::MAX as u32 { i32::MAX as u32 } else { next }
}

pub(crate) fn begin_battle_fairy_attribute_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    sufferer: ShapeIdentity,
    mut state: BattleFairyAttributeState,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(user_region) = resolve_state_move_shape(game, region_id, holder)
        .map(|shape| shape.shape().get_region_id())
    else { return false };
    state.started_at_ms = now();
    let sufferer = resolve_state_move_shape(game, region_id, sufferer)
        .map(|shape| (shape.shape().get_region_id(), sufferer));
    let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) else { return false };
    let key = shape.append_applied_state_record(state, &state.encoded_for_install());
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some((user_region, holder)));
    shape.set_applied_state_sufferer(key, sufferer);
    true
}

pub(crate) fn update_battle_fairy_attribute_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BattleFairyAttributeState>(key))
    else { return false };
    let visual_target = if state.kind.targets_self() {
        StatePropertyTarget::Sufferer
    } else {
        StatePropertyTarget::User
    };
    if resolve_applied_state_user(game, region_id, holder, key).is_none() {
        return false;
    }
    let _ = update_property_state_visual::<BattleFairyAttributeState>(
        game, region_id, holder, key, visual_target, now,
        |state, now| state.client_state_time(now),
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BattleFairyAttributeState>(key)).copied()
    else { return false };
    let Some((target_region, target)) = resolve_applied_state_user(
        game, region_id, holder, key,
    ) else { return false };
    match target.object_type {
        400 => {
            let Some(player) = game.find_player_mut(target.id) else { return false };
            player.update_state_combat_properties(|properties| state.apply_to_player(properties));
        }
        600 => match state.kind {
            BattleFairyAttributeKind::AttackLoss => {
                for minimum in [true, false] {
                    let Some(shape) = resolve_state_move_shape_mut(game, target_region, target)
                    else { return false };
                    let modifiers = shape.property_modifiers_mut();
                    let value = if minimum {
                        &mut modifiers.minimum_attack
                    } else {
                        &mut modifiers.maximum_attack
                    };
                    *value = value.wrapping_sub(state.value);
                    let Some(monster) = game.find_region(target_region)
                        .and_then(|region| region.base().find_monster_by_id(target.id))
                    else { return false };
                    let Some(properties) = monster.base_property_key()
                        .and_then(|name| game.find_monster_property_by_origin_name(name))
                    else { return false };
                    let (low, high) = monster.state_attack_bounds(
                        properties.minimum_attack, properties.maximum_attack,
                    );
                    let current = (if minimum { low } else { high }) as i32;
                    if current < 0 {
                        let Some(shape) = resolve_state_move_shape_mut(game, target_region, target)
                        else { return false };
                        let modifiers = shape.property_modifiers_mut();
                        let value = if minimum {
                            &mut modifiers.minimum_attack
                        } else {
                            &mut modifiers.maximum_attack
                        };
                        *value = value.wrapping_sub(current);
                    }
                }
            }
            BattleFairyAttributeKind::ElementModifyLoss => {
                let Some(monster) = game.find_region(target_region)
                    .and_then(|region| region.base().find_monster_by_id(target.id))
                else { return false };
                let current = monster.element_modifier() as i32;
                let value = state.apply_to_monster_element(current);
                let Some(shape) = resolve_state_move_shape_mut(game, target_region, target)
                else { return false };
                shape.property_modifiers_mut().element_modify = value;
            }
            _ => {}
        },
        _ => {}
    }
    true
}

pub(crate) fn restart_battle_fairy_attribute_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: crate::gameserver::appserver::moveshape::StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    // Все восемь object Begin проверяют user до base Begin и visual.
    false
}

pub(crate) fn update_battle_fairy_attribute_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now_ms: u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BattleFairyAttributeState>(key))
        .copied()
    else { return false };
    if !state.expired(now_ms) { return false }
    let visual_target = if state.kind.targets_self() {
        StatePropertyTarget::Sufferer
    } else {
        StatePropertyTarget::User
    };
    update_applied_state_end_visual(game, region_id, holder, key, visual_target);
    end_battle_fairy_attribute_state(game, region_id, holder, key)
}

pub(crate) fn end_battle_fairy_attribute_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BattleFairyAttributeState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES)
}
