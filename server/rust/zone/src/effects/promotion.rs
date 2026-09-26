//! Данные, срок и сохраняемая запись CPromotionState (0x142) в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/promotionstate.cpp/.h.
//! Конструктор с параметрами записывает срок и оба
//! WORD-коэффициента из аргументов без чтения часов; конструктор по
//! умолчанию задаёт оба коэффициента равными 1; Restart обновляет
//! только часы. Serialize пишет ID, вызывает getter срока
//! и добавляет оба WORD (атака +0x3C, лечение +0x3E); Unserialize
//! после внешнего ID читает часы и затем три поля записи.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const PROMOTION_STATE_ID: u32 = 0x142;
pub const PROMOTION_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromotionState {
    started_at_ms: u32,
    keep_time_ms: u32,
    magic_attack_factor: u16,
    heal_recover_factor: u16,
}

impl PromotionState {
    pub const fn new(
        keep_time_ms: u32,
        magic_attack_factor: u16,
        heal_recover_factor: u16,
    ) -> Self {
        Self {
            started_at_ms: 0,
            keep_time_ms,
            magic_attack_factor,
            heal_recover_factor,
        }
    }

    pub const fn skill_id(self) -> u32 {
        PROMOTION_STATE_ID
    }

    /// Restart меняет только отметку начала: срок и коэффициенты сохраняются.
    pub fn restart(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn magic_attack_factor(self) -> u16 {
        self.magic_attack_factor
    }

    pub const fn heal_recover_factor(self) -> u16 {
        self.heal_recover_factor
    }

    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_state_time(self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    pub fn client_time(self, now: impl FnMut() -> u32) -> i32 {
        self.client_state_time(now) as i32
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != PROMOTION_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        // Unserialize читает часы после внешнего ID и до полей срока.
        let started_at_ms = now();
        let keep_time_ms = reader.read_u32()?;
        let magic_attack_factor = reader.read_u16()?;
        let heal_recover_factor = reader.read_u16()?;
        Ok(Self {
            started_at_ms,
            keep_time_ms,
            magic_attack_factor,
            heal_recover_factor,
        })
    }

    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; PROMOTION_STATE_BYTES] {
        self.encoded_with_remaining(self.client_state_time(now))
    }

    pub fn encoded_for_install(self) -> [u8; PROMOTION_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; PROMOTION_STATE_BYTES] {
        // Serialize записывает ID до вызова GetRemainedTime.
        let mut bytes = [0; PROMOTION_STATE_BYTES];
        bytes[0..4].copy_from_slice(&PROMOTION_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining_time_ms.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.magic_attack_factor.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.heal_recover_factor.to_le_bytes());
        bytes
    }
}

/// Стихийный множитель Promotion в `CFightDefense::PreDefense` VA
/// `0x005B0C80`: только `kind == 3` (Element); FILD WORD-коэффициента,
/// FIMUL signed-урона, FMUL f32-константы `0.001`, FISTP-усечение обратно.
pub fn promotion_element_attack(factor: u16, damage: i32) -> i32 {
    crate::combat::truncate_original(f64::from(factor) * f64::from(damage) * f64::from(0.001_f32))
}

impl PromotionState {
    pub fn apply_element_attack(self, damage: i32) -> i32 {
        promotion_element_attack(self.magic_attack_factor(), damage)
    }
}
