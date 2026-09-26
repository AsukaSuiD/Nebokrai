//! Обработчик M2W auction WorldServer перенесён в Realm app.
//! Здесь его реэкспорт для переходных потребителей.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::onmsg_m2w_auction::*;
