//! Каноническое состояние подавления `CRoarState` (`0x83`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/roarstate.cpp`. Состояние до строгого истечения срока
//! уменьшает обе границы физической атаки и дополнительную стихийную атаку,
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
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ROAR_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(0, reader.read_u32()?, reader.read_i32()?, reader.read_i32()?))
    }
    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self { self.started_at_ms = now_ms; self }
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
        let attack_loss = self.attack_loss.max(0) as u32;
        let minimum_loss = properties.minimum_attack.min(attack_loss) & 0xffff;
        let maximum_loss = properties.maximum_attack.min(attack_loss) & 0xffff;
        let element_loss = properties.add_element_attack.min(self.element_attack_loss.max(0) as u32);
        properties.minimum_attack = properties.minimum_attack.wrapping_sub(minimum_loss).min(i32::MAX as u32);
        properties.maximum_attack = properties.maximum_attack.wrapping_sub(maximum_loss).min(i32::MAX as u32);
        properties.add_element_attack = properties.add_element_attack.wrapping_sub(element_loss);
        properties
    }
    pub(crate) const fn apply_to_monster(self, minimum: u32, maximum: u32, element: i32) -> (u32, u32, i32) {
        let attack_loss = self.attack_loss as u32;
        let element_loss = self.element_attack_loss;
        (
            minimum.wrapping_sub(if minimum < attack_loss { minimum } else { attack_loss }),
            maximum.wrapping_sub(if maximum < attack_loss { maximum } else { attack_loss }),
            element.wrapping_sub(if element < element_loss { element } else { element_loss }),
        )
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
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    removed
}
