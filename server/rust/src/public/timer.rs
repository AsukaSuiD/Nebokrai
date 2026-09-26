//! Calendar и interval-owner `CTimer` перенесён в Shared runtime.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_shared::runtime::{
    AsyncTimerCallbackDisposition, AsyncTimerCallbackHandler, AsyncTimerRunBlock, CTimer,
    CalendarTimerRegistration, TimerCallbackInvocation, TimerCallbackSource, TimerId,
};
