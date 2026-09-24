//! Состояние принятого TCP-соединения CServerClient перенесено в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::CServerClient;
