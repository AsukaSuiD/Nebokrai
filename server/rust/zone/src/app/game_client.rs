//! Достигнутый outgoing/receive и Linux I/O owner GameServer `CMyNetClient`
//! из `nets/netserver/mynetclient.cpp/.h`, перенесённый в Zone app — исходящий
//! край Game к World и Billing. Источник контракта — та же точная пара,
//! что у [`crate::app::game_message`].
//!
//! Машинно подтверждённые точки (дизассемблер этого прохода,
//! `.exe/gameserver.exe`):
//! - ctor `CMyNetClient` `0x41A4B0`: базовый `CClient` ctor, task type zero
//!   (исходный `ST_UNKNOWNSERVER == 0`), destructor `0x41A530` и
//!   `GetSocketCommand` `0x41A520` остаются теми же свидетельствами;
//! - `OnReceive` `0x41A6A0` имеет статус `IMPLEMENTED, VERIFIED_DISASSEMBLY`:
//!   один вызов читает не более `0x2800`; `WSAEWOULDBLOCK 0x2733` и `recv == 0`
//!   — только журнал; цикл frames, пока накоплено `>= 0xC`: CRC длины через
//!   общий `DataCrc32 0x47B0A0`, `declared > size` — останов без потери хвоста,
//!   create только несжатым `0x413850` (upstream без RLE), повторный CRC,
//!   publish в FIFO `+0x100` через `0x4126D0`; shrink к `0x100000`;
//! - `HandleClose` `0x41A590` (virtual слот 19 диспетчера close): по server
//!   type `[client+0x1F0]` публикует в ту же FIFO `+0x100` `CMessage(0x6F901)`
//!   (World) либо `CMessage(0x6F903)` (Billing) через ctor `CMessage 0x4136D0`;
//!   ветка unknown type ставила stack-local value без доказанного смысла —
//!   эта одна UB-граница не получает Rust-значения и остаётся typed
//!   `UnknownServerTypeCloseReaction`.
//!
//! Общий `CClient` receive capacity `0x100000`, ручные realloc/memmove,
//! `CSocketCommands::Clear`, destructor и deleting thunk заменены
//! `Vec`/`CMsgQueue`/RAII. Component хранит owned `TcpStream`, resolved endpoint,
//! control-send и один awaitable read/send шаг поверх общего `CClient`.
//! EOF и подтверждённая системная ошибка TCP проходят один и тот же
//! `OnClose -> HandleClose`: socket закрывается до публикации synthetic
//! World/Billing сообщения, которое живой `CGame` обрабатывает в своём FIFO.
//! `SetSendRevBuf` RVA `0x0001A650` передавал Windows `SO_SNDBUF=0`; Linux
//! backpressure-эквивалент не доказан, поэтому эта socket-option остаётся
//! локальным `BLOCKED_MISSING_FACT`, а не получает фиктивный вызов.
//! Завершение GameServer дописывает исходящую очередь до закрытия сокета:
//! CClient::Close (0x004191b0) ждёт ExitSocketThread, чей send-loop
//! (0x0041a171..0x0041a19d) проверяет очередь и живое соединение.
//! Ожидание writable выполняет Tokio, частичный хвост хранит ClientSendQueue.
//! Успех означает только передачу байтов TCP, не подтверждение World/БД.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;
use std::net::SocketAddrV4;

use tokio::net::{TcpSocket, TcpStream};

use nebokrai_shared::network::{
    connect as connect_client, read_client_tcp_chunk, CMsgQueue, ClientConnectError,
    ClientSendError, ClientSendQueue, FlushOutcome, INITIAL_RECEIVE_CAPACITY,
};
use nebokrai_shared::protocol::data_crc32;

use super::game_message::{CMessage, CreateMessageError};

const SERVER_ENVELOPE_LEN: usize = 12;
const MIN_SERVER_FRAME_LEN: usize = SERVER_ENVELOPE_LEN + 16;
const WORLD_DISCONNECTED: i32 = 0x0006_F901;
const BILLING_DISCONNECTED: i32 = 0x0006_F903;

/// Причина остановки World/Billing receive accumulator.
#[derive(Debug, Eq, PartialEq)]
pub enum ReceiveError {
    LengthChecksumMismatch {
        expected: u32,
        actual: u32,
    },
    SignedFrameLengthReactionUnknown {
        declared: u32,
    },
    ShortFrameReactionUnknown {
        declared: u32,
    },
    Message(CreateMessageError),
    ContentChecksumMismatch {
        message_type: i32,
        expected: u32,
        actual: u32,
    },
    PendingSizeOverflowReactionUnknown,
}

