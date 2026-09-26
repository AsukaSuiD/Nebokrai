//! DB-владелец village-war `CRsVillageWar` WorldServer рядом с расписанием
//! `CVillageWarSys` (`villagewarsys`).
//!
//! Lifecycle не хранит самостоятельного состояния; save делегируется
//! существующему соединению с исходными значениями bool и SQL-порядком.
//! Поэтому Rust-модуль остаётся документационным и не вводит фиктивный
//! API: факт создания owner-а фиксирует startup-дискриминант
//! `WorldGameDatabaseOwner::RsVillageWar` (`crate::app::world_runtime`).
