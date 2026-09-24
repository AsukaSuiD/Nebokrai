//! Данные, таймер и 16-байтная запись состояний восстановления от
//! расходуемых предметов в Zone: `CRestoreHpState` (ID `100000`) и
//! `CRestoreMpState` (ID `100001`).
//! Источник: gameserver.exe + GameServer.pdb, appserver/player.cpp и
//! appserver/other states/{restorehpstate,restorempstate}.cpp/.h.
//! Конструкторы VA `0x004F8410`/`0x004F87B0` копируют keep/frequency/gain,
//! оставляя count и timestamp нулевыми без чтения часов. Vtable
//! `0x0065355C`/`0x006535BC` разделяют с семейством Heal Serialize
//! `0x005F65F0` (ID, остаток через getter, frequency, gain — четыре DWORD,
//! живой keep не меняется), Unserialize `0x005EEC70` (свой clock после
//! внешнего ID, до трёх полей), GetRemainedTime `0x005F2CD0` и
//! additional `0x00601200` (ноль). AI VA `0x004F8650`/`0x004F8AA0`:
//! один шаг при строгом `frequency * count + started < now`, count
//! увеличивается до записи ресурса, второй clock проверяет
//! `keep + started < now`; смерть приостанавливает оба действия.
//! Живые Begin/restart/AI/End и применение ресурса остаются у
//! переходного Game.

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const CONSUMABLE_RESTORE_STATE_BYTES: usize = 16;
pub const RESTORE_HP_STATE_ID: i32 = 100_000;
pub const RESTORE_MP_STATE_ID: i32 = 100_001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreStateData<const ID: i32> {
    time_to_keep_ms: u32,
    frequency_ms: u32,
    gain: u32,
    restore_count: u32,
    started_at_ms: u32,
}

impl<const ID: i32> RestoreStateData<ID> {
    pub const fn new(time_to_keep_ms: u32, frequency_ms: u32, gain: u32) -> Self {
        Self {
            time_to_keep_ms,
            frequency_ms,
            gain,
            restore_count: 0,
            started_at_ms: 0,
        }
    }

    pub const fn state_id(&self) -> i32 {
        ID
    }

    /// Unserialize читает свой clock после внешнего ID и до трёх полей.
    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let _state_id = reader.read_i32()?;
        let started_at_ms = now();
        let mut state = Self::new(reader.read_u32()?, reader.read_u32()?, reader.read_u32()?);
        state.started_at_ms = started_at_ms;
        Ok(state)
    }

    pub fn begin_primary_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
        self.reset_restore_count();
    }

    pub fn reset_restore_count(&mut self) {
        self.restore_count = 0;
    }

    /// Один шаг при строгом расписании; счётчик растёт до записи ресурса.
    pub fn take_due_gain(&mut self, checked_at_ms: u32) -> Option<u32> {
        let due_at_ms = self
            .frequency_ms
            .wrapping_mul(self.restore_count)
            .wrapping_add(self.started_at_ms);
        if due_at_ms >= checked_at_ms {
            return None;
        }
        self.restore_count = self.restore_count.wrapping_add(1);
        Some(self.gain)
    }

    pub const fn expired(&self, checked_at_ms: u32) -> bool {
        self.time_to_keep_ms.wrapping_add(self.started_at_ms) < checked_at_ms
    }

    pub fn client_state_time(&self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.time_to_keep_ms, now) as i32
    }

    /// Serialize пишет ID до вызова GetRemainedTime и не меняет живой keep.
    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        self.encoded_with_remaining(self.client_state_time(now) as u32)
    }

    pub fn encoded_for_install(&self) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        self.encoded_with_remaining(self.time_to_keep_ms)
    }

    fn encoded_with_remaining(&self, remaining: u32) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        let mut record = [0; CONSUMABLE_RESTORE_STATE_BYTES];
        for (slot, value) in
            record
                .chunks_exact_mut(4)
                .zip([ID as u32, remaining, self.frequency_ms, self.gain])
        {
            slot.copy_from_slice(&value.to_le_bytes());
        }
        record
    }
}

pub type RestoreHpState = RestoreStateData<RESTORE_HP_STATE_ID>;
pub type RestoreMpState = RestoreStateData<RESTORE_MP_STATE_ID>;

/// HP и MP остаются разными вариантами и отдельными записями общей арены.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsumableRestoreState {
    Health(RestoreHpState),
    Mana(RestoreMpState),
}

impl ConsumableRestoreState {
    pub fn decode(payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32) -> Option<Self> {
        match payload.get(offset..offset.checked_add(4)?)? {
            id if id == RESTORE_HP_STATE_ID.to_le_bytes() => {
                RestoreHpState::decode(payload, offset, now)
                    .ok()
                    .map(Self::Health)
            }
            id if id == RESTORE_MP_STATE_ID.to_le_bytes() => {
                RestoreMpState::decode(payload, offset, now)
                    .ok()
                    .map(Self::Mana)
            }
            _ => None,
        }
    }

    pub const fn state_id(&self) -> i32 {
        match self {
            Self::Health(state) => state.state_id(),
            Self::Mana(state) => state.state_id(),
        }
    }

    pub fn begin_primary_at(&mut self, now_ms: u32) {
        match self {
            Self::Health(state) => state.begin_primary_at(now_ms),
            Self::Mana(state) => state.begin_primary_at(now_ms),
        }
    }

    pub fn reset_restore_count(&mut self) {
        match self {
            Self::Health(state) => state.reset_restore_count(),
            Self::Mana(state) => state.reset_restore_count(),
        }
    }

    pub fn take_due_gain(&mut self, checked_at_ms: u32) -> Option<u32> {
        match self {
            Self::Health(state) => state.take_due_gain(checked_at_ms),
            Self::Mana(state) => state.take_due_gain(checked_at_ms),
        }
    }

    pub fn client_state_time(&self, now: impl FnMut() -> u32) -> i32 {
        match self {
            Self::Health(state) => state.client_state_time(now),
            Self::Mana(state) => state.client_state_time(now),
        }
    }

    pub const fn expired(&self, checked_at_ms: u32) -> bool {
        match self {
            Self::Health(state) => state.expired(checked_at_ms),
            Self::Mana(state) => state.expired(checked_at_ms),
        }
    }

    pub const fn is_health(&self) -> bool {
        matches!(self, Self::Health(_))
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        match self {
            Self::Health(state) => state.encoded(now),
            Self::Mana(state) => state.encoded(now),
        }
    }

    pub fn encoded_for_install(&self) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        match self {
            Self::Health(state) => state.encoded_for_install(),
            Self::Mana(state) => state.encoded_for_install(),
        }
    }
}

/// Отдельные интервалы повторного применения игрока; Decode/Clear состояний
/// их не сбрасывает.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConsumableRestoreIntervals {
    last_health_begin_ms: u32,
    last_mana_begin_ms: u32,
}

impl ConsumableRestoreIntervals {
    pub fn try_begin(
        &mut self,
        health: bool,
        interval_ms: u32,
        mut now: impl FnMut() -> u32,
    ) -> bool {
        let checked_at_ms = now();
        let last_begin_ms = if health {
            &mut self.last_health_begin_ms
        } else {
            &mut self.last_mana_begin_ms
        };
        if last_begin_ms.wrapping_add(interval_ms) > checked_at_ms {
            return false;
        }
        *last_begin_ms = now();
        true
    }
}
