//! Исходящее World-направление MiscServer перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::misc_client::{
    CMyNetClient, MiscClientIoError, MiscClientIoStep,
};
