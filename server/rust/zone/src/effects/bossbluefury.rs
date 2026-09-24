//! Данные, срок и 12-байтная запись CBossBlueFuryState (0x1F7) в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/skills/bossbluefurystate.cpp/.h.
//! Конструктор VA 0x005E8A60 записывает срок (+0x38), коэффициент атаки
//! (+0x3C) и срок слабой фазы (+0x40) из аргументов без чтения часов; ID
//! 0x1F7. Vtable 0x0065F994: AI VA 0x005E8D50 (отдельные часы перед слабой
//! и общей границами; после слабой снимает оба запрета на каждом проходе),
//! End VA 0x005E8D10, Restart VA 0x005FD450 (только часы), OnUpdateProperties
//! VA 0x005E8DC0, GetRemainedTime VA 0x005D5F30, Serialize VA 0x005E7330
//! (ID, затем остаток через getter, затем signed-коэффициент), Unserialize
//! VA 0x005D6190 (часы после внешнего ID, до срока и коэффициента; weak_time
//! не читается и остаётся нулевым).

use super::time::timed_client_state_time;
use crate::combat::truncate_original;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const BOSS_BLUE_FURY_STATE_ID: u32 = 0x1f7;
pub const BOSS_BLUE_FURY_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BossBlueFuryState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_factor_percent: i32,
    weak_time_ms: u32,
}

impl BossBlueFuryState {
    pub const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        attack_factor_percent: i32,
        weak_time_ms: u32,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            attack_factor_percent,
            weak_time_ms,
        }
    }

    pub const fn skill_id(self) -> u32 {
        BOSS_BLUE_FURY_STATE_ID
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
        if reader.read_u32()? != BOSS_BLUE_FURY_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        // Unserialize читает часы после внешнего ID и до срока с коэффициентом.
        let started_at_ms = now();
        let keep_time_ms = reader.read_u32()?;
        let attack_factor_percent = reader.read_i32()?;
        Ok(Self::new(
            started_at_ms,
            keep_time_ms,
            attack_factor_percent,
            0,
        ))
    }

    pub fn encoded(self, now_ms: u32) -> [u8; BOSS_BLUE_FURY_STATE_BYTES] {
        // Serialize записывает ID до вызова GetRemainedTime.
        let mut bytes = [0; BOSS_BLUE_FURY_STATE_BYTES];
        bytes[0..4].copy_from_slice(&BOSS_BLUE_FURY_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.client_state_time(|| now_ms).to_le_bytes());
        bytes[8..12].copy_from_slice(&self.attack_factor_percent.to_le_bytes());
        bytes
    }

    pub fn encoded_for_install(self) -> [u8; BOSS_BLUE_FURY_STATE_BYTES] {
        self.encoded(self.started_at_ms)
    }

    pub const fn weak_elapsed(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.weak_time_ms) < now_ms
    }

    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    /// Процент от живого getter атаки монстра: signed-коэффициент и unsigned
    /// атака перемножаются в x87 с `0.01_f32`, результат усекается в i32.
    pub fn attack_modifier(self, attack: u32) -> i32 {
        truncate_original(
            f64::from(self.attack_factor_percent) * f64::from(0.01_f32) * f64::from(attack),
        )
    }
}
