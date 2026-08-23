//! Calendar и interval-owner `CTimer` WorldServer из `public/timer.cpp/.h`.
//! Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Два ordered registry сохраняют wrapping tick arithmetic, callback-before-
//! erase и возможность увидеть добавленный callback-ом больший ID в том же
//! проходе. Calendar сравнивает исходный `TagTime`, real/AI timers используют
//! разные tick sources. `BTreeMap`, typed IDs и dispatcher заменяют MSVC map,
//! function pointers и singleton lifetime без изменения порядка эффектов.

use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::public::date::TagTime;

static NEXT_TIMER_ID: AtomicU32 = AtomicU32::new(0);

/// Оригинал unsigned identity, которую старый owner выдавал всем типам timers.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct TimerId(u32);

impl TimerId {
    pub(crate) const fn from_raw(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u32 {
        self.0
    }
}

/// Доказанные PDB-поля одной периодической записи.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PeriodicTimer<Callback> {
    pub(crate) id: TimerId,
    pub(crate) open: bool,
    pub(crate) use_ai_tick: bool,
    pub(crate) elapsed_time_ms: u32,
    pub(crate) start_time_ms: u32,
    pub(crate) elapsed_ai_tick: u32,
    pub(crate) start_ai_tick: u32,
    pub(crate) parameter: i32,
    pub(crate) callback: Callback,
}

/// Источник одного синхронного вызова старого callback pointer-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimerCallbackSource {
    Periodic(TimerId),
    Calendar(TimerId),
}

/// Узкий typed envelope старых callback pointer и `long` parameter.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TimerCallbackInvocation<Callback> {
    pub(crate) source: TimerCallbackSource,
    pub(crate) callback: Callback,
    pub(crate) parameter: i32,
}

/// Наблюдаемый итог одного полного `CTimer::Run`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TimerRunReport {
    pub(crate) periodic_callbacks: u32,
    pub(crate) calendar_callbacks: u32,
}

/// Новое calendar-событие, которое async domain-callback просит поставить до
/// удаления текущей записи.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CalendarTimerRegistration<Callback> {
    pub(crate) time: TagTime,
    pub(crate) callback: Callback,
    pub(crate) parameter: i32,
}

/// Решение async handler-а для одного уже действующего callback-а.
#[derive(Clone, Copy, Debug)]
pub(crate) enum AsyncTimerCallbackDisposition<Callback> {
    PassThrough,
    Handled {
        next_calendar_event: Option<CalendarTimerRegistration<Callback>>,
    },
}

/// Выполненный prefix `Run` перед локальной safe-границей domain callback-а.
#[derive(Debug)]
pub(crate) struct AsyncTimerRunBlock<Block> {
    pub(crate) timer: TimerRunReport,
    pub(crate) source: Block,
}

/// Async adapter для исходно синхронных callback-ов, которые теперь достигают
/// библиотечного DB owner-а. Остальные callbacks остаются в sync dispatcher-е.
pub(crate) trait AsyncTimerCallbackHandler<Callback, GetTick, GetLocalTime> {
    type Block;

    async fn dispatch(
        &mut self,
        timer: &mut CTimer<Callback>,
        invocation: TimerCallbackInvocation<Callback>,
        get_tick: &mut GetTick,
        get_local_time: &mut GetLocalTime,
    ) -> Result<AsyncTimerCallbackDisposition<Callback>, Self::Block>;

    fn calendar_event_registered(
        &mut self,
        _invocation: TimerCallbackInvocation<Callback>,
        _event_id: TimerId,
    ) {
    }
}

#[derive(Clone, Copy, Debug)]
struct TimeEvent<Callback> {
    id: TimerId,
    time: TagTime,
    parameter: i32,
    callback: Callback,
}

/// Owned форма единственного исходного `CTimer` singleton-а.
#[derive(Debug)]
pub(crate) struct CTimer<Callback> {
    periodic_timers: BTreeMap<TimerId, PeriodicTimer<Callback>>,
    time_events: BTreeMap<TimerId, TimeEvent<Callback>>,
}

impl<Callback> Default for CTimer<Callback> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Callback> CTimer<Callback> {
 /// Создаёт оба пустых ordered registry старого constructor-а.
    pub(crate) const fn new() -> Self {
        Self {
            periodic_timers: BTreeMap::new(),
            time_events: BTreeMap::new(),
        }
    }

