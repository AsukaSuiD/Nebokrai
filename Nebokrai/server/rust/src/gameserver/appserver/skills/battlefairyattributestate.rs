//! Канонические состояния семейства боевого духа `0x212..0x219`.
//!
//! Источник: `gameserver.exe` и `GameServer.pdb`, владельцы
//! `pojia/pobing/pomo/pofa/yujia/yubing/yumo/yufa state`. Порядок состояний
//! остаётся порядком исходного `m_vStates`: замена удаляет прежнюю запись и
//! добавляет новую в хвост. Пакет начала передаёт исходные длительность и
//! отметку времени, а не вычисленный остаток. Формулы игрока и монстра
//! разделены, поскольку часть ослаблений в оригинале не поддерживала монстров.
//! Все восемь concrete vtable (`Pojia..Yufa`) направляют клиентский срок на
//! `CFuryState::GetRemainedTime` по `0x00605E10` с двумя чтениями часов.
//! Те же vtable используют `Serialize` `0x005E7330` и `Unserialize`
//! `0x005FD660`: DB-запись состоит из ID, остатка срока и signed value.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

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
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let Some(definition) = super::battlefairyattribute::definition(skill_id) else { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); };
        Ok(Self::new(skill_id, definition.kind, 0, reader.read_u32()?, reader.read_i32()?))
    }
    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self { self.started_at_ms = now_ms; self }
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

    pub(crate) const fn apply_to_monster_attack(self, value: u32) -> u32 {
        match self.kind {
            BattleFairyAttributeKind::AttackLoss => {
                let next = (value as i32).wrapping_sub(self.value);
                if next < 1 { 0 } else { next as u32 }
            }
            BattleFairyAttributeKind::AttackGain => {
                let next = value.wrapping_add(self.value as u32);
                if next > i32::MAX as u32 { i32::MAX as u32 } else { next }
            }
            _ => value,
        }
    }

    pub(crate) const fn apply_to_monster_element(self, value: i32) -> i32 {
        match self.kind {
            BattleFairyAttributeKind::ElementModifyLoss => {
                let next = value.wrapping_sub(self.value);
                if next < 1 { 0 } else { next }
            }
            BattleFairyAttributeKind::ElementModifyGain => value.wrapping_add(self.value),
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

pub(crate) fn expire_player_battle_fairy_attribute_state(
    game: &mut CGame,
    player_id: i32,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now_ms: u32,
) -> bool {
    let Some(state) = game.find_player_mut(player_id)
        .and_then(|player| player.take_expired_battle_fairy_attribute_state(key, now_ms))
    else { return false };
    let context = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().identity(),
            player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
    });
    let Some((region_id, target, tile_x, tile_y)) = context else { return true };
    send_battle_fairy_attribute_state_visual(game, region_id, target, tile_x, tile_y, state, false);
    let _ = game.update_player_properties(player_id);
    true
}

pub(crate) struct BattleFairyAttributeExpiration {
    target: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BattleFairyAttributeState,
}

impl BattleFairyAttributeExpiration {
    pub(crate) fn deliver(self, game: &mut CGame, region_id: i32) {
        send_battle_fairy_attribute_state_visual(
            game, region_id, self.target, self.tile_x, self.tile_y, self.state, false,
        );
    }
}

pub(crate) fn take_expired_monster_battle_fairy_attribute_state(
    region: &mut CServerRegion,
    monster_id: i32,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now_ms: u32,
) -> Option<BattleFairyAttributeExpiration> {
    let monster = region.find_monster_by_id_mut(monster_id)?;
    let tile_x = monster.move_shape().shape().get_tile_x().unwrap_or_default();
    let tile_y = monster.move_shape().shape().get_tile_y().unwrap_or_default();
    let state = monster.move_shape_mut().take_expired_battle_fairy_attribute_state(key, now_ms)?;
    Some(BattleFairyAttributeExpiration {
        target: ShapeIdentity { object_type: 600, id: monster_id, ex_id: CGuid::GUID_INVALID },
        tile_x,
        tile_y,
        state,
    })
}
