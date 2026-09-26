//! DB-владелец товаров старого WorldServer перенесён в Realm content.
//! Здесь его реэкспорт для переходных потребителей.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::content::dbgoods::*;
