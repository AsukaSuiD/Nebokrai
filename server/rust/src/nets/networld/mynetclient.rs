//! Исходящее LoginServer-направление World перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::app::world_client::CMyNetClient;
