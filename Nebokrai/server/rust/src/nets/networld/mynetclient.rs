//! Исходящее LoginServer-направление WorldServer из
//! `nets/networld/mynetclient.cpp` и `.h`.
//!
//! Статус владельца: `IMPLEMENTED` для constructor/destructor ownership,
//! явного close, transport-close события, 12-байтового CRC receive-
//! accumulator, собственной FIFO и одного Linux read/send шага.
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные пути PDB:
//! `e:\svn\fengyun_russia_dev\nets\networld\mynetclient.cpp` и `.h`.
//! Существенные RVA: конструктор `0x00029B70`, деструктор `0x00029B90`,
//! `HandleClose` `0x00029BA0`, `OnReceive` `0x00029C30`.
//!
//! Конструктор не добавлял инициализации поверх общего `CClient`; найденный
//! World call site выделял полный объект размером `0x1F0`, а очередь входящих
//! сообщений достигалась по offset `0x100`. Rust выражает наследование
//! композицией: этот owner хранит конкретные World FIFO и accumulator, а
//! connect/send использует уже восстановленные операции `nets/clients.rs`.
//!
//! Успешный connect делает направление подключённым. Явный `CClient::Close`
//! только закрывал общий client и не создавал сообщения. Общий transport
//! `OnClose` сначала сбрасывал connect flag, затем вызывал виртуальный
//! `HandleClose`, который публиковал пустое World-сообщение `0x3FC01`.
//! Эти два пути сохранены раздельно в `close` и `handle_transport_close`.
//!
//! Живой receive-контракт — поток envelope
//! `[total_len, crc(total_len), crc(message), message]`. После накопления как
//! минимум 12 bytes сначала проверяется IEEE CRC little-endian длины. Полный
//! frame превращается в `networld::CMessage`, после чего content CRC считается
//! по уже нормализованному внутреннему сообщению. Только прошедший обе проверки
//! объект попадает в ту же FIFO, что и transport-close событие. За один вызов
//! разбираются все полные frames, неполный хвост сохраняется, а доказанная
//! ошибка очищает весь ещё не разобранный вход без отката уже опубликованных
//! сообщений.
//!
//! Общий `CClient` начинал с receive capacity `0x100000`, читал не более
//! `0x2800` bytes за один `recv`, при необходимости заранее расширял storage,
//! переносил хвост через `memmove` и сжимал capacity обратно. `Vec<u8>`
//! сохраняет те же bytes и границы frames без allocator-копий и немедленного
//! выделения памяти по одной недоверенной длине; `drain` заменяет `memmove`.
//! Один awaitable read/send шаг заменяет два Windows socket thread без нового
//! взаимного порядка. `CMsgQueue<CMessage>` заменяет deque указателей и ручное
//! виртуальное удаление.
//!
//! Для длины с sign bit, `total_len < 12`, внутреннего сообщения короче
//! 16 bytes и signed-переполнения `m_nSize` исходный код допускает unsigned
//! underflow либо чтение вне frame. Наблюдаемая реакция не доказана, поэтому
//! safe Rust локально возвращает отдельные ошибки и не выдаёт это решение за
//! поведение оригинального процесса. `total_len == 12` отличается: нулевая
//! длина внутреннего сообщения доказанно даёт null create и очищает вход.
//!
//! Чужой `CMyNetServer` deleting-destructor thunk, SEH allocation unwind,
//! vtable, ручные buffer reallocations и exception plumbing классифицированы
//! как compiler/allocator noise. `InitNetClient` теперь связывает этот owner с
//! setup, bind/connect и регистрацией у `worldserver/game.rs`.
//! `ReConnectLoginServer` создаёт такой же owner отдельно и передаёт его через
//! typed event FIFO `CMyNetServer`, заменяющий внутрипроцессный
//! `0x3FC03 + pointer` без integer-pointer. Control-send остаётся выключенным
//! до фактической замены в World `OnServerMessage` RVA `0x000ADCF0`, opcode
//! `0x3FC03`.

use std::error::Error;
use std::fmt;
use std::io;
use std::net::SocketAddrV4;

use tokio::net::{TcpSocket, TcpStream};

