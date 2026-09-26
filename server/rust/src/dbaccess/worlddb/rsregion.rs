//! DB-владелец регионов старого WorldServer перенесён в Realm regions.
//! Здесь его реэкспорт для старого пакета.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::regions::rsregion::*;
