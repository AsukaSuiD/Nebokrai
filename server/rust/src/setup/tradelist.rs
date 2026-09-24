//! Общие торговые списки `CTradeList` перенесены в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    CTradeList, TradeListDecodeError, TradeListFormatError, TradeListSerializeError,
};
