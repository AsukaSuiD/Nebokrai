//! Общие средства запуска и механизмы ожиданий без реестров доменов.

mod timer;
mod tools;

pub use timer::{
    AsyncTimerCallbackDisposition, AsyncTimerCallbackHandler, AsyncTimerRunBlock, CTimer,
    CalendarTimerRegistration, PeriodicTimer, TimerCallbackInvocation, TimerCallbackSource,
    TimerId, TimerRunReport,
};
pub use tools::{
    add_game_error_log_text, add_game_log_text, get_line_direction, ini_decode, put_debug_string,
    put_string_to_file,
};
