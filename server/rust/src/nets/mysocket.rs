//! Общие наблюдаемые факты CMySocket перенесены в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::{
    legacy_inet_addr, legacy_ipv4_word, SocketIdAllocator, DEFAULT_IP, DEFAULT_PORT,
    DEFAULT_SOCKET_TYPE,
};
