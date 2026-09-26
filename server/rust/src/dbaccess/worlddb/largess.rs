//! DB-владелец и worker подарков старого WorldServer перенесены в Realm
//! persistence. Здесь их реэкспорт для переходных потребителей.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::persistence::largess::*;
