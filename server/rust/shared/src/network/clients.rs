//! Общий исходящий TCP-клиент исторических серверных направлений
//! (`nets/clients.cpp/.h`, пары EXE/PDB Login/Misc/Game/World в
//! `server/rust/src/manifest/`). Реализованы owned send-команды с приоритетом,
//! отправка до конца с возвратом неполного хвоста, счётчики трафика и
//! десятисекундный connect к уже разрешённому IPv4 endpoint. Receive framing,
//! hostname resolution и callbacks конкретных направлений остаются за
//! владельцами роли.
//!
//! `SendToServer` немедленно копировал вход и ставил `SENDTOSOCKET` в начало
//! (priority) или конец очереди. Send-loop атомарно забирал очередь и слал
//! каждую команду до конца; при временной невозможности или I/O-ошибке
//! owned-копия неполного хвоста возвращалась вместе с остальными перед
//! поступившими параллельно. Поле `lSocketID` роли не играло (четыре
//! `IsSameSocketID` безусловно возвращали `true`); `lNum2` передавался в
//! `send` как flags и сохраняется буквально. Решение закрыть соединение
//! остаётся у направления, как и исходный `OnClose`.
//!
//! `Connect` ждал `FD_CONNECT` ровно 10000 ms; DNS не включён в этот предел,
//! исходный `gethostbyname` выполнялся до ожидания события. Receive начинал с
//! buffer capacity `0x100000`, читал не более `0x2800` за вызов и передавал
//! `frame[4..declared_len]` конкретному `CMessage::CreateMessage`.
//!
//! Направление обслуживания и тип сообщения остаются у владельца роли; Shared
//! несёт transport и семантику частичной отправки с возвратом хвоста.
//! Доказательства: docs/reconstruction/shared-technical.md#исходящий-tcp-клиент-cclient

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;
use std::net::SocketAddrV4;
use std::time::Duration;

use tokio::net::{TcpSocket, TcpStream};
use tokio::time;

use super::socketcommands::CSocketCommands;
use super::transport::{connect_tcp_ipv4, try_write_tcp};

/// Исходный предел ожидания события успешного TCP connect.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Начальная ёмкость receive-buffer исходного `CClient`.
pub const INITIAL_RECEIVE_CAPACITY: usize = 0x10_0000;
/// Максимум байт одного исходного вызова `recv`.
pub const MAX_RECEIVE_CHUNK: usize = 0x2800;

const RATE_SAMPLE_THRESHOLD: i32 = 0x9F_FFFF;

/// Ошибка исходящего подключения к уже разрешённому IPv4 endpoint.
#[derive(Debug)]
pub enum ClientConnectError {
    /// Событие успешного подключения не наступило за исходные десять секунд.
    Timeout,
    /// Transport вернул системную ошибку создания соединения.
    Io(io::Error),
}

impl fmt::Display for ClientConnectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => formatter.write_str("подключение не завершилось за 10 секунд"),
            Self::Io(error) => write!(formatter, "ошибка исходящего TCP-подключения: {error}"),
        }
    }
}

impl Error for ClientConnectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Timeout => None,
            Self::Io(error) => Some(error),
        }
    }
}

/// Подключает ранее созданный и bind-нутый TCP socket к одному IPv4 endpoint.
///
/// Timeout охватывает только connect, как исходное ожидание `FD_CONNECT` после
/// синхронного разрешения имени. При ошибке socket закрывается через `Drop`;
/// политику повторного создания определит конкретный `net*`-владелец.
pub async fn connect(
    socket: TcpSocket,
    remote: SocketAddrV4,
) -> Result<TcpStream, ClientConnectError> {
    match time::timeout(CONNECT_TIMEOUT, connect_tcp_ipv4(socket, remote)).await {
        Ok(Ok(stream)) => Ok(stream),
        Ok(Err(error)) => Err(ClientConnectError::Io(error)),
        Err(_) => Err(ClientConnectError::Timeout),
    }
}

