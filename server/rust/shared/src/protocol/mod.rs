//! Общая Rust-инфраструктура для совместимых бинарных форматов.
//!
//! Собственные примитивы поверх bytes. Форматы и привязки к оригинальным
//! .cpp/.h и машинному коду документируются у использующих их декодеров.

mod crc32;
mod errors;
mod md5;
mod reader;
mod writer;

pub use crc32::{data_crc32, file_crc32};
pub use errors::{LegacyReadBlock, LegacyWriteBlock};
pub use md5::message_digest;
pub use reader::LegacyReader;
pub use writer::LegacyWriter;
