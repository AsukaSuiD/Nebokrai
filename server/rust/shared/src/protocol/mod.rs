//! Общая Rust-инфраструктура для совместимых бинарных форматов.
//!
//! Собственные примитивы поверх bytes. Форматы и привязки к оригинальным
//! .cpp/.h и машинному коду документируются у использующих их декодеров.

mod crc32; // CRC-32 владельца public/crc32static.
mod errors; // ошибки границ чтения и записи кодека.
mod md5; // MD5 владельца public/md5.
mod reader; // LegacyReader: последовательное чтение бинарных полей.
mod writer; // LegacyWriter: запись бинарных полей.

pub use crc32::{data_crc32, file_crc32}; // CRC-32 блока данных и файла.
pub use errors::{LegacyReadBlock, LegacyWriteBlock}; // ошибки границ чтения и записи.
pub use md5::message_digest; // MD5-дайджест сообщения.
pub use reader::LegacyReader; // последовательный читатель бинарных полей.
pub use writer::LegacyWriter; // писатель бинарных полей.
