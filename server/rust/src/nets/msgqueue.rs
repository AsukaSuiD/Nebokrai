//! Потокобезопасная очередь сообщений перенесена в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::CMsgQueue;
