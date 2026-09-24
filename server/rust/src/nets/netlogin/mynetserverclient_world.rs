//! Состояние принятого WorldServer-соединения Login перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::login_world_server_client::WorldReceiveError;
