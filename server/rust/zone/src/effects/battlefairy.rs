//! Данные, срок и 12-байтная запись состояний боевой феи Po/Yu в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/skills/{pojia,pobing,pomo,pofa,yujia,yubing,yumo,yufa}state.cpp/.h.
//! Конструкторы записывают свой ID без чтения часов. Все восемь классов
//! разделяют Serialize (ID, затем остаток через getter, затем
//! signed-значение), Unserialize (часы после внешнего ID и до срока со
//! значением), GetRemainedTime, AI (завершение только при
//! `now > start + keep`) и End.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone
//! Отображение вида (Po снижает цель, Yu усиливает держателя) подтверждено
//! различием вызовов в Begin восьми классов; пофункциональная сверка всех
//! property callbacks не выполнялась — числовые формулы перенесены из
//! существующего Rust без изменения (PARTIAL).

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyAttributeKind {
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
    /// Yu-состояния живут на держателе и меняют его самого.
    pub const fn targets_self(self) -> bool {
        matches!(
            self,
            Self::AttackAvoidGain
                | Self::AttackGain
                | Self::ElementModifyGain
                | Self::ElementAvoidGain
        )
    }
}

/// Соответствие ID состояния его виду: 0x212..0x215 — Po, 0x216..0x219 — Yu.
pub const fn battle_fairy_attribute_kind(skill_id: u32) -> Option<BattleFairyAttributeKind> {
    match skill_id {
        0x212 => Some(BattleFairyAttributeKind::AttackAvoidLoss),
        0x213 => Some(BattleFairyAttributeKind::AttackLoss),
        0x214 => Some(BattleFairyAttributeKind::ElementModifyLoss),
        0x215 => Some(BattleFairyAttributeKind::ElementAvoidLoss),
        0x216 => Some(BattleFairyAttributeKind::AttackAvoidGain),
        0x217 => Some(BattleFairyAttributeKind::AttackGain),
        0x218 => Some(BattleFairyAttributeKind::ElementModifyGain),
        0x219 => Some(BattleFairyAttributeKind::ElementAvoidGain),
        _ => None,
    }
}

/// Пять свойств игрока, которые меняют формулы состояния.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleFairyAttributePlayerView {
    pub attack_avoid: u16,
    pub element_avoid: u16,
    pub minimum_attack: u32,
    pub maximum_attack: u32,
    pub element_modify: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleFairyAttributeState {
    skill_id: u32,
    kind: BattleFairyAttributeKind,
    started_at_ms: u32,
    keep_time_ms: u32,
    value: i32,
}

impl BattleFairyAttributeState {
    pub const fn new(
        skill_id: u32,
        kind: BattleFairyAttributeKind,
        keep_time_ms: u32,
        value: i32,
    ) -> Self {
        Self {
            skill_id,
            kind,
            started_at_ms: 0,
            keep_time_ms,
            value,
        }
    }

    pub fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub const fn kind(self) -> BattleFairyAttributeKind {
        self.kind
    }

    pub const fn keep_time_ms(self) -> u32 {
        self.keep_time_ms
    }

    pub const fn value(self) -> i32 {
        self.value
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let kind = battle_fairy_attribute_kind(skill_id).ok_or(LegacyReadBlock {
            offset,
            needed: 4,
            available: payload.len().saturating_sub(offset),
        })?;
        // Unserialize читает часы после внешнего ID и до срока со значением.
        let started_at_ms = now();
        let keep_time_ms = reader.read_u32()?;
        let value = reader.read_i32()?;
        Ok(Self {
            skill_id,
            kind,
            started_at_ms,
            keep_time_ms,
            value,
        })
    }

    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_state_time(self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES] {
        self.encoded_with_remaining(|| self.client_state_time(now))
    }

    pub fn encoded_for_install(self) -> [u8; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES] {
        self.encoded_with_remaining(|| self.keep_time_ms)
    }

    fn encoded_with_remaining(
        self,
        remaining: impl FnOnce() -> u32,
    ) -> [u8; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES] {
        // Serialize записывает ID до вызова GetRemainedTime.
        let mut bytes = [0; BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining().to_le_bytes());
        bytes[8..].copy_from_slice(&self.value.to_le_bytes());
        bytes
    }

    /// Формулы восьми видов для игрока: Po вычитают с нижней границей 0,
    /// Yu прибавляют (атака — с верхней границей INT_MAX).
    pub fn apply_to_player_view(
        self,
        mut view: BattleFairyAttributePlayerView,
    ) -> BattleFairyAttributePlayerView {
        match self.kind {
            BattleFairyAttributeKind::AttackAvoidLoss => {
                let next = i32::from(view.attack_avoid).wrapping_sub(self.value);
                view.attack_avoid = if next < 1 { 0 } else { next as u16 };
            }
            BattleFairyAttributeKind::AttackLoss => {
                view.minimum_attack = subtract_player_attack(view.minimum_attack, self.value);
                view.maximum_attack = subtract_player_attack(view.maximum_attack, self.value);
            }
            BattleFairyAttributeKind::ElementModifyLoss => {
                let next = view.element_modify.wrapping_sub(self.value);
                view.element_modify = if next < 1 { 0 } else { next };
            }
            BattleFairyAttributeKind::ElementAvoidLoss => {
                let next = i32::from(view.element_avoid).wrapping_sub(self.value);
                view.element_avoid = if next < 1 { 0 } else { next as u16 };
            }
            BattleFairyAttributeKind::AttackAvoidGain => {
                view.attack_avoid = view.attack_avoid.wrapping_add(self.value as u16);
            }
            BattleFairyAttributeKind::AttackGain => {
                view.minimum_attack = add_player_attack(view.minimum_attack, self.value);
                view.maximum_attack = add_player_attack(view.maximum_attack, self.value);
            }
            BattleFairyAttributeKind::ElementModifyGain => {
                view.element_modify = view.element_modify.wrapping_add(self.value);
            }
            BattleFairyAttributeKind::ElementAvoidGain => {
                view.element_avoid = view.element_avoid.wrapping_add(self.value as u16);
            }
        }
        view
    }

    /// Среди монстровых формул существует только снижение модификатора Pomo.
    pub const fn apply_to_monster_element(self, value: i32) -> i32 {
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
    if next > i32::MAX as u32 {
        i32::MAX as u32
    } else {
        next
    }
}
