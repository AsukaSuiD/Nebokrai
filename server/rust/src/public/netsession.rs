//! Асинхронная CNetSession перенесена в Shared runtime.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::runtime::{
    NetSessionAsyncResult, NetSessionAsyncResultKind, NetSessionEndpoint,
};
