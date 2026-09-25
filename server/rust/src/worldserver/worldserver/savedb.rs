//! Оркестрация сохранения из `worldserver/savedb.cpp/.h`, worker-вход
//! `SaveThreadFunc` и его frozen-вход `WorldSaveThreadJob` перенесены в
//! Realm `persistence/savedb`. Здесь их реэкспорт для старого пакета.

pub(crate) use nebokrai_realm::persistence::savedb::*;
