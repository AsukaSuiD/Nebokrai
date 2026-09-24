//! Сообщение направления GameServer ↔ BillingServer перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::billing_message::{BillingMessageHandlers, CMessage};
