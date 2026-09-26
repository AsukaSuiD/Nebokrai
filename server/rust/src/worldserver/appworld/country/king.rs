//! King state старого WorldServer перенесён в Realm organizations.
//! Здесь его реэкспорт для старого пакета.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::organizations::king::*;
