//! Тюремная конфигурация `PrisonConf` перенесена в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    PrisonConf, PrisonConfDecodeError, PrisonConfSerializeError,
};
