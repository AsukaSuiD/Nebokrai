//! Жизненный цикл ресурсов World перенесён в Realm content.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::content::{
    DefaultClientResourceOwner, DefaultClientResourceReplacement, LOAD_SERVER_RESOURCE_SUCCESS_LOG,
};