 /// Вставляет уже доказанную PDB-запись, не моделируя не относящийся к контракту inline `SetTimer`.
    pub(crate) fn insert_periodic_record(
        &mut self,
        timer: PeriodicTimer<Callback>,
    ) -> Option<PeriodicTimer<Callback>> {
        self.periodic_timers.insert(timer.id, timer)
    }

 /// Повторяет копирование `SetTimeEvent` и возвращает wrapping process ID.
    pub(crate) fn set_time_event(
        &mut self,
        time: TagTime,
        callback: Callback,
        parameter: i32,
    ) -> TimerId {
        let id = TimerId(NEXT_TIMER_ID.fetch_add(1, Ordering::Relaxed));
        self.time_events.insert(
            id,
            TimeEvent {
                id,
                time,
                parameter,
                callback,
            },
        );
        id
    }

 /// Удаляет calendar event и возвращает исходный found/not-found результат.
    pub(crate) fn kill_time_event(&mut self, id: TimerId) -> bool {
        self.time_events.remove(&id).is_some()
    }
}

impl<Callback: Copy> CTimer<Callback> {
 /// Выполняет оба registry в точном исходном порядке и синхронно вызывает dispatcher.
    pub(crate) fn run<GetTick, GetLocalTime, Dispatch>(
        &mut self,
        ai_tick: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        mut dispatch: Dispatch,
    ) -> TimerRunReport
    where
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> TagTime,
        Dispatch: FnMut(&mut Self, TimerCallbackInvocation<Callback>),
    {
        let mut report = TimerRunReport::default();
        let mut periodic_id = self.periodic_timers.keys().next().copied();
        while let Some(id) = periodic_id {
            let invocation = {
                let timer = self
                    .periodic_timers
                    .get_mut(&id)
                    .expect("ID получен из текущего periodic registry");
                if !timer.open {
                    None
                } else if timer.use_ai_tick {
                    let elapsed = ai_tick.wrapping_sub(timer.start_ai_tick);
                    if timer.elapsed_ai_tick <= elapsed {
                        timer.start_ai_tick = ai_tick;
                        Some(TimerCallbackInvocation {
                            source: TimerCallbackSource::Periodic(id),
                            callback: timer.callback,
                            parameter: timer.parameter,
                        })
                    } else {
                        None
                    }
                } else {
                    let now = get_tick();
                    let elapsed = now.wrapping_sub(timer.start_time_ms);
                    if timer.elapsed_time_ms <= elapsed {
                        timer.start_time_ms = now;
                        Some(TimerCallbackInvocation {
                            source: TimerCallbackSource::Periodic(id),
                            callback: timer.callback,
                            parameter: timer.parameter,
                        })
                    } else {
                        None
                    }
                }
            };
            if let Some(invocation) = invocation {
                report.periodic_callbacks = report.periodic_callbacks.wrapping_add(1);
                dispatch(self, invocation);
            }
            periodic_id = self
                .periodic_timers
                .range((Excluded(id), Unbounded))
                .next()
                .map(|(&next_id, _)| next_id);
        }

        let mut event_id = self.time_events.keys().next().copied();
        while let Some(id) = event_id {
            let now = get_local_time();
            let event = *self
                .time_events
                .get(&id)
                .expect("ID получен из текущего calendar registry");
            debug_assert_eq!(event.id, id);
            if now.legacy_ge(event.time) {
                report.calendar_callbacks = report.calendar_callbacks.wrapping_add(1);
                dispatch(
                    self,
                    TimerCallbackInvocation {
                        source: TimerCallbackSource::Calendar(id),
                        callback: event.callback,
                        parameter: event.parameter,
                    },
                );
                self.time_events.remove(&id);
            }
            event_id = self
                .time_events
                .range((Excluded(id), Unbounded))
                .next()
                .map(|(&next_id, _)| next_id);
        }
        report
    }

