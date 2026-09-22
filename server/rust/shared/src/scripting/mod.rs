//! Общие форматы языка; игровые владельцы и диспетчеры команд принадлежат Zone.

mod functionlist;
mod ini;
mod variablelist;

pub use functionlist::{
    FunctionDefinition, FunctionListError, FunctionListErrorKind, FunctionListRecords,
};
pub use variablelist::{
    VariableDefault, VariableDefinition, VariableListError, VariableListErrorKind,
    VariableListRecords,
};
