//! Сценарные значения и кодек Zone.

mod variables; // механика и wire-формат списков сценарных переменных.

pub use variables::{
    CVariableList, GameVariable, GameVariableMutationOutcome, GameVariableSnapshotError,
    GameVariableValue,
}; // типы сценарных переменных и их мутаций.
