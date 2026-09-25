//! Общие операторы `CGMList` перенесены в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    CGMList, GmInfo, GmListCollection, GmListDecodeError,
    GmListSerializationBlock,
};
