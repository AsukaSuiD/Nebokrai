//! Общее country contribution `CContributeSetup` перенесено в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    CContributeSetup, ContributeSetupDecodeError, ContributeSetupSerializeError,
};