 /// Сохраняет ordered traversal `Run`, но разрешает одному domain adapter-у
 /// дождаться библиотечной async DB-операции внутри исходной callback-позиции.
    pub(crate) async fn run_with_async_handler<GetTick, GetLocalTime, Handler, Dispatch>(
        &mut self,
        ai_tick: u32,
        mut get_tick: GetTick,
        mut get_local_time: GetLocalTime,
        handler: &mut Handler,
        mut dispatch: Dispatch,
    ) -> Result<TimerRunReport, AsyncTimerRunBlock<Handler::Block>>
    where
        Handler: AsyncTimerCallbackHandler<Callback, GetTick, GetLocalTime>,
        GetTick: FnMut() -> u32,
        GetLocalTime: FnMut() -> TagTime,
        Dispatch: FnMut(&mut Self, TimerCallbackInvocation<Callback>),
    {
        let mut report = TimerRunReport::default();
        let mut periodic_id = self.periodic_timers.keys().next().copied();
        while let Some(id) = periodic_id {
            let invocation = {
                let timer = self
                    .periodic_timers
                    .get_mut(&id)
                    .expect("ID получен из текущего periodic registry");
                if !timer.open {
                    None
                } else if timer.use_ai_tick {
                    let elapsed = ai_tick.wrapping_sub(timer.start_ai_tick);
                    if timer.elapsed_ai_tick <= elapsed {
                        timer.start_ai_tick = ai_tick;
                        Some(TimerCallbackInvocation {
                            source: TimerCallbackSource::Periodic(id),
                            callback: timer.callback,
                            parameter: timer.parameter,
                        })
                    } else {
                        None
                    }
                } else {
                    let now = get_tick();
                    let elapsed = now.wrapping_sub(timer.start_time_ms);
                    if timer.elapsed_time_ms <= elapsed {
                        timer.start_time_ms = now;
                        Some(TimerCallbackInvocation {
                            source: TimerCallbackSource::Periodic(id),
                            callback: timer.callback,
                            parameter: timer.parameter,
                        })
                    } else {
                        None
                    }
                }
            };
            if let Some(invocation) = invocation {
                report.periodic_callbacks = report.periodic_callbacks.wrapping_add(1);
                let disposition = handler
                    .dispatch(self, invocation, &mut get_tick, &mut get_local_time)
                    .await
                    .map_err(|source| AsyncTimerRunBlock {
                        timer: report,
                        source,
                    })?;
                match disposition {
                    AsyncTimerCallbackDisposition::PassThrough => dispatch(self, invocation),
                    AsyncTimerCallbackDisposition::Handled {
                        next_calendar_event,
                    } => {
                        if let Some(event) = next_calendar_event {
                            let event_id = self.set_time_event(
                                event.time,
                                event.callback,
                                event.parameter,
                            );
                            handler.calendar_event_registered(invocation, event_id);
                        }
                    }
                }
            }
            periodic_id = self
                .periodic_timers
                .range((Excluded(id), Unbounded))
                .next()
                .map(|(&next_id, _)| next_id);
        }

        let mut event_id = self.time_events.keys().next().copied();
        while let Some(id) = event_id {
            let now = get_local_time();
            let event = *self
                .time_events
                .get(&id)
                .expect("ID получен из текущего calendar registry");
            debug_assert_eq!(event.id, id);
            if now.legacy_ge(event.time) {
                report.calendar_callbacks = report.calendar_callbacks.wrapping_add(1);
                let invocation = TimerCallbackInvocation {
                    source: TimerCallbackSource::Calendar(id),
                    callback: event.callback,
                    parameter: event.parameter,
                };
                let disposition = handler
                    .dispatch(self, invocation, &mut get_tick, &mut get_local_time)
                    .await
                    .map_err(|source| AsyncTimerRunBlock {
                        timer: report,
                        source,
                    })?;
                match disposition {
                    AsyncTimerCallbackDisposition::PassThrough => dispatch(self, invocation),
                    AsyncTimerCallbackDisposition::Handled {
                        next_calendar_event,
                    } => {
                        if let Some(event) = next_calendar_event {
                            let next_id = self.set_time_event(
                                event.time,
                                event.callback,
                                event.parameter,
                            );
                            handler.calendar_event_registered(invocation, next_id);
                        }
                    }
                }
                self.time_events.remove(&id);
            }
            event_id = self
                .time_events
                .range((Excluded(id), Unbounded))
                .next()
                .map(|(&next_id, _)| next_id);
        }
        Ok(report)
    }
}