/// Результат одного Linux read/send ожидания World/Billing направления.
#[derive(Debug, Eq, PartialEq)]
pub enum GameClientIoStep {
    Received { messages: usize },
    Sent(FlushOutcome),
    Closed,
}

/// Ошибка одного Linux I/O шага Game `CMyNetClient`.
#[derive(Debug)]
pub enum GameClientIoError {
    NotConnected,
    Io(io::Error),
    Receive(ReceiveError),
    Send(ClientSendError),
    UnknownServerTypeClose,
}

impl fmt::Display for GameClientIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => formatter.write_str("GameServer upstream не подключён"),
            Self::Io(error) => write!(formatter, "ошибка I/O Game upstream: {error}"),
            Self::Receive(_) => formatter.write_str("upstream передал некорректный Game frame"),
            Self::Send(error) => write!(formatter, "ошибка отправки Game upstream: {error}"),
            Self::UnknownServerTypeClose => {
                formatter.write_str("transport закрыл CMyNetClient с неизвестным server type")
            }
        }
    }
}

impl Error for GameClientIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Send(error) => Some(error),
            Self::NotConnected | Self::Receive(_) | Self::UnknownServerTypeClose => None,
        }
    }
}

/// BLOCKED_MISSING_FACT для единственной unknown-type ветви `HandleClose`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnknownServerTypeCloseReaction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ServerType {
    Unknown = 0,
    World = 1,
    Billing = 2,
}

pub struct CMyNetClient {
    connection: Option<TcpStream>,
    connected_endpoint: Option<SocketAddrV4>,
    control_send: bool,
    server_type: ServerType,
    send_queue: ClientSendQueue,
    received_messages: CMsgQueue<CMessage>,
    receive_buffer: Vec<u8>,
}

impl CMyNetClient {
    /// Сохраняет exact constructor defaults до выбора направления в `CGame`.
    pub fn new() -> Self {
        Self {
            connection: None,
            connected_endpoint: None,
            control_send: false,
            server_type: ServerType::Unknown,
            send_queue: ClientSendQueue::new(),
            received_messages: CMsgQueue::new(),
            receive_buffer: Vec::with_capacity(INITIAL_RECEIVE_CAPACITY),
        }
    }

    pub const fn set_server_type(&mut self, server_type: ServerType) {
        self.server_type = server_type;
    }

    pub const fn server_type(&self) -> ServerType {
        self.server_type
    }

    /// Возвращает ту же per-client очередь, что virtual `GetSocketCommand`.
    pub const fn send_queue(&self) -> &ClientSendQueue {
        &self.send_queue
    }

