//! DB-владелец игрока `CRsPlayer` старого WorldServer перенесён в Realm
//! persistence. Здесь его реэкспорт для переходных потребителей.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::persistence::rsplayer::*;
