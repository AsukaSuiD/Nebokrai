//! Сценарные значения, кодек и данные диспетчера функций Zone.

pub mod functions; // идентификаторы плотного сценарного диспетчера, виды его параметров и упаковка локального времени.
mod variables; // механика и wire-формат списков сценарных переменных.

pub use variables::{
    CVariableList, GameVariable, GameVariableMutationOutcome, GameVariableSnapshotError,
    GameVariableValue,
};
