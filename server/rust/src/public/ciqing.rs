//! Конфигурация CiQing перенесена в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    CCiQingSetup, CiQingDecodeError, CiQingSerializationBlock,
};
