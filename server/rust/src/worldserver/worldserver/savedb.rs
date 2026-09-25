//! Оркестрация сохранения из `worldserver/savedb.cpp/.h` перенесена в
//! Realm `persistence/savedb`; worker-вход и guards — в `persistence/saveworker`.
//! Здесь реэкспорт для старого пакета.

pub(crate) use nebokrai_realm::persistence::savedb::*;
