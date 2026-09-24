//! Фильтр запрещённых слов перенесён в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    CWordsFilter, WordsFilterDecodeError, WordsFilterSerializeError,
};
