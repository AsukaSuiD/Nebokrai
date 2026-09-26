//! Оркестрация сохранения из `worldserver/savedb.cpp/.h` перенесена в
//! Realm `persistence/savedb`; worker-вход и guards — в `persistence/saveworker`.
//! Здесь реэкспорт для старого пакета.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::persistence::savedb::*;
