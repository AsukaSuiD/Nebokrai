//! Канонические состояния семейства боевого духа `0x212..0x219`.
//!
//! Источник: `gameserver.exe` и `GameServer.pdb`, владельцы
//! `pojia/pobing/pomo/pofa/yujia/yubing/yumo/yufa state`. Порядок состояний
//! остаётся порядком исходного `m_vStates`: замена удаляет прежнюю запись и
//! добавляет новую в хвост. Первый state-пакет принадлежит OnUpdateProperties
//! после silent Begin и передаёт клиентский остаток срока. Формулы игрока и монстра
//! разделены, поскольку часть ослаблений в оригинале не поддерживала монстров.
//! Все восемь concrete vtable (`Pojia..Yufa`) направляют клиентский срок на
//! `CFuryState::GetRemainedTime` по `0x00605E10` с двумя чтениями часов.
//! Те же vtable используют `Serialize` `0x005E7330` и `Unserialize`
//! `0x005FD660`: DB-запись состоит из ID, остатка срока и signed value.
//! Все direct End (+0x1C) используют CState::End (0x005DBCE0): ended,
//! затем RemoveState через GetUser, без visual. Timer (0x005E6E20) вызывает
//! другую перегрузку +0x48 (0x005E7310): visual и затем тот же base End.
//! Runtime Po* Begin получает (target, caster), Y* — (holder, holder);
//! GetUser в обоих случаях совпадает с владельцем state-list
//! (Pojia 0x0052AA6B..0x0052AA87, Yujia 0x00527002..0x0052701A).
//! restart_battle_fairy_attribute_state сохраняет этот false-return для
//! Begin(NULL, holder): без base Begin, изменения timestamp/ended и visual.
//! Собственный Unserialize 0x005FD660 читает часы до remaining/value.
//! Загруженный Begin(null, holder) возвращает до создания visual
//! (0x005E83B0, 0x005E6EE0), поэтому base End лишь отмечает такой ключ:
//! общий Clear удаляет остаток отдельно, таймер не выдумывает End-пакет.
//! OnUpdateProperties(+0x24) требует GetUser до visual: Po* отправляют его
//! user, Yu* — sufferer, после чего формула всегда обращается к user.
//! Из монстровых ветвей существуют только Pobing0x005E7D90 (AddMin/MaxAtk,
//! затем проверка живого getter) и Pomo0x005E7870 (Get/SetElementModifier).
//! Усиления Yu* меняют только игрока. Visual проверяет собственный ended;
//! ended/отсутствие цели сохраняют base tail, отсутствующий resource — нет.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    timed_client_state_time, resolve_state_move_shape, end_base_applied_state,
    resolve_state_move_shape_mut, resolve_applied_state_user,
    update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const ATTRIBUTE_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const ATTRIBUTE_STATE_END_MESSAGE: i32 = 0x000b_fe04;
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
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let Some(definition) = super::battlefairyattribute::definition(skill_id) else { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); };
        Ok(Self::new(skill_id, definition.kind, now_ms, reader.read_u32()?, reader.read_i32()?))
    }
    pub(crate) fn encoded(self, now_ms: u32) -> [u8; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES] {
        let elapsed = now_ms.wrapping_sub(self.started_at_ms); let remaining = if elapsed >= self.keep_time_ms { 0 } else { self.keep_time_ms.wrapping_sub(elapsed) };
        let mut bytes = [0; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES]; bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes()); bytes[4..8].copy_from_slice(&remaining.to_le_bytes()); bytes[8..].copy_from_slice(&self.value.to_le_bytes()); bytes
    }
    pub(crate) fn encoded_for_install(self) -> [u8; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES] { self.encoded(self.started_at_ms) }

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

pub(crate) fn send_battle_fairy_attribute_state_visual(
    game: &mut CGame,
    region_id: i32,
    target: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BattleFairyAttributeState,
    begin: bool,
) {
    let mut message = CMessage::new(if begin {
        ATTRIBUTE_STATE_BEGIN_MESSAGE
    } else {
        ATTRIBUTE_STATE_END_MESSAGE
    });
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.keep_time_ms() as i32);
        message.add_long(state.started_at_ms() as i32);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
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
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BattleFairyAttributeState>(key))
        .is_some_and(|state| state.expired(now_ms));
    if !expired { return false }
    let Some(shape) = resolve_state_move_shape(game, region_id, holder) else { return false };
    if shape.applied_state_was_loaded(key) == Some(false) {
        let Some(state) = shape.applied_state::<BattleFairyAttributeState>(key).copied() else { return false };
        let mut message = CMessage::new(ATTRIBUTE_STATE_END_MESSAGE);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.skill_id() as i32);
        let _ = game.send_move_shape_around(region_id, holder, &message);
    }
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
