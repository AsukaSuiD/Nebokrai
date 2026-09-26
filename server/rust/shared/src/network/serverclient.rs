//! Состояние принятого TCP-соединения `CServerClient`, восстановленное из
//! `nets/serverclient.cpp` и `.h`.
//!
//! Статус владельца: `IMPLEMENTED` для connection metadata, накопления
//! receive/send bytes, серверного send-limit, полной transport-записи batch,
//! числа незавершённых send-операций, двухступенчатого close flag и per-client
//! receive-rate. Разбор
//! 12-байтового envelope до конкретного `CMessage` остаётся владельцам
//! направлений: их типы сообщений и набор присваиваемых metadata различаются.
//! Формат внешнего TCP-обрамления: docs/protocol/transport.md.
//!
//! Точные варианты корпуса — пары EXE/PDB Auth, Billing, Login, Game и World;
//! их идентификаторы (SHA-256) зафиксированы в `server/rust/src/manifest/`.
//!
//! Исходные пути PDB:
//! `h:\fengyun\fy_russia\src\nets\serverclient.{cpp,h}`,
//! `d:\complite_version\fengyun_russia\trunk\nets\serverclient.{cpp,h}` и
//! `e:\svn\fengyun_russia_dev\nets\serverclient.{cpp,h}`.
//!
//! Существенные RVA по порядку Auth / Billing / Login / Game / World:
//! - `ReadFromCompletionPort`: `0x000142C0` / `0x0000DBC0` / `0x0006CFC0` /
//!   `0x0001B080` / `0x0002A620`;
//! - `Send`: `0x00014320` / `0x0000DC20` / `0x0006D020` / `0x0001B0E0` /
//!   `0x0002A680`;
//! - `AddReceiveData`: `0x00014490` / `0x0000DD90` / `0x0006D190` /
//!   `0x0001B250` / `0x0002A7F0`;
//! - `AddSendData`: `0x00014540` / `0x0000DE40` / `0x0006D240` /
//!   `0x0001B300` / `0x0002A8A0`;
//! - `AddPackageSize`: `0x000146D0` / `0x0000DFD0` / `0x0006D3D0` /
//!   `0x0001B480` / `0x0002AA20`;
//! - конструктор: `0x000147E0` / `0x0000E0F0` / `0x0006D4E0` /
//!   `0x0001B850` / `0x0002ADF0`;
//! - `OnReceive`: `0x000148A0` / `0x0000E1B0` / `0x0006D5A0` /
//!   `0x0001B580` / `0x0002AB20`.
//!
//! `AddSendData` последовательно объединял команды в один buffer. Он возвращал
//! `false`, если новый суммарный размер превышал
//! `CServer::m_lPermitMaxClientSendBufSize` (исходный default `0xC800`), но при
//! уже установленном `m_bCloseFlag` ничего не добавлял и всё равно возвращал
//! `true`. Эта странность сохранена. Рост capacity вдвое, временные копии и
//! аварийный `exit(0)` после невозможного для старого кода `operator_new == 0`
//! не являются сетевой семантикой; `Vec<u8>` сохраняет только bytes и порядок.
//!
//! `Send` не использовал свои pointer/length arguments: он копировал весь
//! накопленный buffer в отдельную overlapped-операцию, очищал накопитель сразу
//! после принятия `WSASend`, увеличивал `m_lIOOperatorNum` и считал запрошенный,
//! а не подтверждённый размер. Точный аудит `CServer::DoWorkerThreadFunc` (RVA
//! Auth `0x0000DE60`, Billing `0x00007D00`, Login `0x000669D0`, Game
//! `0x00015290`, World `0x000244B0`) показал: если completion передавал меньше
//! bytes, код только логировал отличие, освобождал весь buffer и публиковал
//! `SENDEND`; остаток не отправлялся повторно. Это факт об IOCP completion,
//! а не о частичном Linux `write`. Rust дописывает batch до завершения операции:
//! readiness и один syscall не означают принятие транспортом всей заявки.
//!
//! WinSock IOCP, `PER_IO_OPERATION_DATA`, ручные allocation/free и отдельная
//! `SENDEND`-команда заменены владеющим `ServerSendBatch` и явным завершением
//! операции. Close flag отдельно от одноразового начала shutdown сохраняет
//! старую границу `QUIT -> closesocket -> worker DELETE -> OnClose/Drop`.
//! Ожидание writable может повторяться при ложной readiness; после частичной
//! записи передаётся только оставшийся хвост. Нулевой write для
//! непустого batch соответствует исходному disconnect-пути, а не partial send.
//!
//! Receive-path накапливал TCP-фрагменты, начиная с capacity `0x100000`;
//! IOCP-приём использовал блок `0x2000`. `OnReceive` разбирал envelope
//! `[total_len, crc(total_len), crc(message), message]`, проверял IEEE CRC,
//! создавал компонентный `CMessage`, затем ставил socket ID, map ID, peer IPv4,
//! а Auth/Billing/Login также копировали байтовую map/CD-key строку. Только
//! после этого сообщение передавалось `CServer::m_RecvMessages`. Общая фабрика
//! здесь не вводится: соответствующие `message.rs` обязаны принять
//! `ServerClientMessageContext` при собственной реализации.
//!
//! Конструктор также задавал `lost=false`, `quit=false`, `server_type=0`, пустую
//! map-строку, `map_id=0`, `close=false` и нулевое число I/O. Defaults без
//! доказанного потребителя здесь только зафиксированы, но не представлены
//! пустыми Rust-полями; живые map/close/I/O-состояния сохранены. Пустой
//! virtual `OnOneMessageSizeOver` отдельного Rust-тела не получает.

