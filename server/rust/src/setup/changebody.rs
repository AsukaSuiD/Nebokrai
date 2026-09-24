//! Ограничения смены тела `CChangeBodyConf` перенесены в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    CChangeBodyConf, ChangeBodyDecodeError, ChangeBodySerializeError,
};
