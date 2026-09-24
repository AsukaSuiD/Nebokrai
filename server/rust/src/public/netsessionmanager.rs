//! Ordered registry CNetSession перенесён в Shared runtime.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::runtime::{
    CNetSessionManager, CreatedNetSession, NetSessionCallbackOutcome, NetSessionCreateBlock,
    NetSessionManagerBeginBlock, NetSessionManagerVariant, NetSessionRunReport,
    NetSessionSetCallbackBlock,
};
