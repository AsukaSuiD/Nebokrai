//! Каноническое состояние подавления `CRoarState` (`0x83`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/roarstate.cpp`. Состояние до строгого истечения срока
//! уменьшает обе границы физической атаки и стихийный модификатор,
//! ограничивая каждое уменьшение текущим значением. Визуальные сообщения
//! сохраняют `0xBFE03/0xBFE04`; порядок относительно других достигнутых
//! состояниями свойств принадлежит `CanonicalStateStorage`. Vtable exact EXE
//! подтверждает общий с `CBlindState` клиентский срок по `0x005F2CD0` и
//! собственную serializer-пару `0x005F65F0/0x005ECC60`. Persisted-запись
//! `ID + remaining time + attack loss + element attack loss` занимает 16 байт;
//! spatial login восстанавливает срок до общего пересчёта свойств.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! Exact vtable 0x0065FE04: End 0x005FD420 выполняет visual →
//! GetSufferer → RemoveState. Прямой End и AI используют один exact-key хвост,
//! при этом только AI проверяет срок.

//! Restart воспроизводит только Begin(NULL, holder) (0x005ECA00):
//! базовый Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Visual принадлежит экземпляру общей арены: BeginVisualEffect(1) →
//! concrete Update(0) → базовый visual-хвост; только getter пакета читает часы.

//! Unserialize 0x005ECC60 сохраняет один собственный clock в timestamp;
//! decode получает его в now_ms для этой wire-записи, а restart не заменяет его.
//! Нативное чтение часов происходит после трёх полей записи.

//! OnUpdateProperties 0x005ECAA0: GetSufferer → существующий visual Update(0)
//! → type400/600/602 и RTTI. Физические player losses сравниваются как полные
//! unsigned DWORD без WORD-маски; элементная ветвь вычитает signed minimum
//! из element_modify (+0x3F0), НЕ add_element_attack (+0x3E8).
//! Монстр сравнивает все losses unsigned и прибавляет отрицательные deltas
//! к min/max/element modifiers. Исходный GetSufferer разрешает достигнутый
//! региональный monster type600; проверка 602 не расширяет общий resolver.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual, update_applied_state_visual_base,
};
use crate::gameserver::appserver::states::state::{
    resolve_applied_state_sufferer, update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::appserver::moveshape::StateKey;

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

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
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, attack_loss: i32, element_attack_loss: i32) -> Self {
        Self { started_at_ms, keep_time_ms, attack_loss, element_attack_loss }
    }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ROAR_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?, reader.read_i32()?))
    }

    pub(crate) fn encoded_for_install(self) -> [u8; ROAR_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; ROAR_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
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
        now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms
    }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
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

pub(crate) fn send_roar_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    x: i32,
    y: i32,
    state: RoarState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(ROAR_STATE_ID as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, x, y, &message);
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
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key)).copied()
        else { return false };
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.skill_id() as i32);
        message.add_long(state.client_time(now));
        message.add_long(0);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = update_applied_state_visual_base(game, region_id, holder, key);
    }
    true
}

pub(crate) fn update_roar_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_roar_state(game, region_id, holder, key)
}

pub(crate) fn end_roar_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RoarState>(key)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| {
            shape.remove_applied_state_record::<RoarState>(key, ROAR_STATE_BYTES)
        }).is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}
