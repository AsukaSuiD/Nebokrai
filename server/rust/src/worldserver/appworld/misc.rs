//! Узкий process-global owner суточного номера копии ShengSiShiSu.
//!
//! `GetCopyNum`, `AddCopyNum`,
//! `ClearCopyNum` и
//! `RegisterClearShengSiShiSuCopyNumTime` входят в контракт owner-а.
//! `GetCopyNum` читает signed DWORD с initial value `1`. `AddCopyNum` выполняет
//! обычное 32-битное сложение с единицей без overflow gate; `AtomicI32`
//! заменяет только возможную межпоточную data race и сохраняет wrapping bits.
//!
//! Первое событие ставится на следующий день с часом/минутой/секундой `0`, но
//! сохраняет текущие milliseconds: обнуляет
//! три WORD `hour/minute/second`, но не соседний `milliseconds`. Callback
//! сначала возвращает номер к `1`,
//! затем одним новым local-time snapshot ставит следующее событие ровно через
//! `AddDay(1)`, уже сохраняя фактическое время срабатывания. Generic `CTimer`
//! заменяет function pointer/tree plumbing, но parameter `0`, выдача ID и
//! порядок reset-before-reschedule остаются исходными.

use std::sync::atomic::{AtomicI32, Ordering};

use crate::public::date::{TagTime, TagTimeArithmeticBlock};
use crate::public::timer::{CTimer, TimerId};

static COPY_NUMBER: AtomicI32 = AtomicI32::new(1);

pub(crate) fn get_copy_num() -> i32 {
    COPY_NUMBER.load(Ordering::Relaxed)
}

pub(crate) fn add_copy_num() -> i32 {
    COPY_NUMBER.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CopyNumberTimerState {
    event_id: Option<TimerId>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CopyNumberScheduleReport {
    pub(crate) scheduled_time: TagTime,
    pub(crate) event_id: TimerId,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CopyNumberResetReport {
    pub(crate) previous_number: i32,
    pub(crate) scheduled_time: TagTime,
    pub(crate) next_event_id: Option<TimerId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CopyNumberScheduleBlock {
    DateArithmetic(TagTimeArithmeticBlock),
}

impl CopyNumberTimerState {
    pub(crate) const fn event_id(&self) -> Option<TimerId> {
        self.event_id
    }

    pub(crate) const fn is_event(&self, event_id: TimerId) -> bool {
        matches!(self.event_id, Some(current) if current.get() == event_id.get())
    }

    pub(crate) fn register<Callback: Copy>(
        &mut self,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        callback: Callback,
    ) -> Result<CopyNumberScheduleReport, CopyNumberScheduleBlock> {
        let mut scheduled_time = current_time;
        scheduled_time
            .add_day(1)
            .map_err(CopyNumberScheduleBlock::DateArithmetic)?;
        scheduled_time.hour = 0;
        scheduled_time.minute = 0;
        scheduled_time.second = 0;
        let event_id = timer.set_time_event(scheduled_time, callback, 0);
        self.event_id = Some(event_id);
        Ok(CopyNumberScheduleReport {
            scheduled_time,
            event_id,
        })
    }

    pub(crate) fn prepare_reset(
        &self,
        current_time: TagTime,
    ) -> Result<CopyNumberResetReport, CopyNumberScheduleBlock> {
        let previous_number = COPY_NUMBER.swap(1, Ordering::Relaxed);
        let mut scheduled_time = current_time;
        scheduled_time
            .add_day(1)
            .map_err(CopyNumberScheduleBlock::DateArithmetic)?;
        Ok(CopyNumberResetReport {
            previous_number,
            scheduled_time,
            next_event_id: None,
        })
    }

    pub(crate) fn finish_reset(
        &mut self,
        report: &mut CopyNumberResetReport,
        event_id: TimerId,
    ) {
        self.event_id = Some(event_id);
        report.next_event_id = Some(event_id);
    }
}
