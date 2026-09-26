//! Общие структуры организаций старого WorldServer перенесены в Realm content.
//! Здесь их реэкспорт для старого пакета.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::content::organizing::*;
