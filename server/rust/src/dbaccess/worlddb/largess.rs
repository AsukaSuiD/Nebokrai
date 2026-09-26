//! DB-владелец и worker подарков старого WorldServer перенесены в Realm
//! persistence. Здесь их реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::persistence::largess::*;
