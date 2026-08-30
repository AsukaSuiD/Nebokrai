//! Упорядоченный владелец состояний восстановления от расходуемых предметов.
//!
//! `CPlayer::RestoreHp/RestoreMp` имеют независимые интервалы применения, но
//! созданные `CRestoreHpState/CRestoreMpState` входят в один живой список
//! `CMoveShape`. Здесь сохранены общий порядок вставки и отдельные отметки
//! задержки повторного использования; сами формулы и сроки делегируются
//! исходным владельцам состояний.

use super::restorehpstate::RestoreHpState;
use super::restorempstate::RestoreMpState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConsumableRestoreMutation {
    Health(u32),
    Mana(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConsumableRestoreState {
    Health(RestoreHpState),
    Mana(RestoreMpState),
}

impl ConsumableRestoreState {
    const fn state_id(self) -> i32 {
        match self {
            Self::Health(state) => state.state_id(),
            Self::Mana(state) => state.state_id(),
        }
    }

    fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        match self {
            Self::Health(state) => state.client_state_time(now_milliseconds),
            Self::Mana(state) => state.client_state_time(now_milliseconds),
        }
    }

    fn tick(self, checked_at_ms: u32, current: u32, maximum: u32) -> (Self, Option<ConsumableRestoreMutation>) {
        match self {
            Self::Health(mut state) => {
                let value = state.tick(checked_at_ms, current, maximum);
                (Self::Health(state), value.map(ConsumableRestoreMutation::Health))
            }
            Self::Mana(mut state) => {
                let value = state.tick(checked_at_ms, current, maximum);
                (Self::Mana(state), value.map(ConsumableRestoreMutation::Mana))
            }
        }
    }

    const fn expired(self, checked_at_ms: u32) -> bool {
        match self {
            Self::Health(state) => state.expired(checked_at_ms),
            Self::Mana(state) => state.expired(checked_at_ms),
        }
    }

    const fn is_health(self) -> bool {
        matches!(self, Self::Health(_))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ConsumableRestoreStateStorage {
    states: Vec<ConsumableRestoreState>,
    last_health_begin_ms: u32,
    last_mana_begin_ms: u32,
}

impl ConsumableRestoreStateStorage {
    pub(crate) fn begin_health(
        &mut self,
        amount: u32,
        time_to_keep_ms: u32,
        frequency_ms: u32,
        interval_ms: u32,
        mut now_ms: impl FnMut() -> u32,
    ) -> bool {
        let checked_at_ms = now_ms();
        if self.last_health_begin_ms.wrapping_add(interval_ms) > checked_at_ms {
            return false;
        }
        let started_at_ms = now_ms();
        self.last_health_begin_ms = started_at_ms;
        self.states.push(ConsumableRestoreState::Health(RestoreHpState::new(
            time_to_keep_ms,
            frequency_ms,
            amount,
            started_at_ms,
        )));
        true
    }

    pub(crate) fn begin_mana(
        &mut self,
        amount: u32,
        time_to_keep_ms: u32,
        frequency_ms: u32,
        interval_ms: u32,
        mut now_ms: impl FnMut() -> u32,
    ) -> bool {
        let checked_at_ms = now_ms();
        if self.last_mana_begin_ms.wrapping_add(interval_ms) > checked_at_ms {
            return false;
        }
        let started_at_ms = now_ms();
        self.last_mana_begin_ms = started_at_ms;
        self.states.push(ConsumableRestoreState::Mana(RestoreMpState::new(
            time_to_keep_ms,
            frequency_ms,
            amount,
            started_at_ms,
        )));
        true
    }

    pub(crate) const fn len(&self) -> usize {
        self.states.len()
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub(crate) fn count(&self, state_id: i32) -> usize {
        self.states
            .iter()
            .filter(|state| state.state_id() == state_id)
            .count()
    }

    pub(crate) fn contains(&self, state_id: u32) -> bool {
        self.states
            .iter()
            .any(|state| state.state_id() as u32 == state_id)
    }

    pub(crate) fn is_health(&self, index: usize) -> Option<bool> {
        self.states.get(index).copied().map(ConsumableRestoreState::is_health)
    }

    pub(crate) fn client_state_time(
        &self,
        index: usize,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<i32> {
        self.states
            .get(index)
            .copied()
            .map(|state| state.client_state_time(now_milliseconds))
    }

    pub(crate) fn tick(
        &mut self,
        index: usize,
        checked_at_ms: u32,
        current: u32,
        maximum: u32,
    ) -> Option<ConsumableRestoreMutation> {
        let state = *self.states.get(index)?;
        let (state, mutation) = state.tick(checked_at_ms, current, maximum);
        self.states[index] = state;
        mutation
    }

    pub(crate) fn expired(&self, index: usize, checked_at_ms: u32) -> Option<bool> {
        self.states.get(index).copied().map(|state| state.expired(checked_at_ms))
    }

    pub(crate) fn remove(&mut self, index: usize) -> bool {
        if index >= self.states.len() {
            return false;
        }
        self.states.remove(index);
        true
    }
}
