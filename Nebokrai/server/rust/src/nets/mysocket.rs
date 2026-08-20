//! Общие наблюдаемые факты исходного `CMySocket`, восстановленные из
//! `nets/mysocket.cpp` и `.h` без создания пустого аналога WinSock-класса.
//!
//! Статус владельца: `IMPLEMENTED`. Платформенный I/O передан техническому
//! владельцу `crate::transport`; `GetSocketID` и совместимые значения endpoint
//! реализованы здесь.
//!
//! Точные пары и основные RVA:
//! - Auth: `authserver.exe + authserver.pdb`, SHA-256 EXE
//!   `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15`, PDB
//!   `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`:
//!   `SetIP` `0x00001EA0`, init `0x00013030`, cleanup `0x000130C0`, ctor
//!   `0x000130D0`, dtor `0x00013130`, `Create` `0x00013160`, `Bind`
//!   `0x00013170`, `Close` `0x00013270`, `OnClose` `0x00013290`, `Recv`
//!   `0x000132A0`, `Send` `0x00013320`, `RecvFrom` `0x00013330`, `Sendto`
//!   `0x00013420`, `GetSocketID` `0x000134E0`, `WSACreate` `0x00013530`;
//! - Billing: `billingserver.exe + billingserver.pdb`, SHA-256 EXE
//!   `FA32E3C043CB49965686129696A4EB34B733ACA1D60CAF57D369F97D5E68FB19`, PDB
//!   `F900CD0330BEFF32AC071B107AB653FD403CD18746896B3C0187C5751ACA0B21`:
//!   соответственно `0x00001860`, `0x0000D600`, `0x0000D690`, `0x0000D6A0`,
//!   `0x0000D6F0`, `0x0000D720`, `0x0000D730`, `0x0000D830`, `0x0000D850`,
//!   `0x0000D860`, `0x0000D8E0`, `0x0000D8F0`, `0x0000D9E0`, `0x0000DAA0`,
//!   `0x0000DAF0`;
//! - Login: `loginserver.exe + LoginServer.pdb`, SHA-256 EXE
//!   `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876`, PDB
//!   `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`:
//!   соответственно `0x00002AF0`, `0x0006C4F0`, `0x0006C580`, `0x0006C590`,
//!   `0x0006C5E0`, `0x0006C610`, `0x0006C620`, `0x0006C720`, `0x0006C740`,
//!   `0x0006C750`, `0x0006C7D0`, `0x0006C7E0`, `0x0006C8D0`, `0x0006C990`,
//!   `0x0006C9F0`;
//! - Misc: `miscserver.exe + miscserver.pdb`, SHA-256 EXE
//!   `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65`, PDB
//!   `ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7`:
//!   init `0x00012C20`, cleanup `0x00012CB0`, ctor `0x00012CC0`, dtor
//!   `0x00012D10`, `Create` `0x00012D40`, `Bind` `0x00012D50`, `Close`
//!   `0x00012E50`, `OnClose` `0x00012E70`, `Recv` `0x00012E80`, `Send`
//!   `0x00012F00`, `RecvFrom` `0x00012F10`, `Sendto` `0x00013000`,
//!   `GetSocketID` `0x000130C0`; неиспользованные `SetIP/WSACreate` не emitted;
//! - Game: `gameserver.exe + GameServer.pdb`, SHA-256 EXE
//!   `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, PDB
//!   `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`:
//!   `SetIP` `0x00001DD0`, init `0x0001AAC0`, cleanup `0x0001AB50`, ctor
//!   `0x0001AB60`, dtor `0x0001ABB0`, `Create` `0x0001ABE0`, `Bind`
//!   `0x0001ABF0`, `Close` `0x0001ACF0`, `OnClose` `0x0001AD10`, `Recv`
//!   `0x0001AD20`, `RecvFrom` `0x0001ADA0`, `Sendto` `0x0001AEA0`,
//!   `GetSocketID` `0x0001AF60`, `WSACreate` `0x0001AFB0`, вынесенный linker
//!   `Send` `0x001B7020`;
//! - World: `Nworldserver.exe + WorldServer.pdb`, SHA-256 EXE
//!   `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//!   `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`:
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

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::atomic::{AtomicU32, Ordering};

/// Исходный protocol/type по умолчанию (`SOCK_STREAM`).
pub(crate) const DEFAULT_SOCKET_TYPE: i32 = 1;
/// Исходный локальный порт до явной настройки.
pub(crate) const DEFAULT_PORT: u32 = 5000;
/// Исходный IPv4 до явной настройки.
pub(crate) const DEFAULT_IP: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Представляет network-order IPv4 bytes как исходный x86 `unsigned long`.
pub(crate) fn legacy_ipv4_word(address: Ipv4Addr) -> u32 {
    u32::from_le_bytes(address.octets())
}

/// Преобразует доказанный nullable IPv4 и 32-битный port в адрес старого bind.
///
/// `None` соответствует `INADDR_ANY`; port сужается как исходный cast к
/// `u_short`, а не валидируется как новая конфигурационная политика.
pub(crate) fn legacy_bind_endpoint(address: Option<Ipv4Addr>, port: u32) -> SocketAddrV4 {
    SocketAddrV4::new(address.unwrap_or(Ipv4Addr::UNSPECIFIED), port as u16)
}

/// Независимый счётчик socket ID одного исторического сервиса.
pub(crate) struct SocketIdAllocator {
    last_issued: AtomicU32,
}

impl SocketIdAllocator {
    /// Создаёт счётчик с исходным нулевым process-global значением.
    pub(crate) const fn new() -> Self {
        Self {
            last_issued: AtomicU32::new(0),
        }
    }

    /// Выдаёт следующий signed 32-битный ID с машинным wrapping оригинала.
    pub(crate) fn next(&self) -> i32 {
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

// BLOCKED_MISSING_FACT: `inet_addr` принимал legacy IPv4-формы (не только
// dotted-decimal) и одновременно представлял `255.255.255.255` как
// `INADDR_NONE`. До аудита реальных конфигурационных значений строковый parser
// не заменяется строгим `Ipv4Addr::from_str` и не переносится в transport.
