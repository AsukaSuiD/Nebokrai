//! Канонические состояния семейства боевого духа `0x212..0x219`.
//!
//! Источник: `gameserver.exe` и `GameServer.pdb`, владельцы
//! `pojia/pobing/pomo/pofa/yujia/yubing/yumo/yufa state`. Порядок состояний
//! остаётся порядком исходного `m_vStates`: замена удаляет прежнюю запись и
//! добавляет новую в хвост. Пакет начала передаёт исходные длительность и
//! отметку времени, а не вычисленный остаток. Формулы игрока и монстра
//! разделены, поскольку часть ослаблений в оригинале не поддерживала монстров.

use crate::gameserver::appserver::player::PlayerCombatProperties;

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
