//! Общие наблюдаемые факты исходного `CMySocket` (`nets/mysocket.cpp/.h`,
//! пары EXE/PDB шести служб в `server/rust/src/manifest/`). Пустой аналог
//! WinSock-класса не создаётся: платформенный I/O передан `crate::transport`,
//! `GetSocketID` и совместимые значения endpoint реализованы здесь.
//!
//! Конструкторские defaults: protocol/type `1`, IPv4 `127.0.0.1`, port `5000`.
//! `WSACreate` создавал overlapped `AF_INET` socket и сразу вызывал `Bind`
//! (все найденные `CServer::Host` передают type `1`); `Bind` трактовал
//! nullable IP как `0.0.0.0`, сужал port до `u16` и разбирал строку legacy
//! `inet_addr` с отказом `INADDR_NONE`. Return-контракты `Recv`/`RecvFrom`/
//! `Sendto` (отдельный `WSAEWOULDBLOCK`, `0` при любой ошибке `RecvFrom`,
//! повтор `Sendto` при `WSAEWOULDBLOCK`) обязан сохранить непосредственный
//! UDP/TCP owner, если соответствующий путь окажется живым.
//!
//! `GetSocketID` увеличивал process-global signed counter и возвращал новое
//! значение (`VERIFIED_DISASSEMBLY` по Misc). В едином процессе каждый бывший
//! сервис держит свой `SocketIdAllocator`: один общий static изменил бы ID от
//! чужой активности. `AtomicU32` сохраняет 32-битное машинное wrapping.
//!
//! Платформенный I/O принадлежит владельцу процесса; Shared несёт только
//! значения endpoint, legacy-грамматику адресов и шаблон выдачи socket ID.
//! Доказательства: docs/reconstruction/shared-technical.md#общие-факты-cmysocket

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::atomic::{AtomicU32, Ordering};

/// Исходный protocol/type по умолчанию (`SOCK_STREAM`).
pub const DEFAULT_SOCKET_TYPE: i32 = 1;
/// Исходный локальный порт до явной настройки.
pub const DEFAULT_PORT: u32 = 5000;
/// Исходный IPv4 до явной настройки.
pub const DEFAULT_IP: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Представляет network-order IPv4 bytes как исходный x86 `unsigned long`.
pub fn legacy_ipv4_word(address: Ipv4Addr) -> u32 {
    u32::from_le_bytes(address.octets())
}

/// Преобразует доказанный nullable IPv4 и 32-битный port в адрес старого bind.
///
/// `None` соответствует `INADDR_ANY`; port сужается как исходный cast к
/// `u_short`, а не валидируется как новая конфигурационная политика.
pub fn legacy_bind_endpoint(address: Option<Ipv4Addr>, port: u32) -> SocketAddrV4 {
    SocketAddrV4::new(address.unwrap_or(Ipv4Addr::UNSPECIFIED), port as u16)
}

/// Разбирает числовой IPv4 в формате WinSock `inet_addr`.
///
/// Поддерживаются исторические формы `a`, `a.b`, `a.b.c`, `a.b.c.d` и
/// decimal/octal/hex-компоненты. Как исходный API, функция не различает
/// broadcast `255.255.255.255` и ошибку: оба результата представлены `None`.
pub fn legacy_inet_addr(value: &[u8]) -> Option<Ipv4Addr> {
    let value = value.split(|byte| *byte == 0).next().unwrap_or_default();
    if value.is_empty() {
        return None;
    }

    let parts = value.split(|byte| *byte == b'.').collect::<Vec<_>>();
    if parts.len() > 4 || parts.iter().any(|part| part.is_empty()) {
        return None;
    }
    let numbers = parts
        .iter()
        .map(|part| parse_inet_number(part))
        .collect::<Option<Vec<_>>>()?;

    let address = match numbers.as_slice() {
        [a] => *a,
        [a, b] if *a <= 0xff && *b <= 0x00ff_ffff => (*a << 24) | *b,
        [a, b, c] if *a <= 0xff && *b <= 0xff && *c <= 0xffff => (*a << 24) | (*b << 16) | *c,
        [a, b, c, d] if [a, b, c, d].iter().all(|part| **part <= 0xff) => {
            (*a << 24) | (*b << 16) | (*c << 8) | *d
        }
        _ => return None,
    };
    (address != u32::MAX).then(|| Ipv4Addr::from(address.to_be_bytes()))
}

fn parse_inet_number(part: &[u8]) -> Option<u32> {
    let (digits, radix) = if let Some(hex) = part
        .strip_prefix(b"0x")
        .or_else(|| part.strip_prefix(b"0X"))
    {
        (hex, 16)
    } else if part.len() > 1 && part[0] == b'0' {
        (&part[1..], 8)
    } else {
        (part, 10)
    };
    if digits.is_empty() {
        return None;
    }
    digits.iter().try_fold(0_u32, |value, byte| {
        let digit = match byte {
            b'0'..=b'9' => u32::from(byte - b'0'),
            b'a'..=b'f' => u32::from(byte - b'a') + 10,
            b'A'..=b'F' => u32::from(byte - b'A') + 10,
            _ => return None,
        };
        (digit < radix).then_some(())?;
        value.checked_mul(radix)?.checked_add(digit)
    })
}

/// Независимый счётчик socket ID одного исторического сервиса.
pub struct SocketIdAllocator {
    last_issued: AtomicU32,
}

impl SocketIdAllocator {
    /// Создаёт счётчик с исходным нулевым process-global значением.
    pub const fn new() -> Self {
        Self {
            last_issued: AtomicU32::new(0),
        }
    }

    /// Выдаёт следующий signed 32-битный ID с машинным wrapping оригинала.
    pub fn next(&self) -> i32 {
        self.last_issued
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1) as i32
    }
}

impl Default for SocketIdAllocator {
    fn default() -> Self {
        Self::new()
    }
}
