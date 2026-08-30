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

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const ATTRIBUTE_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const ATTRIBUTE_STATE_END_MESSAGE: i32 = 0x000b_fe04;

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

pub(crate) fn expire_player_battle_fairy_attribute_states<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
    runtime: &mut Runtime,
) -> usize {
    let states = game
        .find_player_mut(player_id)
        .map(|player| player.take_expired_battle_fairy_attribute_states(now_ms))
        .unwrap_or_default();
    let ended = states.len();
    if states.is_empty() {
        return 0;
    }
    let Some((region_id, tile_x, tile_y)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        ))
    }) else {
        return ended;
    };
    let target = ShapeIdentity {
        object_type: 400,
        id: player_id,
        ex_id: CGuid::GUID_INVALID,
    };
    for state in states {
        send_battle_fairy_attribute_state_visual(
            game, region_id, target, tile_x, tile_y, state, false,
        );
    }
    let _ = game.update_player_properties(player_id, runtime);
    ended
}

pub(crate) struct BattleFairyAttributeExpiration {
    target: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    states: Vec<BattleFairyAttributeState>,
}

impl BattleFairyAttributeExpiration {
    pub(crate) fn deliver(self, game: &mut CGame, region_id: i32) -> usize {
        let ended = self.states.len();
        for state in self.states {
            send_battle_fairy_attribute_state_visual(
                game,
                region_id,
                self.target,
                self.tile_x,
                self.tile_y,
                state,
                false,
            );
        }
        ended
    }
}

pub(crate) fn take_expired_monster_battle_fairy_attribute_states(
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> Option<BattleFairyAttributeExpiration> {
    region.find_monster_by_id_mut(monster_id).map(|monster| {
        let tile_x = monster
            .move_shape()
            .shape()
            .get_tile_x()
            .unwrap_or_default();
        let tile_y = monster
            .move_shape()
            .shape()
            .get_tile_y()
            .unwrap_or_default();
        let states = monster
            .move_shape_mut()
            .take_expired_battle_fairy_attribute_states(now_ms);
        BattleFairyAttributeExpiration {
            target: ShapeIdentity {
                object_type: 600,
                id: monster_id,
                ex_id: CGuid::GUID_INVALID,
            },
            tile_x,
            tile_y,
            states,
        }
    })
}
