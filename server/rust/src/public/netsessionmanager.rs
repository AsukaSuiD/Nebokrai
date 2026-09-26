//! Ordered registry CNetSession перенесён в Shared runtime.
//! Здесь реэкспорт для переходных потребителей обеих ролей.
//! Session staging/report типы ушли в Realm вместе с единственным
//! потребителем (`organizingctrl`); старый пакет их не переиздаёт.

pub(crate) use nebokrai_shared::runtime::{
    CNetSessionManager, NetSessionCallbackOutcome, NetSessionManagerVariant,
};
