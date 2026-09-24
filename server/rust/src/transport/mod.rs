//! Linux-владелец транспортной механики перенесён в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::{
    bind_tcp_ipv4, listen_tcp_ipv4, read_client_tcp_chunk, read_server_tcp_chunk, shutdown_tcp,
    write_once_tcp,
};