use std::io;
use std::mem;

use std::error::Error;
use std::fmt;

use tokio::net::TcpStream;

use super::transport::write_once_tcp;

/// Исходная начальная ёмкость накопителя входящего потока.
pub const INITIAL_RECEIVE_CAPACITY: usize = 0x10_0000;
/// Исходный default максимума накопленных исходящих bytes одного соединения.
pub const DEFAULT_PERMITTED_SEND_BYTES: i32 = 0xC800;

const RECEIVE_RATE_SAMPLE_THRESHOLD: i32 = 0xC7FF;

/// Ошибка размера за пределами безопасной signed 32-битной модели оригинала.
#[derive(Debug, Eq, PartialEq)]
pub enum ServerClientSizeError {
    /// Сумма входящих TCP-фрагментов переполнила бы исходный `m_nSize`.
    ReceiveSizeOverflowReactionUnknown,
    /// Сумма исходящих данных переполнила бы исходный signed размер buffer.
    SendSizeOverflowReactionUnknown,
}

impl fmt::Display for ServerClientSizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReceiveSizeOverflowReactionUnknown => write!(
                formatter,
                "размер входного буфера вышел за signed 32-битную границу оригинала"
            ),
            Self::SendSizeOverflowReactionUnknown => write!(
                formatter,
                "размер исходного буфера вышел за signed 32-битную границу оригинала"
            ),
        }
    }
}

impl Error for ServerClientSizeError {}

/// Metadata, которыми `CServerClient::OnReceive` дополнял готовое сообщение.
#[derive(Clone, Copy)]
pub struct ServerClientMessageContext<'a> {
    /// ID принятого socket внутри исторического сервиса.
    pub socket_id: i32,
    /// Числовой ID карты, исходно равный нулю.
    pub map_id: i32,
    /// Буквальные bytes исходной `std::string`, без предположения UTF-8.
    pub map_name: &'a [u8],
    /// Исходное 32-битное представление peer IPv4.
    pub peer_ipv4: u32,
}

/// Результат накопления данных для будущей send-операции.
#[derive(Debug, Eq, PartialEq)]
pub enum AddSendDataOutcome {
    /// Bytes добавлены в конец текущего batch.
    Buffered,
    /// Соединение уже закрывается: bytes отброшены, но оригинал вернул `true`.
    IgnoredWhileClosing,
    /// Новый суммарный размер превысил разрешённый сервером предел.
    LimitExceeded,
}

/// Один owned batch, соответствующий одной старой overlapped `WSASend`.
pub struct ServerSendBatch {
    buffer: Vec<u8>,
}

impl ServerSendBatch {
    /// Возвращает число bytes, которое оригинал учитывал при submit операции.
    pub fn requested_bytes(&self) -> usize {
        self.buffer.len()
    }

    /// Дописывает batch последовательно; один syscall не равен IOCP completion.
    pub async fn write_complete(self, stream: &TcpStream) -> io::Result<ServerSendCompletion> {
        let requested = self.buffer.len();
        let mut transferred = 0;
        while transferred < requested {
            let written = write_once_tcp(stream, &self.buffer[transferred..]).await?;
            if written == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "TCP-соединение закрылось во время отправки server-client batch",
                ));
            }
            transferred += written;
        }
        Ok(ServerSendCompletion {
            requested,
            transferred,
        })
    }
}

/// Completion одной исходящей операции принятого соединения.
#[derive(Debug, Eq, PartialEq)]
pub struct ServerSendCompletion {
    requested: usize,
    transferred: usize,
}

impl ServerSendCompletion {
    /// Возвращает число bytes, подтверждённых transport.
    pub const fn transferred_bytes(&self) -> usize {
        self.transferred
    }

    /// Возвращает число bytes, которые оригинал терял после partial completion.
    pub const fn dropped_tail_bytes(&self) -> usize {
        self.requested - self.transferred
    }

