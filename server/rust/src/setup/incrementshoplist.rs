//! Общий increment shop `CIncrementShopList` перенесён в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    CIncrementShopList, IncrementShopDecodeError, IncrementShopGoodsQuery,
    IncrementShopGoodsResult, IncrementShopItem, IncrementShopSerializeError,
};
