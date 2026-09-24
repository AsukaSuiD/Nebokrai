//! Отчёт отказа мутации enemy-связи фракций, перенесённый в Realm
//! `organizations/` формой извлечения RegionParamState/AuthDbContext.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionEnemyMutationBlock {
    pub state_changed: bool,
    pub changed_flag_set: bool,
    pub formatted_len: usize,
}
