//! DB-владелец village-war `CRsVillageWar` WorldServer, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`, перенесённый в Realm `activities/`
//! рядом с расписанием `CVillageWarSys` (`villagewarsys`).
//!
//! Lifecycle не хранит самостоятельного состояния; save делегируется
//! существующему соединению с исходными значениями bool и SQL-порядком.
//! Поэтому Rust-модуль остаётся документационным и не вводит фиктивный
//! API: факт создания owner-а фиксируют startup-дискриминант
//! `WorldGameDatabaseOwner::RsVillageWar` (`app/world_runtime`) и bool-маркер
//! переходного runtime старого пакета.
