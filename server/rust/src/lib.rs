//! Общий module graph восстановленных серверных процессов Miracle.
//!
//! Конкретные owner-ы остаются внутренними для crate. Наружу экспортируются
//! только минимальные process-функции, используемые тонкими бинарными входами.

#[allow(dead_code, reason = "GameServer использует только достигнутые owner-ы своего процесса")]
mod gameserver;
mod nets;
mod public;
#[allow(dead_code, reason = "setup-owner-ы используются соответствующими процессами")]
mod setup;
#[allow(dead_code, reason = "transport-варианты используются соответствующими процессами")]
mod transport;

mod process;

pub use process::{
    run_authserver_process, run_billingserver_process, run_gameserver_process,
    run_loginserver_process, run_miscserver_process, run_worldserver_process,
};
