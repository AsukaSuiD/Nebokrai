//! Входной межсерверный поток MiscServer, восстановленный из
//! `nets/netmisc/mynetclient.cpp` и `.h`.
//!
//! Статус владельца: `IMPLEMENTED`; спорная signed-граница длины дополнительно
//! имеет статус `VERIFIED_DISASSEMBLY`.
//!
//! Точная пара: `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`;
//! SHA-256 EXE
//! `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65`,
//! SHA-256 PDB
//! `ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7`.
//! Исходные пути PDB:
//! `h:\fengyun\fy_russia\src\nets\netmisc\mynetclient.cpp` и `.h`.
//! Существенные RVA: конструктор `0x00012760`, деструктор `0x00012780`,
//! `HandleClose` `0x00012790`, `OnReceive` `0x00012820`.
//!
//! Живой входной контракт — поток 12-байтовых envelope:
//! `[total_len, crc(total_len), crc(message), message]`. После накопления как
//! минимум 12 байт сначала проверяется IEEE CRC первого little-endian слова.
//! Полный кадр превращается в `netmisc::CMessage`, затем CRC вычисляется уже по
//! нормализованному внутреннему сообщению. Только прошедший обе проверки объект
//! передаётся во владеющую FIFO-очередь. За один вызов разбираются все полные
//! кадры, а неполный хвост сохраняется для следующего чтения. Любая доказанная
//! ошибка CRC или создания сообщения отбрасывает весь ещё не разобранный вход;
//! уже поставленные в очередь предыдущие кадры не откатываются.
//!
//! `HandleClose` не закрывает сокет сам: общий `CClient::OnClose` сначала
//! сбрасывал connect flag, затем callback клал пустое сообщение `0x16EA01` в
//! ту же очередь. Явный `Close` synthetic сообщение не создавал. Rust хранит
//! эти пути раздельно, а один awaitable I/O-шаг заменяет два Windows socket
//! thread: читает не более `0x2800` байт либо обслуживает owned send snapshot.
//!
//! Исходный `CClient` начинал с буфера `0x100000`, при необходимости заранее
//! расширял его до объявленной длины, после разбора делал `memmove`, а затем
//! мог сжимать обратно. `Vec<u8>` сохраняет те же принятые байты и границы
//! кадров без allocator-копий и без немедленного выделения памяти по одной лишь
//! недоверенной длине. `drain` заменяет `memmove`; изменение capacity не является
//! wire- или доменной семантикой. `CMsgQueue<CMessage>` заменяет deque указателей
//! и ручное виртуальное удаление. `CGame::ProcessMessage` RVA `0x000026A0`
//! атомарно забирает все элементы через `GetAllMessage`, а системные события
//! также могут положить готовый `CMessage` в эту же очередь; Rust API сохраняет
//! обе операции без раскрытия внутреннего mutex.
//!
//! Для длины с установленным sign bit, `total_len < 12` и `13..27` исходный код
//! мог перейти к unsigned `total_len - 12` либо передать внутренний буфер короче
//! 16 байт в `CreateMessageWithoutRLE`. Наблюдаемая реакция не доказана.
//! `total_len == 12` отличается: нулевая длина внутреннего сообщения доказанно
//! возвращает `nullptr`, после чего вход очищается. Безопасный Rust локально
//! отвергает неизвестные случаи отдельной ошибкой и не выдаёт это решение за
//! поведение оригинального процесса.
//!
//! `CBaseMessage` deleting-destructor, vector cleanup `$L77738`, vtable,
//! allocator и exception plumbing классифицированы как compiler/library noise.
//! Унаследованные connect/send поля выражены композицией общего
//! `nets/clients.rs`; конкретные connect-порядок и registration packets
//! остаются `miscserver/game.rs`.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;
use std::net::SocketAddrV4;

use tokio::net::{TcpSocket, TcpStream};

use crate::nets::clients::{
    ClientConnectError, ClientSendError, ClientSendQueue, FlushOutcome, connect as connect_client,
};
use crate::nets::msgqueue::CMsgQueue;
use crate::public::crc32static::data_crc32;
use crate::transport::read_client_tcp_chunk;

use super::message::{CMessage, CreateMessageError, MessageSender};