/// Владеющая команда отправки одного непрерывного legacy buffer.
pub struct ClientSendCommand {
    buffer: Vec<u8>,
    offset: usize,
    flags: i32,
}

impl ClientSendCommand {
    fn new(buffer: &[u8], flags: i32) -> Self {
        Self {
            buffer: buffer.to_vec(),
            offset: 0,
            flags,
        }
    }

    fn remaining(&self) -> &[u8] {
        &self.buffer[self.offset..]
    }

    fn advance(&mut self, written: usize) {
        self.offset += written;
    }
}

/// Результат неблокирующей отправки текущего снимка очереди.
#[derive(Debug, Eq, PartialEq)]
pub enum FlushOutcome {
    /// Все команды снимка отправлены; новые команды могли поступить параллельно.
    Drained { bytes_sent: u64 },
    /// Socket перестал быть writable; неполный хвост уже возвращён в очередь.
    WouldBlock { bytes_sent: u64 },
}

/// Ошибка отправки с сохранённым в очереди неполным хвостом.
#[derive(Debug)]
pub enum ClientSendError {
    /// Ненулевые legacy flags ещё не сопоставлены Linux API по живым call sites.
    UnsupportedFlags { flags: i32, bytes_sent: u64 },
    /// Ненулевой buffer получил нулевой результат записи.
    WriteZero { bytes_sent: u64 },
    /// Transport вернул системную ошибку.
    Io { source: io::Error, bytes_sent: u64 },
}

impl fmt::Display for ClientSendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFlags { flags, .. } => {
                write!(
                    formatter,
                    "не восстановлена отправка с socket flags {flags}"
                )
            }
            Self::WriteZero { .. } => {
                formatter.write_str("TCP transport вернул нулевую запись для непустого буфера")
            }
            Self::Io { source, .. } => write!(formatter, "ошибка исходящей TCP-отправки: {source}"),
        }
    }
}

impl Error for ClientSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::UnsupportedFlags { .. } | Self::WriteZero { .. } => None,
        }
    }
}

/// Очередь исходящих команд одного исторического сервиса.
///
/// Исходный `GetSocketCommand` возвращал process-global
/// `m_ClientSocketOperaCommands`. В объединённом процессе экземпляр обязан
/// принадлежать конкретному бывшему EXE, а не становиться общим для всех шести
/// сервисов.
pub struct ClientSendQueue {
    commands: CSocketCommands<ClientSendCommand>,
}

impl ClientSendQueue {
    /// Создаёт пустую очередь исходящего клиента.
    pub fn new() -> Self {
        Self {
            commands: CSocketCommands::new(),
        }
    }

    /// Немедленно копирует buffer и ставит его в начало либо конец очереди.
    ///
    /// Возвращает исходный безусловный успех `1`: после получения корректного
    /// Rust-среза команда всегда владеет своей копией.
    pub fn send_to_server(&self, buffer: &[u8], prioritized: bool, flags: i32) -> i32 {
        let command = ClientSendCommand::new(buffer, flags);
        if prioritized {
            self.commands.push_front(command);
        } else {
            self.commands.push_back(command);
        }
        1
    }

    /// Возвращает число ожидающих команд в исходном 32-битном типе.
    pub fn pending(&self) -> i32 {
        self.commands.get_size()
    }

