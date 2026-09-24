//! Менеджер Auth-заявок LoginServer перенесён в Realm access.
//! Здесь его реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::access::authmanager::*;
