//! Производный владелец GameServer-соединений Billing перенесён в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::billing_server::CServerForGS;
