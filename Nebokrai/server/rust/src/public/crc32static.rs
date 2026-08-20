//! CRC-32 владельца `public/crc32static.cpp`.
//!
//! Статус владельца: `IMPLEMENTED`.
//!
//! Точные варианты `CCrc32Static::DataCrc32`:
//! - AuthServer: `authserver.exe + authserver.pdb`, RVA `0x00015F80`;
//! - BillingServer: `billingserver.exe + billingserver.pdb`, RVA `0x00013530`;
//! - LoginServer: `loginserver.exe + LoginServer.pdb`, RVA `0x0007F050`;
//! - MiscServer: `miscserver.exe + miscserver.pdb`, RVA `0x00005730`;
//! - GameServer: `gameserver.exe + GameServer.pdb`, RVA `0x0007B0A0`;
//! - WorldServer: `Nworldserver.exe + WorldServer.pdb`, RVA `0x000A43A0`.
//!
//! Для всех вариантов исходные EXE/PDB и их SHA-256 совпадают с зафиксированными
//! в `src/manifest`; пути PDB: `h:\fengyun\fy_russia\src\public`,
//! `d:\complite_version\fengyun_russia\trunk\public` и
//! `e:\svn\fengyun_russia_dev\public`.
//!
//! Все шесть тел совпадают: регистр начинается с `0xFFFF_FFFF`, каждый байт
//! обрабатывается reflected-таблицей с полиномом IEEE CRC-32, затем результат
//! инвертируется. Это ровно алгоритм, гарантируемый `crc32fast`; библиотека
//! заменяет исходную статическую таблицу и ручной цикл без изменения checksum.
//!
//! Вариант ServerUpdate дополнительно содержит `GetFileSizeQW` RVA `0x00003D10`
//! и `FileCrc32Filemap` RVA `0x00003D70` из точной пары
//! `GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb`. Успешный путь
//! вычисляет тот же CRC-32 по всем байтам файла; Windows file mapping по блокам
//! до `0xA00000` является техническим механизмом и заменён потоковым чтением.
//! Вместо пары `DWORD error + out-param` Rust возвращает `io::Result<u32>` и не
//! скрывает ошибку открытия или чтения.
//!
//! Остальные 12 тысяч строк старого корпуса классифицированы как чужие
//! template-инстанцирования, MFC/COM/CRT, allocator и compiler cleanup из
//! линковки ServerUpdate, GameServer и WorldServer. Они не являются семантикой
//! CRC-владельца и удалены; их настоящие владельцы остаются в собственных
//! экспортированных `.rs`. Полный сырой экспорт доступен в истории Git.

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

use crc32fast::Hasher;

/// Вычисляет исходный reflected IEEE CRC-32 для переданного набора байт.
pub(crate) fn data_crc32(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/// Вычисляет исходный CRC-32 всего файла потоково, не загружая его целиком.
///
/// Ошибка открытия или чтения возвращается вызывающему без частичного checksum.
pub(crate) fn file_crc32(path: impl AsRef<Path>) -> io::Result<u32> {
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
