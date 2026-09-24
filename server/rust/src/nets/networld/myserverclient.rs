//! Состояние принятого GameServer-соединения World перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::world_server_client::GameServerReceiveError;
