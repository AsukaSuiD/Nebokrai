//! Достигнутый outgoing/receive и Linux I/O owner GameServer `CMyNetClient`.
//!
//! Constructor `0x0001A4B0`, `GetSocketCommand` `0x0001A520` и destructor
//! `0x0001A530` имеют статус `IMPLEMENTED`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходники
//! `nets/netserver/mynetclient.cpp/.h`. Constructor сначала строил общий
//! `CClient`, задавал `ST_UNKNOWNSERVER == 0`, строил собственный
//! `CSocketCommands` и немедленно очищал его. `CGame::InitNetClientOfWS/BS`
//! затем назначали соответственно `ST_WORLDSERVER == 1` и
//! `ST_BILLINGSERVER == 2`.
//!
//! Для `CMessage::Send/SendToBS` общий `ClientSendQueue` сохраняет немедленную
//! owned-копию, priority и flags. `OnReceive` RVA `0x0001A6A0` имеет статус
//! `IMPLEMENTED`: он разбирает exact
//! `[total_len, crc(total_len), crc(normalized_message), message]`, проверяет
//! content CRC после `CreateMessageWithoutRLE` и публикует все полные сообщения
//! в собственную FIFO с сохранением TCP-хвоста. `HandleClose` RVA `0x0001A590`
//! публикует отдельные World/Billing close messages; машинный код подтвердил,
//! что unknown type ставил в очередь неинициализированный stack pointer, поэтому
//! эта одна UB-граница не получает Rust-значения.
//!
//! Общий `CClient` receive capacity `0x100000`, ручные realloc/memmove,
//! `CSocketCommands::Clear`, destructor и deleting thunk заменены
//! `Vec`/`CMsgQueue`/RAII. Component хранит owned `TcpStream`, resolved endpoint,
//! control-send и один awaitable read/send шаг поверх общего `CClient`.
//! `SetSendRevBuf` RVA `0x0001A650` передавал Windows `SO_SNDBUF=0`; Linux
//! backpressure-эквивалент не доказан, поэтому эта socket-option остаётся
//! локальным `BLOCKED_MISSING_FACT`, а не получает фиктивный вызов.

use std::collections::VecDeque;
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
const MIN_SERVER_FRAME_LEN: usize = SERVER_ENVELOPE_LEN + 16;
const WORLD_DISCONNECTED: i32 = 0x0006_F901;
const BILLING_DISCONNECTED: i32 = 0x0006_F903;

/// Причина остановки World/Billing receive accumulator.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ReceiveError {
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
pub(crate) enum GameClientIoStep {
    Received { messages: usize },
    Sent(FlushOutcome),
    Closed,
}

/// Ошибка одного Linux I/O шага Game `CMyNetClient`.
#[derive(Debug)]
pub(crate) enum GameClientIoError {
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
pub(crate) struct UnknownServerTypeCloseReaction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum ServerType {
    Unknown = 0,
    World = 1,
    Billing = 2,
}

pub(crate) struct CMyNetClient {
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
    pub(crate) fn new() -> Self {
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

    pub(crate) const fn set_server_type(&mut self, server_type: ServerType) {
        self.server_type = server_type;
    }

    pub(crate) const fn server_type(&self) -> ServerType {
        self.server_type
    }

    /// Возвращает ту же per-client очередь, что virtual `GetSocketCommand`.
    pub(crate) const fn send_queue(&self) -> &ClientSendQueue {
        &self.send_queue
    }

    /// Подключает уже bind-нутый IPv4 socket к выбранному upstream.
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

    /// Воспроизводит `OnClose -> HandleClose` текущего направления.
    pub(crate) fn handle_transport_close(&mut self) -> Result<(), UnknownServerTypeCloseReaction> {
        self.connection = None;
        self.handle_close()
    }

    pub(crate) fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub(crate) const fn connected_endpoint(&self) -> Option<SocketAddrV4> {
        self.connected_endpoint
    }

    pub(crate) fn enable_control_send(&mut self) {
        self.control_send = true;
    }

    pub(crate) fn disable_control_send(&mut self) {
        self.control_send = false;
    }

    /// Отправляет один owned snapshot только при живом socket и control-send.
    pub(crate) fn try_flush_outgoing(&self) -> Option<Result<FlushOutcome, ClientSendError>> {
        if !self.control_send {
            return None;
        }
        self.connection
            .as_ref()
            .map(|stream| self.send_queue.try_flush(stream))
    }

    /// Ждёт один read либо send-readiness и выполняет один transport-шаг.
    pub(crate) async fn run_io_once(&mut self) -> Result<GameClientIoStep, GameClientIoError> {
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
            Ready::Read(Err(error)) | Ready::Write(Err(error)) => Err(GameClientIoError::Io(error)),
            Ready::Write(Ok(())) => {
                let stream = self
                    .connection
                    .as_ref()
                    .ok_or(GameClientIoError::NotConnected)?;
                self.send_queue
                    .try_flush(stream)
                    .map(GameClientIoStep::Sent)
                    .map_err(GameClientIoError::Send)
            }
        }
    }

    /// Публикует synthetic close event доказанного направления.
    pub(crate) fn handle_close(&self) -> Result<(), UnknownServerTypeCloseReaction> {
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
    pub(crate) fn accept_received_bytes(&mut self, received: &[u8]) -> Result<usize, ReceiveError> {
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

    pub(crate) fn take_all_messages(&self) -> VecDeque<CMessage> {
        self.received_messages.take_all()
    }

    pub(crate) fn pending_bytes(&self) -> usize {
        self.receive_buffer.len()
    }

    fn discard_pending<T>(&mut self, error: ReceiveError) -> Result<T, ReceiveError> {
        self.receive_buffer.clear();
        Err(error)
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\nets\netserver\mynetclient.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\nets\netserver\mynetclient.h

// IMPLEMENTED: constructor, `GetSocketCommand` и destructor материализованы
// выше; чужой deleting thunk и allocator/SEH cleanup выражены RAII.

// IMPLEMENTED: `HandleClose` материализован выше с изолированной unknown-type
// UB-границей.

// ============================================================================
// FUNCTION: CMyNetClient::SetSendRevBuf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\nets\netserver\mynetclient.cpp:232
// RVA: 0x0001A650
// ADDRESS: 0041a650
// PROTOTYPE: void __thiscall SetSendRevBuf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `OnReceive` материализован выше; покрытые allocation,
// accumulator и memmove выражены владеющим `Vec`.

// CLASSIFIED_TECHNICAL_NOISE: placement delete и constructor unwind не
// имеют самостоятельного эффекта поверх Rust ownership.

// COMPONENT_VARIANT_END: GameServer
