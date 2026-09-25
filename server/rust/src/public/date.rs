//! Владелец исторического `tagTime` перенесён в Shared values.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::values::{TagTime, TagTimeArithmeticBlock};