const SERVER_ENVELOPE_LEN: usize = 12;
const INNER_MESSAGE_HEADER_LEN: usize = 16;
const MIN_SERVER_FRAME_LEN: usize = SERVER_ENVELOPE_LEN + INNER_MESSAGE_HEADER_LEN;
const INITIAL_RECEIVE_CAPACITY: usize = 0x10_0000;
const CLOSE_MESSAGE_TYPE: i32 = 0x0016_EA01;

/// Причина, по которой текущий неразобранный вход был отброшен.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ReceiveError {
    /// CRC little-endian слова `total_len` не совпал со вторым словом envelope.
    LengthChecksumMismatch { expected: u32, actual: u32 },
    /// Signed-ветвление оригинала не задаёт безопасную реакцию на этот размер.
    SignedFrameLengthReactionUnknown { declared: u32 },
    /// Длина `0..11` либо `13..27` ведёт в недоказанное короткое сообщение.
    ShortFrameReactionUnknown { declared: u32 },
    /// Создание нормализованного внутреннего сообщения завершилось ошибкой.
    Message(CreateMessageError),
    /// CRC нормализованного сообщения не совпал с третьим словом envelope.
    ContentChecksumMismatch {
        message_type: i32,
        expected: u32,
        actual: u32,
    },
    /// Размер накопителя вышел за signed 32-битную границу исходного `m_nSize`.
    PendingSizeOverflowReactionUnknown,
}

/// Результат одного Linux read/send ожидания исходящего Misc-клиента.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MiscClientIoStep {
    /// Принят один TCP block и опубликовано указанное число полных сообщений.
    Received { messages: usize },
    /// Выполнена попытка отправки текущего атомарного снимка очереди.
    Sent(FlushOutcome),
    /// EOF преобразован в доказанный `OnClose -> 0x16EA01`.
    Closed,
}

/// Ошибка одного Linux read/send ожидания Misc-клиента.
#[derive(Debug)]
pub(crate) enum MiscClientIoError {
    NotConnected,
    Io(io::Error),
    Receive(ReceiveError),
    Send(ClientSendError),
}

impl fmt::Display for MiscClientIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => formatter.write_str("MiscServer не подключён к WorldServer"),
            Self::Io(error) => write!(formatter, "ошибка I/O Misc-соединения: {error}"),
            Self::Receive(_) => formatter.write_str("WorldServer передал некорректный Misc frame"),
            Self::Send(error) => write!(formatter, "ошибка отправки Misc-соединения: {error}"),
        }
    }
}

impl Error for MiscClientIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Send(error) => Some(error),
            Self::NotConnected | Self::Receive(_) => None,
        }
    }
}

/// Производный сетевой client MiscServer в границах доказанного receive-owner.
pub(crate) struct CMyNetClient {
    connection: Option<TcpStream>,
    connected_endpoint: Option<SocketAddrV4>,
    control_send: bool,
    send_queue: ClientSendQueue,
    receive_buffer: Vec<u8>,
    received_messages: CMsgQueue<CMessage>,
}

impl CMyNetClient {
    /// Создаёт пустой накопитель исходной начальной ёмкости и очередь сообщений.
    pub(crate) fn new() -> Self {
        Self {
            connection: None,
            connected_endpoint: None,
            control_send: false,
            send_queue: ClientSendQueue::new(),
            receive_buffer: Vec::with_capacity(INITIAL_RECEIVE_CAPACITY),
            received_messages: CMsgQueue::new(),
        }
    }

    /// Подключает уже созданный и bind-нутый IPv4 socket к WorldServer.
    pub(crate) async fn connect(
        &mut self,
        socket: TcpSocket,
        remote: SocketAddrV4,
    ) -> Result<(), ClientConnectError> {
        let stream = connect_client(socket, remote).await?;
        self.connection = Some(stream);
        self.connected_endpoint = Some(remote);
        Ok(())
    }

    /// Выполняет явный `CClient::Close` без synthetic close-сообщения.
    pub(crate) fn close(&mut self) -> i32 {
        self.connection = None;
        1
    }

