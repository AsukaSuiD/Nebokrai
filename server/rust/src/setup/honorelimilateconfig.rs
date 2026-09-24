//! Порог honor за убийство `HonorElimilateConfig` перенесён в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{HonorElimilateConfig, HonorEliminateDecodeError};
