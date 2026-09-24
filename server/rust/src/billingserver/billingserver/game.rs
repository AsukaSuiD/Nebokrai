//! Runtime-owner `CGame` старого BillingServer перенесён в Realm billing.
//! Здесь его реэкспорт для старого пакета.

pub(crate) use nebokrai_realm::billing::game::*;