    /// Подключает уже bind-нутый IPv4 socket к выбранному upstream.
    pub async fn connect(
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
    pub fn close(&mut self) -> i32 {
        self.connection = None;
        1
    }

    /// Воспроизводит `OnClose -> HandleClose` текущего направления.
    pub fn handle_transport_close(&mut self) -> Result<(), UnknownServerTypeCloseReaction> {
        self.connection = None;
        self.handle_close()
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub const fn connected_endpoint(&self) -> Option<SocketAddrV4> {
        self.connected_endpoint
    }

    pub fn enable_control_send(&mut self) {
        self.control_send = true;
    }

    pub fn disable_control_send(&mut self) {
        self.control_send = false;
    }

    /// Отправляет один owned snapshot только при живом socket и control-send.
    pub fn try_flush_outgoing(&self) -> Option<Result<FlushOutcome, ClientSendError>> {
        if !self.control_send {
            return None;
        }
        self.connection
            .as_ref()
            .map(|stream| self.send_queue.try_flush(stream))
    }

    pub async fn flush_outgoing_before_close(
        &self,
    ) -> Option<Result<FlushOutcome, ClientSendError>> {
        if !self.control_send {
            return None;
        }
        let stream = self.connection.as_ref()?;
        let mut total_sent = 0u64;
        while self.send_queue.pending() > 0 {
            match self.send_queue.try_flush(stream) {
                Ok(FlushOutcome::Drained { bytes_sent }) => {
                    total_sent = total_sent.wrapping_add(bytes_sent);
                }
                Ok(FlushOutcome::WouldBlock { bytes_sent }) => {
                    total_sent = total_sent.wrapping_add(bytes_sent);
                    if let Err(source) = stream.writable().await {
                        return Some(Err(ClientSendError::Io {
                            source,
                            bytes_sent: total_sent,
                        }));
                    }
                }
                Err(error) => return Some(Err(error)),
            }
        }
        Some(Ok(FlushOutcome::Drained {
            bytes_sent: total_sent,
        }))
    }

    /// Ждёт один read либо send-readiness и выполняет один transport-шаг.
    pub async fn run_io_once(&mut self) -> Result<GameClientIoStep, GameClientIoError> {
        enum Ready {
            Read(io::Result<Vec<u8>>),
            Write(io::Result<()>),
        }

        let ready = {
            let stream = self
                .connection
                .as_ref()
                .ok_or(GameClientIoError::NotConnected)?;
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
                self.handle_transport_close()
                    .map_err(|_| GameClientIoError::UnknownServerTypeClose)?;
                Ok(GameClientIoStep::Closed)
            }
            Ready::Read(Ok(received)) => self
                .accept_received_bytes(&received)
                .map(|messages| GameClientIoStep::Received { messages })
                .map_err(GameClientIoError::Receive),
            Ready::Read(Err(error)) | Ready::Write(Err(error)) => {
                self.handle_transport_close()
                    .map_err(|_| GameClientIoError::UnknownServerTypeClose)?;
                Err(GameClientIoError::Io(error))
            }
            Ready::Write(Ok(())) => {
                let stream = self
                    .connection
                    .as_ref()
                    .ok_or(GameClientIoError::NotConnected)?;
                match self.send_queue.try_flush(stream) {
                    Ok(outcome) => Ok(GameClientIoStep::Sent(outcome)),
                    Err(
                        error @ (ClientSendError::WriteZero { .. } | ClientSendError::Io { .. }),
                    ) => {
                        self.handle_transport_close()
                            .map_err(|_| GameClientIoError::UnknownServerTypeClose)?;
                        Err(GameClientIoError::Send(error))
                    }
                    Err(error @ ClientSendError::UnsupportedFlags { .. }) => {
                        Err(GameClientIoError::Send(error))
                    }
                }
            }
        }
    }

    /// Публикует synthetic close event доказанного направления.
    pub fn handle_close(&self) -> Result<(), UnknownServerTypeCloseReaction> {
        let message_type = match self.server_type {
            ServerType::World => WORLD_DISCONNECTED,
            ServerType::Billing => BILLING_DISCONNECTED,
            // BLOCKED_MISSING_FACT, VERIFIED_DISASSEMBLY: Game RVA 0x1A625
            // загружает неинициализированный local pointer и передаёт его
            // `CMsgQueue::PushMessage`. Safe Rust не назначает payload.
            ServerType::Unknown => return Err(UnknownServerTypeCloseReaction),
        };
        self.received_messages.push(CMessage::new(message_type));
        Ok(())
    }

    /// Добавляет TCP-фрагмент и разбирает все полные World/Billing frames.
    pub fn accept_received_bytes(&mut self, received: &[u8]) -> Result<usize, ReceiveError> {
        let pending_size = self
            .receive_buffer
            .len()
            .checked_add(received.len())
            .filter(|size| *size <= i32::MAX as usize);
        let Some(pending_size) = pending_size else {
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
                .expect("наличие полного Game server envelope уже проверено");
            let declared = u32::from_le_bytes(declared_bytes);
            let expected_length_crc = u32::from_le_bytes(
                frame[4..8]
                    .try_into()
                    .expect("наличие полного Game server envelope уже проверено"),
            );
            let actual_length_crc = data_crc32(&declared_bytes);
            if actual_length_crc != expected_length_crc {
                return self.discard_pending(ReceiveError::LengthChecksumMismatch {
                    expected: expected_length_crc,
                    actual: actual_length_crc,
                });
            }

            if (declared as i32) < 0 {
                // BLOCKED_MISSING_FACT: signed compare пропускает sign-bit к
                // последующему unsigned `total_len - 12`.
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
                return self.discard_pending(ReceiveError::ShortFrameReactionUnknown { declared });
            }

            let expected_content_crc = u32::from_le_bytes(
                frame[8..12]
                    .try_into()
                    .expect("наличие полного Game server envelope уже проверено"),
            );
            let message_wire = &frame[SERVER_ENVELOPE_LEN..frame_len];
            let message = match CMessage::create_without_rle(message_wire) {
                Ok(message) => message,
                Err(error) => return self.discard_pending(ReceiveError::Message(error)),
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

    pub fn take_all_messages(&self) -> VecDeque<CMessage> {
        self.received_messages.take_all()
    }

    pub fn pending_bytes(&self) -> usize {
        self.receive_buffer.len()
    }

    fn discard_pending<T>(&mut self, error: ReceiveError) -> Result<T, ReceiveError> {
        self.receive_buffer.clear();
        Err(error)
    }
}