    /// Пытается отправить атомарно взятый снимок очереди без busy loop.
    ///
    /// При `WouldBlock`, нулевой записи, неподдержанном flags либо I/O-ошибке
    /// текущий неполный buffer и остальные команды возвращаются перед всеми
    /// командами, поступившими после начала вызова.
    pub fn try_flush(&self, stream: &TcpStream) -> Result<FlushOutcome, ClientSendError> {
        let mut batch = self.commands.take_all();
        let mut bytes_sent = 0_u64;

        while let Some(mut command) = batch.pop_front() {
            if command.flags != 0 {
                let flags = command.flags;
                restore_unsent(&self.commands, command, batch);
                return Err(ClientSendError::UnsupportedFlags { flags, bytes_sent });
            }

            while !command.remaining().is_empty() {
                match try_write_tcp(stream, command.remaining()) {
                    Ok(0) => {
                        restore_unsent(&self.commands, command, batch);
                        return Err(ClientSendError::WriteZero { bytes_sent });
                    }
                    Ok(written) => {
                        command.advance(written);
                        bytes_sent = bytes_sent.wrapping_add(written as u64);
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        restore_unsent(&self.commands, command, batch);
                        return Ok(FlushOutcome::WouldBlock { bytes_sent });
                    }
                    Err(source) => {
                        restore_unsent(&self.commands, command, batch);
                        return Err(ClientSendError::Io { source, bytes_sent });
                    }
                }
            }
        }

        Ok(FlushOutcome::Drained { bytes_sent })
    }
}

impl Default for ClientSendQueue {
    fn default() -> Self {
        Self::new()
    }
}

fn restore_unsent(
    commands: &CSocketCommands<ClientSendCommand>,
    command: ClientSendCommand,
    mut remainder: VecDeque<ClientSendCommand>,
) {
    remainder.push_front(command);
    commands.prepend(remainder);
}

/// Накопитель исходной статистики send/receive одного направления.
pub struct TransferCounter {
    window_started_ms: u32,
    window_bytes: i32,
    bytes_per_second: i32,
    total_bytes: i64,
}

impl TransferCounter {
    /// Начинает пустое окно с переданного wrapping monotonic tick.
    pub const fn new(now_ms: u32) -> Self {
        Self {
            window_started_ms: now_ms,
            window_bytes: 0,
            bytes_per_second: 0,
            total_bytes: 0,
        }
    }

    /// Добавляет signed Windows `long` и возвращает последнюю рассчитанную
    /// скорость в байтах в секунду.
    ///
    /// Пересчёт происходит только после превышения `0x9F_FFFF` байт. Tick и
    /// 32-битная арифметика сохраняют wrapping `timeGetTime` и x86 `long`.
    pub fn add(&mut self, amount: i32, now_ms: u32) -> i32 {
        self.window_bytes = self.window_bytes.wrapping_add(amount);
        self.total_bytes = self.total_bytes.wrapping_add(i64::from(amount));

        if self.window_bytes > RATE_SAMPLE_THRESHOLD {
            let elapsed_ms = now_ms.wrapping_sub(self.window_started_ms);
            let scaled = self.window_bytes.wrapping_mul(1000) as u32;
            if let Some(bytes_per_second) = scaled.checked_div(elapsed_ms) {
                self.bytes_per_second = bytes_per_second as i32;
                self.window_started_ms = now_ms;
                self.window_bytes = 0;
            }
        }

        self.bytes_per_second
    }

    /// Возвращает signed 64-битный накопительный объём исходного счётчика.
    pub const fn total_bytes(&self) -> i64 {
        self.total_bytes
    }
}

// BLOCKED_MISSING_FACT: `Connect` сначала без проверки копировал вход в
// 64-байтовый stack buffer, затем для имени, начинающегося не с цифры, выбирал
// первый IPv4 `gethostbyname`; `ConnectServer` повторял fallback после
// `inet_addr == INADDR_NONE`. До аудита реальных config-строк не назначаются
// новые правила длины, legacy IPv4-форм и выбора адреса. Текущий `connect`
// принимает уже разрешённый `SocketAddrV4` и не маскирует эту неизвестность.

// BLOCKED_MISSING_FACT: `OnReceive` использовал declared little-endian `u32`
// как общий размер frame и передавал `declared_len - 4` в `CreateMessage`.
// Для `declared_len` 0..3 и значений с установленным sign bit в исходнике
// достижимы unsigned underflow либо чтение вне фактического frame. Пока не
// доказаны внешняя достижимость и реакция каждого конкретного `CMessage`, эти
// malformed-границы не получают придуманного общего fail-closed результата.
