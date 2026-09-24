//! Состояние принятого GameServer-соединения Billing перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::billing_server_client::BillingReceiveError;