use crate::nets::clients::{
    ClientConnectError, ClientSendError, ClientSendQueue, FlushOutcome, INITIAL_RECEIVE_CAPACITY,
    connect as connect_client,
};
use crate::nets::msgqueue::CMsgQueue;
use crate::public::crc32static::data_crc32;
use crate::transport::read_client_tcp_chunk;

use super::message::{CMessage, CreateMessageError};

const SERVER_ENVELOPE_LEN: usize = 12;
const INNER_MESSAGE_HEADER_LEN: usize = 16;
const MIN_SERVER_FRAME_LEN: usize = SERVER_ENVELOPE_LEN + INNER_MESSAGE_HEADER_LEN;
const CLOSE_MESSAGE_TYPE: i32 = 0x0003_FC01;

/// Причина, по которой текущий неразобранный LoginServer-вход был отброшен.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ReceiveError {
    /// CRC little-endian слова `total_len` не совпал со вторым словом envelope.
    LengthChecksumMismatch { expected: u32, actual: u32 },
    /// Длина меньше полного frame либо не представима положительным long.
    InvalidFrameLength { declared: u32 },
    /// Создание нормализованного World-сообщения завершилось ошибкой.
    Message(CreateMessageError),
    /// CRC нормализованного сообщения не совпал с третьим словом envelope.
    ContentChecksumMismatch {
        message_type: i32,
        expected: u32,
        actual: u32,
    },
    /// Размер накопителя вышел за signed 32-битную границу исходного `m_nSize`.
    PendingSizeOutsideLegacyRange,
}

/// Результат одного Linux read/send ожидания World-to-Login клиента.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldClientIoStep {
    /// Принят один TCP block и опубликовано указанное число полных сообщений.
    Received { messages: usize },
    /// Выполнена попытка отправки текущего атомарного снимка очереди.
    Sent(FlushOutcome),
    /// EOF преобразован в доказанный `OnClose -> 0x3FC01`.
    Closed,
}

/// Ошибка одного Linux read/send ожидания World-to-Login клиента.
#[derive(Debug)]
pub(crate) enum WorldClientIoError {
    /// Вызов сделан без успешного LoginServer-соединения.
    NotConnected,
    /// Linux transport вернул ошибку чтения либо readiness.
    Io(io::Error),
    /// Component framing отклонил текущий накопленный вход.
    Receive(ReceiveError),
    /// Общий `CClient` не смог отправить текущий owned snapshot.
    Send(ClientSendError),
}

impl fmt::Display for WorldClientIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => formatter.write_str("WorldServer не подключён к LoginServer"),
            Self::Io(error) => write!(formatter, "ошибка I/O LoginServer-соединения: {error}"),
            Self::Receive(_) => formatter.write_str("LoginServer передал некорректный World frame"),
            Self::Send(error) => {
                write!(formatter, "ошибка отправки LoginServer-соединения: {error}")
            }
        }
    }
}

impl Error for WorldClientIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Send(error) => Some(error),
            Self::NotConnected | Self::Receive(_) => None,
        }
    }
}

/// Производный World-клиент с общей send-очередью и собственной World FIFO.
pub(crate) struct CMyNetClient {
    connection: Option<TcpStream>,
    connected_endpoint: Option<SocketAddrV4>,
    control_send: bool,
    send_queue: ClientSendQueue,
    receive_buffer: Vec<u8>,
    received_messages: CMsgQueue<CMessage>,
}

impl CMyNetClient {
    /// Создаёт отключённое направление с исходной receive capacity.
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

    /// Подключает ранее созданный и bind-нутый IPv4 socket к LoginServer.
    ///
    /// Десятисекундный предел и transport error принадлежат общему `CClient`.
    /// Состояние меняется только после успешного connect.
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

    /// Воспроизводит явный `CClient::Close` без synthetic close-сообщения.
    pub(crate) fn close(&mut self) -> i32 {
        self.connection = None;
        1
    }

    /// Воспроизводит transport `OnClose -> HandleClose -> 0x3FC01`.
    pub(crate) fn handle_transport_close(&mut self) {
        self.connection = None;
        self.push_message(CMessage::new(CLOSE_MESSAGE_TYPE));
    }

