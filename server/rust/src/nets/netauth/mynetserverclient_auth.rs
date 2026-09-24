//! Состояние принятого LoginServer-соединения Auth перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::auth_server_client::AuthReceiveError;
