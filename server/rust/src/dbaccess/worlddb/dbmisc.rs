//! DB-владелец мирового Misc с аукционными потоками перенесён в Realm persistence.
//! Здесь его реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::persistence::dbmisc::*;