    /// Воспроизводит transport `OnClose -> HandleClose -> 0x16EA01`.
    pub(crate) fn handle_transport_close(&mut self) {
        self.connection = None;
        self.push_message(CMessage::new(CLOSE_MESSAGE_TYPE));
    }

    /// Сообщает состояние исходного `m_bConnect`.
    pub(crate) fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    /// Возвращает endpoint последнего успешного connect.
    pub(crate) const fn connected_endpoint(&self) -> Option<SocketAddrV4> {
        self.connected_endpoint
    }

    /// Включает исходный `m_bControlSend` после успешного connect.
    pub(crate) fn enable_control_send(&mut self) {
        self.control_send = true;
    }

    /// Возвращает общую owned send-очередь конкретного Misc-клиента.
    pub(crate) fn send_queue(&self) -> &ClientSendQueue {
        &self.send_queue
    }

    /// Пытается отправить текущий снимок очереди при включённом control-send.
    pub(crate) fn try_flush_outgoing(&self) -> Option<Result<FlushOutcome, ClientSendError>> {
        if !self.control_send {
            return None;
        }
        self.connection
            .as_ref()
            .map(|stream| self.send_queue.try_flush(stream))
    }

    /// Ждёт ровно один read либо send-readiness и выполняет один I/O-шаг.
    pub(crate) async fn run_io_once(
        &mut self,
        mut recv_time_ms: impl FnMut() -> u32,
    ) -> Result<MiscClientIoStep, MiscClientIoError> {
        enum Ready {
            Read(io::Result<Vec<u8>>),
            Write(io::Result<()>),
        }

        let ready = {
            let stream = self
                .connection
                .as_ref()
                .ok_or(MiscClientIoError::NotConnected)?;
            if self.control_send && self.send_queue.pending() > 0 {
                tokio::select! {
                    received = read_client_tcp_chunk(stream) => Ready::Read(received),
                    writable = stream.writable() => Ready::Write(writable),
                }
            } else {
                Ready::Read(read_client_tcp_chunk(stream).await)
            }
        };

        match ready {
            Ready::Read(Ok(received)) if received.is_empty() => {
                self.handle_transport_close();
                Ok(MiscClientIoStep::Closed)
            }
            Ready::Read(Ok(received)) => self
                .accept_received_bytes(&received, &mut recv_time_ms)
                .map(|messages| MiscClientIoStep::Received { messages })
                .map_err(MiscClientIoError::Receive),
            Ready::Read(Err(error)) | Ready::Write(Err(error)) => Err(MiscClientIoError::Io(error)),
            Ready::Write(Ok(())) => {
                let stream = self
                    .connection
                    .as_ref()
                    .ok_or(MiscClientIoError::NotConnected)?;
                self.send_queue
                    .try_flush(stream)
                    .map(MiscClientIoStep::Sent)
                    .map_err(MiscClientIoError::Send)
            }
        }
    }

