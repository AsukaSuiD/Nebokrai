//! JJC система WorldServer перенесена в Realm activities.
//! Здесь её реэкспорт для переходных потребителей.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::activities::jjcsystem::*;
