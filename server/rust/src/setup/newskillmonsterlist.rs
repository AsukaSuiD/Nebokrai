//! Списки монстров новых навыков `CNewSkillMonserConf` перенесены в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.

pub(crate) use nebokrai_shared::resources::{
    NewSkillMonsterConf, NewSkillMonsterDecodeError,
};
