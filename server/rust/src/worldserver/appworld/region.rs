//! Мировой регион перенесён в Realm regions.
//! Здесь его реэкспорт для переходных потребителей.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::regions::region::*;
