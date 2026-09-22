//! Общие форматы языка; игровые владельцы и диспетчеры команд принадлежат Zone.

mod functionlist;
mod ini;
mod integer_expression;
mod variablelist;
pub mod parser;

pub use functionlist::{
    FunctionDefinition, FunctionListError, FunctionListErrorKind, FunctionListRecords,
};
pub use variablelist::{
    VariableDefault, VariableDefinition, VariableListError, VariableListErrorKind,
    VariableListRecords,
};
