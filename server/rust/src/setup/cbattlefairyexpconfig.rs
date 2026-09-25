//! Опыт fairy/battle fairy `CBattleFairyExpConfig` перенесён в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    BattleFairyExpDecodeError, CBattleFairyExpConfig,
};
