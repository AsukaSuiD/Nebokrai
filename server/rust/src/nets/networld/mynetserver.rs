//! Сервер принятых GameServer-соединений World перенесён в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::world_server::{CMyNetServer, WorldServerEvent};
