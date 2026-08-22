//! Общий module graph восстановленных серверных процессов Miracle.
//!
//! Конкретные owner-ы остаются внутренними для crate. Наружу экспортируются
//! только минимальные process-функции, используемые тонкими бинарными входами.

#[allow(dead_code, reason = "общий корпус содержит owner-ы соседних процессов")]
mod authserver;
#[allow(dead_code, reason = "общий корпус содержит owner-ы соседних процессов")]
mod billingserver;
#[allow(dead_code, reason = "каждый процесс использует только свои DB-owner-ы")]
mod dbaccess;
#[allow(dead_code, reason = "GameServer не получает process entrypoint в этом проходе")]
mod gameserver;
#[allow(dead_code, reason = "LoginServer подключается отдельным process owner-ом")]
mod loginserver;
#[allow(dead_code, reason = "MiscServer подключается отдельным process owner-ом")]
mod miscserver;
mod nets;
mod public;
#[allow(dead_code, reason = "setup-owner-ы используются соответствующими процессами")]
mod setup;
#[allow(dead_code, reason = "transport-варианты используются соответствующими процессами")]
mod transport;
#[allow(dead_code, reason = "WorldServer подключается отдельным process owner-ом")]
mod worldserver;

mod process;

pub use process::{run_authserver_process, run_billingserver_process};