    /// Сообщает, совпал ли completion с полным размером операции.
    pub const fn is_complete(&self) -> bool {
        self.requested == self.transferred
    }
}

/// Восстановленное общее состояние одного принятого соединения.
/// Установленный экземпляр и его потребители остаются у владельца роли.
pub struct CServerClient {
    socket_id: i32,
    peer_ipv4: u32,
    map_id: i32,
    map_name: Vec<u8>,
    closing: bool,
    close_started: bool,
    receive_buffer: Vec<u8>,
    receive_shrink_target: usize,
    send_buffer: Vec<u8>,
    io_operations: i32,
    receive_rate: PackageRateCounter,
}

impl CServerClient {
    /// Создаёт состояние нового принятого соединения с исходными defaults.
    pub fn new(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> Self {
        Self::with_receive_shrink_target(
            socket_id,
            peer_ipv4,
            now_ms,
            INITIAL_RECEIVE_CAPACITY,
            INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Создаёт то же общее состояние с доказанной component capacity.
    ///
    /// Auth-производный клиент начинал с `0xA00000`, тогда как обычный
    /// `CServerClient` — с `0x100000`. Это техническая форма accumulator, а не
    /// новая общая настройка сервера.
    pub fn with_receive_capacity(
        socket_id: i32,
        peer_ipv4: u32,
        now_ms: u32,
        receive_capacity: usize,
    ) -> Self {
        Self::with_receive_shrink_target(
            socket_id,
            peer_ipv4,
            now_ms,
            receive_capacity,
            INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Создаёт то же общее состояние с доказанным component shrink target.
    ///
    /// World принятого клиента после разбора сжимает accumulator обратно не к
    /// общему `0x100000`, а к своему начальному `0x1400000` (машинно
    /// подтверждённая форма `CMyServerClient::OnReceive`); Billing — к
    /// `0x100000`. Shrink target — свойство component receive path, а не
    /// единый для всех планов.
    pub fn with_receive_shrink_target(
        socket_id: i32,
        peer_ipv4: u32,
        now_ms: u32,
        receive_capacity: usize,
        receive_shrink_target: usize,
    ) -> Self {
        Self {
            socket_id,
            peer_ipv4,
            map_id: 0,
            map_name: Vec::new(),
            closing: false,
            close_started: false,
            receive_buffer: Vec::with_capacity(receive_capacity),
            receive_shrink_target,
            send_buffer: Vec::new(),
            io_operations: 0,
            receive_rate: PackageRateCounter::new(now_ms),
        }
    }

    /// Возвращает metadata для конкретного component message-owner.
    pub fn message_context(&self) -> ServerClientMessageContext<'_> {
        ServerClientMessageContext {
            socket_id: self.socket_id,
            map_id: self.map_id,
            map_name: &self.map_name,
            peer_ipv4: self.peer_ipv4,
        }
    }

    /// Обновляет доказанную map identity без предположения о кодировке строки.
    pub fn set_map_identity(&mut self, map_id: i32, map_name: &[u8]) {
        self.map_id = map_id;
        self.map_name.clear();
        self.map_name.extend_from_slice(map_name);
    }

    /// Обновляет только числовую map identity, как `PLAYERJOIN` server-command.
    pub fn set_map_id(&mut self, map_id: i32) {
        self.map_id = map_id;
    }

    /// Обновляет только строковую identity, как `CDKEYJOIN` server-command.
    pub fn set_map_name(&mut self, map_name: &[u8]) {
        self.map_name.clear();
        self.map_name.extend_from_slice(map_name);
    }

    /// Устанавливает исходный close flag; последующие send-data будут отброшены.
    pub fn mark_closing(&mut self) {
        self.closing = true;
    }

    /// Сбрасывает close flag при доказанном component `OnAccept`.
    pub fn mark_open(&mut self) {
        self.closing = false;
        self.close_started = false;
    }

    /// Сообщает, установлен ли исходный close flag.
    pub const fn is_closing(&self) -> bool {
        self.closing
    }

    /// Возвращает `true` ровно один раз после установки close flag.
    pub fn begin_close(&mut self) -> bool {
        if !self.closing || self.close_started {
            return false;
        }
        self.close_started = true;
        true
    }

    /// Добавляет уже прочитанный TCP-фрагмент без разбора component message.
    pub fn add_receive_data(&mut self, received: &[u8]) -> Result<(), ServerClientSizeError> {
        let Some(combined) = self.receive_buffer.len().checked_add(received.len()) else {
            return Err(ServerClientSizeError::ReceiveSizeOverflowReactionUnknown);
        };
        if combined > i32::MAX as usize {
            return Err(ServerClientSizeError::ReceiveSizeOverflowReactionUnknown);
        }
        self.receive_buffer.extend_from_slice(received);
        Ok(())
    }

    /// Возвращает число накопленных bytes будущему component receive-owner.
    pub fn pending_receive_bytes(&self) -> usize {
        self.receive_buffer.len()
    }

    /// Предоставляет накопленный TCP-поток component receive-owner без копии.
    pub fn receive_bytes(&self) -> &[u8] {
        &self.receive_buffer
    }

    /// Удаляет уже разобранный префикс, сохраняя неполный хвост следующего кадра.
    ///
    /// Возвращает `false`, если вызывающий попытался снять больше накопленного:
    /// общий owner не назначает такому рассогласованию реакцию component parser.
    pub fn consume_receive_prefix(&mut self, count: usize) -> bool {
        if count > self.receive_buffer.len() {
            return false;
        }

        self.receive_buffer.drain(..count);
        if self.receive_buffer.capacity() > self.receive_shrink_target
            && self.receive_buffer.len() <= self.receive_shrink_target
        {
            self.receive_buffer.shrink_to(self.receive_shrink_target);
        }
        true
    }

    /// Отбрасывает накопленный входной поток при завершении соединения.
    pub fn discard_receive_data(&mut self) {
        self.receive_buffer.clear();
    }

    /// Добавляет bytes в send batch при исходном серверном ограничении.
    pub fn add_send_data(
        &mut self,
        data: &[u8],
        permitted_max: i32,
    ) -> Result<AddSendDataOutcome, ServerClientSizeError> {
        if self.closing {
            return Ok(AddSendDataOutcome::IgnoredWhileClosing);
        }

        let current = i32::try_from(self.send_buffer.len())
            .map_err(|_| ServerClientSizeError::SendSizeOverflowReactionUnknown)?;
        let incoming = i32::try_from(data.len())
            .map_err(|_| ServerClientSizeError::SendSizeOverflowReactionUnknown)?;
        let Some(combined) = current.checked_add(incoming) else {
            return Err(ServerClientSizeError::SendSizeOverflowReactionUnknown);
        };
        if combined > permitted_max {
            return Ok(AddSendDataOutcome::LimitExceeded);
        }

        self.send_buffer.extend_from_slice(data);
        Ok(AddSendDataOutcome::Buffered)
    }

    /// Возвращает размер ещё не переданного send batch.
    pub fn pending_send_bytes(&self) -> usize {
        self.send_buffer.len()
    }

    /// Передаёт текущий batch одной transport-операции и увеличивает CurIO.
    ///
    /// Пустой накопитель соответствует исходному `Send -> 1` без создания I/O
    /// и возвращает `None`. Ошибка последующего transport-write терминальна для
    /// соединения: оригинальный server удалял client, не возвращая batch назад.
    pub fn begin_send(&mut self) -> Option<ServerSendBatch> {
        if self.send_buffer.is_empty() {
            return None;
        }

        self.io_operations = self.io_operations.wrapping_add(1);
        Some(ServerSendBatch {
            buffer: mem::take(&mut self.send_buffer),
        })
    }

    /// Завершает одну send-операцию, как обработка старой команды `SENDEND`.
    pub fn finish_send_operation(&mut self) {
        self.io_operations = self.io_operations.wrapping_sub(1);
    }

    /// Возвращает signed число незавершённых send-операций.
    pub const fn io_operations(&self) -> i32 {
        self.io_operations
    }

    /// Учитывает принятые bytes и возвращает последнюю per-client скорость.
    pub fn add_package_size(&mut self, amount: i32, now_ms: u32) -> i32 {
        self.receive_rate.add(amount, now_ms)
    }
}

struct PackageRateCounter {
    window_started_ms: u32,
    window_bytes: i32,
    bytes_per_second: i32,
}

impl PackageRateCounter {
    const fn new(now_ms: u32) -> Self {
        Self {
            window_started_ms: now_ms,
            window_bytes: 0,
            bytes_per_second: 0,
        }
    }

    fn add(&mut self, amount: i32, now_ms: u32) -> i32 {
        self.window_bytes = self.window_bytes.wrapping_add(amount);
        if self.window_bytes > RECEIVE_RATE_SAMPLE_THRESHOLD {
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
}

// BLOCKED_MISSING_FACT: `OnReceive` проверяет length CRC уже при 12 bytes, но
// для `total_len < 12`, `12`, `13..27` и значений с sign bit затем достигаются
// разные underflow/короткие входы конкретного `CMessage::CreateMessage`.
// Без проверки каждой component message-фабрики общий owner не назначает этим
// malformed envelope единый fail-closed результат. Нормальный envelope и
// порядок length CRC -> create/normalize -> content CRC доказаны полностью.
