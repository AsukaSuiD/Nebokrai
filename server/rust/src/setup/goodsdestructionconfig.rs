//! Уничтожение предметов `CGoodsDestroySetup` перенесено в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    GoodsDestroyDecodeError,
    GoodsDestroySetup,
};
