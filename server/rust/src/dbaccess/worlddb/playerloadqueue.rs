//! FIFO запросов загрузки игроков старого WorldServer перенесена в Realm characters.
//! Здесь её реэкспорт для старого пакета.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::characters::playerloadqueue::*;
