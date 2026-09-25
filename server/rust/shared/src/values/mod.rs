//! Общие представления значений без реестров и игрового состояния.

mod date;
mod guid;

pub use date::{TagTime, TagTimeArithmeticBlock, TagTimeParseBlock};
pub use guid::{CGuid, GuidParseError, NULL_GUID};
