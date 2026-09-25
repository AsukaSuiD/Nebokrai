//! Общие средства запуска и механизмы ожиданий без реестров доменов.

mod netsession; // CNetSession: асинхронная сетевая сессия, общая для Game/World.
mod netsessionmanager; // CNetSessionManager: упорядоченный реестр net-сессий.
mod timer; // CTimer: calendar и interval-owner таймеры.
mod tools; // общие технические функции public/tools.cpp.

pub use netsession::{
    CNetSession, NetSessionAsyncResult, NetSessionAsyncResultKind, NetSessionBeginBlock,
    NetSessionBeginDispatch, NetSessionCallbackAlreadyAssigned, NetSessionCookie,
    NetSessionEndpoint,
};
pub use netsessionmanager::{
    CNetSessionManager, CreatedNetSession, NetSessionCallbackOutcome, NetSessionCreateBlock,
    NetSessionManagerBeginBlock, NetSessionManagerVariant, NetSessionRunReport,
    NetSessionSetCallbackBlock,
};
pub use timer::{
    AsyncTimerCallbackDisposition, AsyncTimerCallbackHandler, AsyncTimerRunBlock, CTimer,
    CalendarTimerRegistration, PeriodicTimer, TimerCallbackInvocation, TimerCallbackSource,
    TimerId, TimerRunReport,
};
pub use tools::{
    add_game_error_log_text, add_game_log_text, get_line_direction, ini_decode, put_debug_string,
    put_string_to_file,
};
