//! Владелец TimeToReturn перенесён в Realm content.
//! Здесь реэкспорт для переходных потребителей.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::content::{
    TimeToReturn, TimeToReturnCallbacks, TimeToReturnContext, TimeToReturnFireReport,
    TimeToReturnLoadError, TimeToReturnLoadReport,
};
