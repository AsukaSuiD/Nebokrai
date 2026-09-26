//! Общие наблюдаемые факты исходного `CMySocket`, восстановленные из
//! `nets/mysocket.cpp` и `.h` без создания пустого аналога WinSock-класса.
//!
//! Статус владельца: `IMPLEMENTED`. Платформенный I/O передан техническому
//! владельцу `crate::transport`; `GetSocketID` и совместимые значения endpoint
//! реализованы здесь.
//!
//! Точные пары и основные RVA. Идентификаторы SHA-256 всех перечисленных
//! EXE/PDB зафиксированы в `server/rust/src/manifest/`:
//! - Auth: `authserver.exe + authserver.pdb`:
//!   `SetIP` `0x00001EA0`, init `0x00013030`, cleanup `0x000130C0`, ctor
//!   `0x000130D0`, dtor `0x00013130`, `Create` `0x00013160`, `Bind`
//!   `0x00013170`, `Close` `0x00013270`, `OnClose` `0x00013290`, `Recv`
//!   `0x000132A0`, `Send` `0x00013320`, `RecvFrom` `0x00013330`, `Sendto`
//!   `0x00013420`, `GetSocketID` `0x000134E0`, `WSACreate` `0x00013530`;
//! - Billing: `billingserver.exe + billingserver.pdb`:
//!   соответственно `0x00001860`, `0x0000D600`, `0x0000D690`, `0x0000D6A0`,
//!   `0x0000D6F0`, `0x0000D720`, `0x0000D730`, `0x0000D830`, `0x0000D850`,
//!   `0x0000D860`, `0x0000D8E0`, `0x0000D8F0`, `0x0000D9E0`, `0x0000DAA0`,
//!   `0x0000DAF0`;
//! - Login: `loginserver.exe + LoginServer.pdb`:
//!   соответственно `0x00002AF0`, `0x0006C4F0`, `0x0006C580`, `0x0006C590`,
//!   `0x0006C5E0`, `0x0006C610`, `0x0006C620`, `0x0006C720`, `0x0006C740`,
//!   `0x0006C750`, `0x0006C7D0`, `0x0006C7E0`, `0x0006C8D0`, `0x0006C990`,
//!   `0x0006C9F0`;
//! - Misc: `miscserver.exe + miscserver.pdb`:
//!   init `0x00012C20`, cleanup `0x00012CB0`, ctor `0x00012CC0`, dtor
//!   `0x00012D10`, `Create` `0x00012D40`, `Bind` `0x00012D50`, `Close`
//!   `0x00012E50`, `OnClose` `0x00012E70`, `Recv` `0x00012E80`, `Send`
//!   `0x00012F00`, `RecvFrom` `0x00012F10`, `Sendto` `0x00013000`,
//!   `GetSocketID` `0x000130C0`; неиспользованные `SetIP/WSACreate` не emitted;
//! - Game: `gameserver.exe + GameServer.pdb`:
//!   `SetIP` `0x00001DD0`, init `0x0001AAC0`, cleanup `0x0001AB50`, ctor
//!   `0x0001AB60`, dtor `0x0001ABB0`, `Create` `0x0001ABE0`, `Bind`
//!   `0x0001ABF0`, `Close` `0x0001ACF0`, `OnClose` `0x0001AD10`, `Recv`
//!   `0x0001AD20`, `RecvFrom` `0x0001ADA0`, `Sendto` `0x0001AEA0`,
//!   `GetSocketID` `0x0001AF60`, `WSACreate` `0x0001AFB0`, вынесенный linker
//!   `Send` `0x001B7020`;
//! - World: `Nworldserver.exe + WorldServer.pdb`:
//!   `SetIP` `0x000011C0`, init `0x0002A060`, cleanup `0x0002A0F0`, ctor
//!   `0x0002A100`, dtor `0x0002A150`, `Create` `0x0002A180`, `Bind`
//!   `0x0002A190`, `Close` `0x0002A290`, `OnClose` `0x0002A2B0`, `Recv`
//!   `0x0002A2C0`, `RecvFrom` `0x0002A340`, `Sendto` `0x0002A440`,
//!   `GetSocketID` `0x0002A500`, `WSACreate` `0x0002A550`, вынесенный linker
//!   `Send` `0x000DBD10`.
//!
//! Исходные пути PDB:
//! `h:\fengyun\fy_russia\src\nets\mysocket.{cpp,h}`,
//! `d:\complite_version\fengyun_russia\trunk\nets\mysocket.{cpp,h}` и
//! `e:\svn\fengyun_russia_dev\nets\mysocket.{cpp,h}`.
//!
//! Общий конструктор задавал protocol/type `1`, IPv4 `127.0.0.1`, port `5000`,
//! invalid socket, нулевой последний UDP-port и пустой последний UDP-IP.
//! `SetIP` выполнял неконтролируемый C-string copy; Rust-владелец конфигурации
//! не должен восстанавливать переполнение, но обязан сохранить байтовую
//! кодировку и доказанные ограничения фактического поля.
//!
//! `WSACreate` создавал overlapped `AF_INET` socket с переданным type и сразу
//! вызывал `Bind`; все найденные `CServer::Host` передают type `1` (`TCP`).
//! `Bind` трактовал nullable IP как `0.0.0.0`, порт сужал до `u16`, строку
//! разбирал через legacy `inet_addr` и отвергал результат `INADDR_NONE`.
//! Технический `crate::transport::bind_tcp_ipv4` сохраняет отдельные create и
//! bind, не включает `SO_REUSEADDR` и возвращает ещё не слушающий `TcpSocket`:
//! backlog остаётся у `CServer::Listen`.
//!
//! Базовые виртуальные `Create` и `Send` намеренно возвращали `0` и `1`; живые
//! реализации принадлежат производным `CClient/CServerClient`, поэтому пустых
//! Rust-методов здесь нет. `Recv` возвращал число байт или socket error, отдельно
//! узнавая `WSAEWOULDBLOCK`; `RecvFrom` на любой ошибке возвращал `0`, а при
//! успехе — число байт, dotted IPv4 и host-order port. `Sendto` повторял вызов
//! при `WSAEWOULDBLOCK`, на успехе возвращал `1`, иначе `0`. Эти различающиеся
//! return-контракты обязаны быть сохранены непосредственным UDP/TCP owner, если
//! соответствующий путь окажется живым.
//!
//! `WSAStartup/WSACleanup`, socket handle `-1`, `closesocket`, vtable,
//! security-cookie, imported CRT/WinSock и чужие STL-тела не получают Rust-
//! аналогов. Tokio закрывает socket через `Drop` и предоставляет Linux
//! readiness вместо `WSAEventSelect/IOCP`; очередность, лимиты чтения, connect
//! timeout и публикация команд остаются обязанностью будущих `clients/servers`.
//!
//! `GetSocketID` увеличивал отдельный process-global signed `long` и возвращал
//! новое значение. `VERIFIED_DISASSEMBLY`: Misc RVA `0x000130C0` читает,
//! увеличивает и записывает storage RVA `0x001505B0`; адрес лежит в нулевом
//! virtual tail секции `.data`, поэтому loader задавал исходный ноль. В едином
//! процессе каждому историческому сервису нужен свой `SocketIdAllocator`: один
//! общий static изменил бы ID из-за активности других бывших EXE. `AtomicU32`
//! сохраняет 32-битное машинное wrapping и устраняет неопределённую data race,
//! не меняя последовательность одного сервиса.

//! Платформенный I/O принадлежит владельцу процесса; Shared несёт только
//! значения endpoint, legacy-грамматику адресов и шаблон выдачи socket ID.

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
