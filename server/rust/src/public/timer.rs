//! Calendar и interval-owner `CTimer` перенесён в Shared runtime.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::runtime::{
    AsyncTimerCallbackDisposition, AsyncTimerCallbackHandler, AsyncTimerRunBlock, CTimer,
    CalendarTimerRegistration, TimerCallbackInvocation, TimerCallbackSource, TimerId,
};
