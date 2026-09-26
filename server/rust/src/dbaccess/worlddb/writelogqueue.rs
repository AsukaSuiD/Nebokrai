//! Исторический путь FIFO журнала WorldServer; реализация перенесена в
//! `nebokrai_realm::persistence::writelogqueue`.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::persistence::writelogqueue::*;
