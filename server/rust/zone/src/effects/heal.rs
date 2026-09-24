//! Данные, срок и сохраняемая запись периодического лечения в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/{healstate,
//! healstate2,superhealstate,superhealstate2}.cpp/.h. Четыре класса имеют
//! одинаковый payload и различаются только ID: CHealState VA
//! 0x005F8940/0x005F89D0 (vtable 0x00660D9C, ID 0xD3), CHeal2State VA
//! 0x005EFAC0/0x005EFB50 (vtable 0x0066024C, ID 0xE3), CSuperHealState VA
//! 0x005F63B0/0x005F6440 (vtable 0x00660A64, ID 0xD9), CSuperHeal2State VA
//! 0x005EE960/0x005EE9F0 (vtable 0x00660134, ID 0xE4).
//! Все четыре vtable разделяют AI VA 0x005EEDF0, End VA 0x005EEBA0,
//! GetRemainedTime VA 0x005F2CD0, Serialize VA 0x005F65F0 и Unserialize
//! VA 0x005EEC70. Writer записывает ID, вызывает getter срока и добавляет
//! частоту и объём; reader после внешнего ID сначала читает часы и лишь
//! затем три оставшихся DWORD.

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const HEAL_STATE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealState {
    skill_id: u32,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    hp_gain: u32,
    heal_count: u32,
}

impl HealState {
    pub const fn new(skill_id: u32, keep_time_ms: u32, frequency_ms: u32, hp_gain: u32) -> Self {
        Self {
            skill_id,
            started_at_ms: 0,
            keep_time_ms,
            frequency_ms,
            hp_gain,
            heal_count: 0,
        }
    }

    pub const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    /// Restart восстанавливает visual и обнуляет счётчик, не меняя старт.
    pub fn reset_ticks(&mut self) {
        self.heal_count = 0;
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        // Unserialize читает часы до трёх оставшихся полей записи.
        let started_at_ms = now();
        let keep_time_ms = reader.read_u32()?;
        let frequency_ms = reader.read_u32()?;
        let hp_gain = reader.read_u32()?;
        Ok(Self {
            skill_id,
            started_at_ms,
            keep_time_ms,
            frequency_ms,
            hp_gain,
            heal_count: 0,
        })
    }

    pub fn encoded(self, now: impl FnMut() -> u32) -> [u8; HEAL_STATE_BYTES] {
        self.encoded_with_remaining(self.client_state_time(now))
    }

    pub fn encoded_for_install(self) -> [u8; HEAL_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; HEAL_STATE_BYTES] {
        // Serialize записывает ID до вызова GetRemainedTime.
        let mut bytes = [0; HEAL_STATE_BYTES];
        bytes[0..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining_time_ms.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.frequency_ms.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.hp_gain.to_le_bytes());
        bytes
    }

    pub fn client_state_time(self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    pub fn client_time(self, now: impl FnMut() -> u32) -> i32 {
        self.client_state_time(now) as i32
    }

    /// Строгий unsigned wrapping срок; равенство границе ещё активно.
    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    /// Тик по расписанию `start + frequency * count`: равенство не наступает.
    /// Счётчик увеличивается до чтения HP и максимума вызывающим.
    pub fn take_due_gain(&mut self, now_ms: u32) -> Option<u32> {
        if self
            .started_at_ms
            .wrapping_add(self.frequency_ms.wrapping_mul(self.heal_count))
            >= now_ms
        {
            return None;
        }
        self.heal_count = self.heal_count.wrapping_add(1);
        Some(self.hp_gain)
    }
}
