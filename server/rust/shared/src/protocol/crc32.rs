//! CRC-32 владельца `public/crc32static.cpp` (пары EXE/PDB шести служб и
//! ServerUpdate в `server/rust/src/manifest/`).
//!
//! Все шесть тел `DataCrc32` совпадают: регистр начинается с `0xFFFF_FFFF`,
//! каждый байт обрабатывается reflected-таблицей с полиномом IEEE CRC-32,
//! результат инвертируется. Это ровно алгоритм `crc32fast`; библиотека
//! заменяет статическую таблицу и ручной цикл без изменения checksum.
//! ServerUpdate-вариант файлового CRC считает тот же CRC-32 по всем байтам;
//! Windows file mapping по блокам до `0xA00000` заменён потоковым чтением, а
//! пара `DWORD error + out-param` — `io::Result<u32>`.
//!
//! Экземпляров нет: helpers без состояния, доступные всем направлениям.
//! Доказательства: docs/reconstruction/shared-technical.md#crc32-ccrc32static

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

use crc32fast::Hasher;

/// Вычисляет исходный reflected IEEE CRC-32 для переданного набора байт.
pub fn data_crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/// Вычисляет исходный CRC-32 всего файла потоково, не загружая его целиком.
///
/// Ошибка открытия или чтения возвращается вызывающему без частичного checksum.
pub fn file_crc32(path: impl AsRef<Path>) -> io::Result<u32> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Hasher::new();
    let mut buffer = [0; 64 * 1024];

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok(hasher.finalize());
        }
        hasher.update(&buffer[..read]);
    }
}