    /// Сообщает состояние исходного `m_bConnect`.
    pub(crate) fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    /// Возвращает resolved IPv4 endpoint последнего успешного connect.
    pub(crate) const fn connected_endpoint(&self) -> Option<SocketAddrV4> {
        self.connected_endpoint
    }

    /// Включает исходный `m_bControlSend` в позиции будущего `CGame` init.
    pub(crate) fn enable_control_send(&mut self) {
        self.control_send = true;
    }

    /// Возвращает очередь, в которую `CMessage::Send` копирует envelope.
    pub(crate) fn send_queue(&self) -> &ClientSendQueue {
        &self.send_queue
    }

    /// Пытается отправить текущий снимок LoginServer send-очереди.
    ///
    /// `None` означает выключленный control-send либо отсутствие socket;
    /// команды в обоих случаях остаются во владеющей очереди.
    pub(crate) fn try_flush_outgoing(&self) -> Option<Result<FlushOutcome, ClientSendError>> {
        if !self.control_send {
            return None;
        }
        self.connection
            .as_ref()
            .map(|stream| self.send_queue.try_flush(stream))
    }

    /// Ждёт один read либо доступность отправки и выполняет ровно один шаг.
    ///
    /// Read и send обслуживались разными Windows threads без доказанного
    /// взаимного порядка. `tokio::select!` сохраняет отсутствие такого порядка,
    /// не создавая busy-loop `Sleep(1)`. Ошибка I/O сама не назначает close-
    /// политику; доказанный EOF выполняет component `OnClose` немедленно.
    pub(crate) async fn run_io_once(
        &mut self,
        mut recv_time_ms: impl FnMut() -> u32,
    ) -> Result<WorldClientIoStep, WorldClientIoError> {
        enum Ready {
            Read(io::Result<Vec<u8>>),
            Write(io::Result<()>),
        }

        let ready = {
            let stream = self
                .connection
                .as_ref()
                .ok_or(WorldClientIoError::NotConnected)?;
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
                Ok(WorldClientIoStep::Closed)
            }
            Ready::Read(Ok(received)) => self
                .accept_received_bytes(&received, &mut recv_time_ms)
                .map(|messages| WorldClientIoStep::Received { messages })
                .map_err(WorldClientIoError::Receive),
            Ready::Read(Err(error)) | Ready::Write(Err(error)) => {
                Err(WorldClientIoError::Io(error))
            }
            Ready::Write(Ok(())) => {
                let stream = self
                    .connection
                    .as_ref()
                    .ok_or(WorldClientIoError::NotConnected)?;
                self.send_queue
                    .try_flush(stream)
                    .map(WorldClientIoStep::Sent)
                    .map_err(WorldClientIoError::Send)
            }
        }
    }

    /// Добавляет TCP-фрагмент и разбирает все достигнутые полные frames.
    ///
    /// `recv_time_ms` вызывается отдельно для каждого успешно созданного
    /// сообщения в позиции старого `CMessage::CreateMessageWithoutRLE`.
    /// При ошибке весь оставшийся вход отбрасывается, а ранее опубликованные
    /// сообщения сохраняются.
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
            return self.discard_pending(ReceiveError::PendingSizeOutsideLegacyRange);
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

            if (declared as i32) < 0 {
                return self.discard_pending(ReceiveError::InvalidFrameLength { declared });
            }

            let frame_len = declared as usize;
            if frame.len() < frame_len {
                break;
            }
            if frame_len == SERVER_ENVELOPE_LEN {
                return self.discard_pending(ReceiveError::Message(CreateMessageError::EmptyInput));
            }
            if frame_len < MIN_SERVER_FRAME_LEN {
                return self.discard_pending(ReceiveError::InvalidFrameLength { declared });
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

    /// Возвращает число сообщений в начале текущего доменного прохода.
    pub(crate) fn pending_messages(&self) -> i32 {
        self.received_messages.get_size()
    }

    /// Передаёт старейшее LoginServer-сообщение владельцу `CGame`.
    pub(crate) fn pop_received_message(&self) -> Option<CMessage> {
        self.received_messages.pop()
    }

    /// Возвращает число bytes неполного хвоста входного TCP-потока.
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
