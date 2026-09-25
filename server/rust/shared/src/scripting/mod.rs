//! Общие форматы языка; игровые владельцы и диспетчеры команд принадлежат Zone.

mod functionlist; // реестр команд CScript::LoadFunction.
mod ini; // CIni: чтение списков (memory-ветвь).
mod integer_expression; // Check/Count/ComputeVar целочисленных выражений.
mod variablelist; // объявления переменных (LoadVarList).
pub mod parser; // структурный байтовый разбор команд для Zone-исполнителя CScript.

pub use functionlist::{
    FunctionDefinition, FunctionListError, FunctionListErrorKind, FunctionListRecords,
};
pub use variablelist::{
    VariableDefault, VariableDefinition, VariableListError, VariableListErrorKind,
    VariableListRecords,
};
