//! Потокобезопасная очередь сокетных команд перенесена в Shared network.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_shared::network::CSocketCommands;
