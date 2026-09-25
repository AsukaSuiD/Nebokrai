//! Общие представления значений без реестров и игрового состояния.

mod date; // TagTime: владелец исторического tagTime.
mod guid; // CGuid: совместимый GUID.

pub use date::{TagTime, TagTimeArithmeticBlock, TagTimeParseBlock}; // исторический tagTime и ошибки его разбора/арифметики.
pub use guid::{CGuid, GuidParseError, NULL_GUID}; // совместимый GUID и его null-значение.
