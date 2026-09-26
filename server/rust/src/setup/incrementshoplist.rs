//! Общий increment shop `CIncrementShopList` перенесён в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_shared::resources::{
    CIncrementShopList, IncrementShopDecodeError, IncrementShopGoodsQuery,
    IncrementShopGoodsResult, IncrementShopItem,
};
