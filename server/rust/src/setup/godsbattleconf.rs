//! Общий Gods Battle configuration owner перенесён в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    CGodsBattleConf, GodsBattleDecodeError, GodsBattleFactionNpcName,
    GodsBattleFactionXydUpdate, GodsBattleLoadError, GodsBattleNpcFactionUpdate,
    GodsBattleSzlCalculation,
};
