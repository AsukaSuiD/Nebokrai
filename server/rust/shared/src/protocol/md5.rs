//! `IMPLEMENTED` — владелец MD5 из общего `public/md5.cpp` и `public/md5.h`.
//!
//! Источник: `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`
//! (идентификаторы: `server/rust/src/manifest/_loginserver_export_manifest.toml`).
//! Исходные пути PDB:
//! `d:\complite_version\fengyun_russia\trunk\public\md5.cpp/.h`.
//! Существенный `MessageDigest` имеет RVA `0x4263B0`: его контракт —
//! инициализировать контекст константами MD5 IV `0x67452301`,
//! `0xEFCDAB89`, `0x98BADCFE`, `0x10325476`, последовательно добавить один и
//! тот же буфер `rounds` раз и вернуть стандартный 16-байтовый MD5 digest. При
//! неположительном `rounds` оригинал завершал пустой контекст, то есть
//! хешировал пустую строку.
//!
//! Ручные `Transform`, `Update`, `Final`, endian-копирование и очистка контекста
//! были внутренностями заменяемого алгоритма и удалены. `md-5` из RustCrypto
//! предоставляет тот же стандартный digest; владение контекстом и его очистку
//! обеспечивает Rust. Локальных неизвестностей у этого контракта нет.

use md5::{Digest, Md5};

/// Возвращает MD5 от `input`, повторённого `rounds` раз.
pub fn message_digest(input: &[u8], rounds: i32) -> [u8; 16] {
    let mut digest = Md5::new();
    for _ in 0..rounds.max(0) {
        digest.update(input);
    }
    digest.finalize().into()
}