    /// Добавляет уже прочитанный TCP-фрагмент и разбирает все полные кадры.
    ///
    /// `recv_time_ms` вызывается отдельно для каждого успешно созданного
    /// сообщения, в той же точке, где оригинал вызывал `timeGetTime`.
    /// Возвращает число новых сообщений. При ошибке весь оставшийся вход
    /// отбрасывается, тогда как ранее опубликованные сообщения сохраняются.
    pub(crate) fn accept_received_bytes(
        &mut self,
        received: &[u8],
        mut recv_time_ms: impl FnMut() -> u32,
    ) -> Result<usize, ReceiveError> {
        let pending_size = self
            .receive_buffer
            .len()
            .checked_add(received.len())
            .filter(|size| *size <= i32::MAX as usize);
        let Some(pending_size) = pending_size else {
            // BLOCKED_MISSING_FACT: исходный signed `m_nSize` не задаёт
            // пригодную реакцию на арифметическое переполнение накопителя.
            return self.discard_pending(ReceiveError::PendingSizeOverflowReactionUnknown);
        };
        self.receive_buffer
            .reserve(pending_size - self.receive_buffer.len());
        self.receive_buffer.extend_from_slice(received);

        let mut consumed = 0;
        let mut accepted = 0;

        while self.receive_buffer.len() - consumed >= SERVER_ENVELOPE_LEN {
            let frame = &self.receive_buffer[consumed..];
            let declared_bytes: [u8; 4] = frame[..4]
                .try_into()
                .expect("наличие полного envelope уже проверено");
            let declared = u32::from_le_bytes(declared_bytes);
            let expected_length_crc = u32::from_le_bytes(
                frame[4..8]
                    .try_into()
                    .expect("наличие полного envelope уже проверено"),
            );
            let actual_length_crc = data_crc32(&declared_bytes);
            if actual_length_crc != expected_length_crc {
                return self.discard_pending(ReceiveError::LengthChecksumMismatch {
                    expected: expected_length_crc,
                    actual: actual_length_crc,
                });
            }

            // BLOCKED_MISSING_FACT, VERIFIED_DISASSEMBLY: MiscServer RVA
            // 0x00012820 сравнивает `total_len > m_nSize` через signed `jg` и
            // лишь внутри этой ветки проверяет `total_len >= 0`. Для значения
            // с sign bit условие `jg` при нормальном положительном m_nSize не
            // выполняется, после чего машинный код вычисляет `total_len - 12`.
            if (declared as i32) < 0 {
                return self
                    .discard_pending(ReceiveError::SignedFrameLengthReactionUnknown { declared });
            }

            let frame_len = declared as usize;
            if frame.len() < frame_len {
                break;
            }
            if frame_len == SERVER_ENVELOPE_LEN {
                return self.discard_pending(ReceiveError::Message(CreateMessageError::EmptyInput));
            }
            if frame_len < MIN_SERVER_FRAME_LEN {
                // BLOCKED_MISSING_FACT: при `total_len < 12` исходный unsigned
                // `total_len - 12` переполняется; при 13..27 внутренний buffer
                // короче header, а CreateMessageWithoutRLE проверяет лишь ноль
                // и затем читает все 16 байт.
                return self.discard_pending(ReceiveError::ShortFrameReactionUnknown { declared });
            }

            let expected_content_crc = u32::from_le_bytes(
                frame[8..12]
                    .try_into()
                    .expect("наличие полного envelope уже проверено"),
            );
            let message_wire = &frame[SERVER_ENVELOPE_LEN..frame_len];
            let message = CMessage::create_without_rle(message_wire, recv_time_ms())
                .map_err(ReceiveError::Message);
            let message = match message {
                Ok(message) => message,
                Err(error) => return self.discard_pending(error),
            };
            let actual_content_crc = data_crc32(message.as_wire_bytes());
            if actual_content_crc != expected_content_crc {
                return self.discard_pending(ReceiveError::ContentChecksumMismatch {
                    message_type: message.message_type(),
                    expected: expected_content_crc,
                    actual: actual_content_crc,
                });
            }

            self.received_messages.push(message);
            consumed += frame_len;
            accepted += 1;
        }

        if consumed != 0 {
            self.receive_buffer.drain(..consumed);
        }
        if self.receive_buffer.capacity() > INITIAL_RECEIVE_CAPACITY
            && self.receive_buffer.len() <= INITIAL_RECEIVE_CAPACITY
        {
            self.receive_buffer.shrink_to(INITIAL_RECEIVE_CAPACITY);
        }
        Ok(accepted)
    }

    /// Передаёт очереди готовое системное либо сетевое сообщение.
    pub(crate) fn push_message(&self, message: CMessage) {
        self.received_messages.push(message);
    }

    /// Атомарно забирает все сообщения в исходном FIFO-порядке.
    pub(crate) fn take_all_messages(&self) -> VecDeque<CMessage> {
        self.received_messages.take_all()
    }

    /// Возвращает число байт неполного хвоста, ожидающих продолжения TCP-потока.
    pub(crate) fn pending_bytes(&self) -> usize {
        self.receive_buffer.len()
    }

    fn discard_pending<T>(&mut self, error: ReceiveError) -> Result<T, ReceiveError> {
        self.receive_buffer.clear();
        Err(error)
    }
}

impl Default for CMyNetClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageSender for CMyNetClient {
    fn send_to_server(&self, buffer: &[u8], prioritized: bool, flags: i32) -> i32 {
        self.send_queue.send_to_server(buffer, prioritized, flags)
    }
}
