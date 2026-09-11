//! Состояния восстановления от расходуемых предметов и интервалы применения.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! владельцы appserver/player.cpp и appserver/other states/restorehpstate.cpp,
//! restorempstate.cpp; этот enum лишь объединяет два подтверждённых состояния.
//!
//! `CPlayer::RestoreHp/RestoreMp` имеют независимые интервалы применения, но
//! созданные `CRestoreHpState/CRestoreMpState` входят в один живой список
//! `CMoveShape`. Сами экземпляры принадлежат общему AppliedStateEntries;
//! здесь остаются варианты состояния и отдельные отметки задержки повторного
//! использования, не являющиеся состояниями. Формулы и сроки делегируются
//! исходным владельцам состояний. Полный клиентский снимок обходит тот же
//! список и вычисляет remaining-time отдельными исходными чтениями часов.
//! Техническое SlotMap-хранилище не меняет порядок HP/MP-записей, повторные
//! экземпляры, два чтения часов успешного Begin и сброс интервалов при загрузке.
//! Общий AI работает с одним живым ключом этой арены. HP сохраняет generic
//! CMoveShape health/OnChangeStates и death-pause; MP non-player немедленно
//! заканчивается без часов. Во время публикации payload остаётся у владельца,
//! после неё срок проверяется по тому же поколенческому ключу.

use super::restorehpstate::{RESTORE_HP_STATE_ID, RestoreHpState};
use super::restorempstate::{RESTORE_MP_STATE_ID, RestoreMpState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConsumableRestoreMutation {
    Health(u32),
    Mana(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConsumableRestoreState {
    Health(RestoreHpState),
    Mana(RestoreMpState),
}

impl ConsumableRestoreState {
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Option<Self> {
        match payload.get(offset..offset.checked_add(4)?)? {
            id if id == RESTORE_HP_STATE_ID.to_le_bytes() => {
                RestoreHpState::decode(payload, offset).ok().map(Self::Health)
            }
            id if id == RESTORE_MP_STATE_ID.to_le_bytes() => {
                RestoreMpState::decode(payload, offset).ok().map(Self::Mana)
            }
            _ => None,
        }
    }

    pub(crate) const fn state_id(self) -> i32 {
        match self {
            Self::Health(state) => state.state_id(),
            Self::Mana(state) => state.state_id(),
        }
    }

    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        match self {
            Self::Health(state) => state.client_state_time(now_milliseconds),
            Self::Mana(state) => state.client_state_time(now_milliseconds),
        }
    }

    pub(crate) fn tick(
        &mut self,
        checked_at_ms: u32,
        current: u32,
        maximum: u32,
    ) -> Option<ConsumableRestoreMutation> {
        match self {
            Self::Health(state) => state
                .tick(checked_at_ms, current, maximum)
                .map(ConsumableRestoreMutation::Health),
            Self::Mana(state) => state
                .tick(checked_at_ms, current, maximum)
                .map(ConsumableRestoreMutation::Mana),
        }
    }

    pub(crate) const fn expired(self, checked_at_ms: u32) -> bool {
        match self {
            Self::Health(state) => state.expired(checked_at_ms),
            Self::Mana(state) => state.expired(checked_at_ms),
        }
    }

    pub(crate) const fn is_health(self) -> bool {
        matches!(self, Self::Health(_))
    }

    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; 16] {
        match self {
            Self::Health(state) => state.encoded(now_milliseconds),
            Self::Mana(state) => state.encoded(now_milliseconds),
        }
    }

    pub(crate) fn encoded_for_install(self) -> [u8; 16] {
        match self {
            Self::Health(state) => state.encoded_for_install(),
            Self::Mana(state) => state.encoded_for_install(),
        }
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        match self {
            Self::Health(state) => state.activate_loaded(now_ms),
            Self::Mana(state) => state.activate_loaded(now_ms),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ConsumableRestoreIntervals {
    last_health_begin_ms: u32,
    last_mana_begin_ms: u32,
}

impl ConsumableRestoreIntervals {
    pub(crate) fn begin_health(
        &mut self,
        amount: u32,
        time_to_keep_ms: u32,
        frequency_ms: u32,
        interval_ms: u32,
        mut now_ms: impl FnMut() -> u32,
    ) -> Option<ConsumableRestoreState> {
        let checked_at_ms = now_ms();
        if self.last_health_begin_ms.wrapping_add(interval_ms) > checked_at_ms {
            return None;
        }
        let started_at_ms = now_ms();
        self.last_health_begin_ms = started_at_ms;
        Some(ConsumableRestoreState::Health(RestoreHpState::new(
            time_to_keep_ms,
            frequency_ms,
            amount,
            started_at_ms,
        )))
    }

    pub(crate) fn begin_mana(
        &mut self,
        amount: u32,
        time_to_keep_ms: u32,
        frequency_ms: u32,
        interval_ms: u32,
        mut now_ms: impl FnMut() -> u32,
    ) -> Option<ConsumableRestoreState> {
        let checked_at_ms = now_ms();
        if self.last_mana_begin_ms.wrapping_add(interval_ms) > checked_at_ms {
            return None;
        }
        let started_at_ms = now_ms();
        self.last_mana_begin_ms = started_at_ms;
        Some(ConsumableRestoreState::Mana(RestoreMpState::new(
            time_to_keep_ms,
            frequency_ms,
            amount,
            started_at_ms,
        )))
    }
}
