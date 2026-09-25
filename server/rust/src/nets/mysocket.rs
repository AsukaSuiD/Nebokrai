//! Общие наблюдаемые факты CMySocket перенесены в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::{DEFAULT_SOCKET_TYPE, legacy_ipv4_word};
