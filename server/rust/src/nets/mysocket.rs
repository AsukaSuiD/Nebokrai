//! Общие наблюдаемые факты CMySocket перенесены в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_shared::network::{DEFAULT_SOCKET_TYPE, legacy_ipv4_word};
