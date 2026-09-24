//! Общие средства запуска и механизмы ожиданий без реестров доменов.

mod timer;

pub use timer::{
    AsyncTimerCallbackDisposition, AsyncTimerCallbackHandler, AsyncTimerRunBlock, CTimer,
    CalendarTimerRegistration, PeriodicTimer, TimerCallbackInvocation, TimerCallbackSource,
    TimerId, TimerRunReport,
};
