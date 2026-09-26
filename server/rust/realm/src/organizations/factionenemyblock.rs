//! Отчёт заблокированной мутации enemy-связи фракций.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionEnemyMutationBlock {
    pub state_changed: bool,
    pub changed_flag_set: bool,
    pub formatted_len: usize,
}
