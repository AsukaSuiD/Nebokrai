//! DB-адаптер общих переменных WorldServer перенесён в Realm persistence.
//! Здесь его реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::persistence::rsgenvar::*;
