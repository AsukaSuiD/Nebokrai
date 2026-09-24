//! Общая Rust-инфраструктура для совместимых бинарных форматов.
//!
//! Собственные примитивы поверх bytes. Форматы и привязки к оригинальным
//! .cpp/.h и машинному коду документируются у использующих их декодеров.

mod errors;
mod md5;
mod reader;
mod writer;

pub use errors::{LegacyReadBlock, LegacyWriteBlock};
pub use md5::message_digest;
pub use reader::LegacyReader;
pub use writer::LegacyWriter;
