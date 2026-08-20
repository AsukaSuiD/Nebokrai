//! Исходящее AuthServer-направление LoginServer из
//! `nets/netlogin/mynetclient_auth.cpp` и `.h`.
//!
//! Статус владельца: `IMPLEMENTED` для композиции общего `CClient`, успешного
//! connect/явного close, transport-close события, 12-байтового CRC receive-
//! accumulator, собственной FIFO и одного Linux read/send шага.
//!
//! Точная пара: `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`;
//! SHA-256 EXE
//! `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876`,
//! SHA-256 PDB
//! `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`.
//! Исходные пути PDB:
//! `d:\complite_version\fengyun_russia\trunk\nets\netlogin\mynetclient_auth.cpp`
//! и `.h`.
//! Существенные RVA: конструктор `0x0006CAC0`, деструктор `0x0006CAE0`,
//! `HandleClose` `0x0006CAF0`, `OnReceive` `0x0006CB90`.
//!
//! Конструктор не добавлял состояния поверх `CClient`: receive-buffer
//! `0x100000`, очередь входящих сообщений и очередь owned send-команд являются
//! его доказанными базовыми частями. Rust выражает наследование композицией:
//! этот owner хранит конкретную Login FIFO и accumulator, а connect и send
//! использует уже восстановленные операции `nets/clients.rs`.
//!
//! Успешный connect делает направление подключённым. Явный `Close` только
//! закрывал общий client и сбрасывал connect-флаг; synthetic сообщения он не
//! создавал. Напротив, общий `CClient::OnClose` сначала сбрасывал флаг, затем
//! вызывал виртуальный `HandleClose`, который публиковал пустое Login-сообщение
//! `0xCF301`. Эти два пути сохранены раздельно в `close` и
//! `handle_transport_close`.
//!
//! Живой receive-контракт — поток envelope
//! `[total_len, crc(total_len), crc(message), message]`. После накопления как
//! минимум 12 bytes проверяется IEEE CRC little-endian длины. Полный кадр
//! превращается в `netlogin::CMessage`, после чего второй CRC считается по уже
//! нормализованному внутреннему сообщению. Только прошедший обе проверки объект
//! попадает в ту же FIFO, что и transport-close событие. Все полные кадры
//! разбираются по порядку, неполный хвост сохраняется, а доказанная ошибка
//! очищает весь ещё не разобранный вход без отката ранее опубликованных кадров.
//!
//! Исходный allocator расширял buffer до объявленной длины, переносил хвост
//! через `memmove` и сжимал capacity обратно до `0x100000`. `Vec<u8>` сохраняет
//! bytes и границы кадров без немедленного выделения по одной недоверенной
//! длине; `drain` заменяет `memmove`. `CMsgQueue<CMessage>` заменяет deque
//! указателей и виртуальное ручное удаление. Vtable, SEH, security-cookie,
//! deleting-destructor и `$L71362` удалены как compiler/allocator noise.
//!
//! Реакция оригинала на длину с sign bit, `total_len < 12`, внутреннее
//! сообщение короче 16 bytes и переполнение signed `m_nSize` не доказана.
//! Безопасный Rust локально отвергает эти границы отдельными ошибками, не
//! выдавая выбранную реакцию за контракт исходного процесса. Неизвестный Auth
//! opcode `0x10F101` относится к будущему доменному обработчику, а не к этому
//! framing-owner.
//!
//! Успешный `ReconnectAS` передавал новый `CMyNetClientAuth*` через synthetic
//! `0xCF302` в очереди старого клиента. Указатель был только внутрипроцессным
//! механизмом передачи владения и не выходил в wire. Rust заменяет пару
//! `CMessage + 32-bit pointer` типизированным `AuthClientEvent::Reconnected` в
//! той же FIFO; порядок относительно уже принятых сообщений сохраняется без
//! integer-pointer и `unsafe`.

use std::error::Error;
use std::fmt;
use std::io;
use std::net::SocketAddrV4;
use std::sync::Arc;

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
const CLOSE_MESSAGE_TYPE: i32 = 0x000C_F301;

/// Причина, по которой текущий неразобранный Auth-вход был отброшен.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ReceiveError {
    /// CRC little-endian слова `total_len` не совпал со вторым словом envelope.
    LengthChecksumMismatch { expected: u32, actual: u32 },
    /// Signed-ветвление оригинала не задаёт безопасную реакцию на этот размер.
    SignedFrameLengthReactionUnknown { declared: u32 },
    /// Короткий frame ведёт в underflow либо чтение неполного message header.
    ShortFrameReactionUnknown { declared: u32 },
    /// Создание нормализованного Login-сообщения завершилось ошибкой.
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

/// Одно событие исходной FIFO Auth-клиента LoginServer.
pub(crate) enum AuthClientEvent {
    /// Обычное wire-сообщение либо synthetic transport-close `0xCF301`.
    Message(CMessage),
    /// Типизированная замена внутрипроцессного `0xCF302 + CClient*`.
    Reconnected(CMyNetClientAuth),
}

/// Клонируемая граница reconnect-задачи к FIFO прежнего Auth-клиента.
///
/// Она заменяет только внутрипроцессную доставку `0xCF302 + pointer`; обычный
/// wire и владение самим подключённым клиентом остаются неизменными.
#[derive(Clone)]
pub(crate) struct AuthClientEventPublisher {
    events: Arc<CMsgQueue<AuthClientEvent>>,
}

impl AuthClientEventPublisher {
    /// Публикует synthetic сообщение в обычной позиции Auth FIFO.
    pub(crate) fn publish_message(&self, message: CMessage) {
        self.events.push(AuthClientEvent::Message(message));
    }

    /// Передаёт FIFO владение успешно подключённым replacement-клиентом.
    pub(crate) fn publish_reconnected(&self, client: CMyNetClientAuth) {
        self.events.push(AuthClientEvent::Reconnected(client));
    }
}

/// Результат одного Linux read/send ожидания исходящего Auth-клиента.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AuthClientIoStep {
    /// Принят один TCP block и опубликовано указанное число полных сообщений.
    Received { messages: usize },
    /// Выполнена попытка отправки текущего атомарного снимка очереди.
    Sent(FlushOutcome),
    /// EOF преобразован в доказанный `OnClose -> 0xCF301`.
    Closed,
}

/// Ошибка одного Linux read/send ожидания Auth-клиента.
#[derive(Debug)]
pub(crate) enum AuthClientIoError {
    /// Вызов сделан без успешного Auth-соединения.
    NotConnected,
    /// Linux transport вернул ошибку чтения либо readiness.
    Io(io::Error),
    /// Component framing отклонил текущий накопленный вход.
    Receive(ReceiveError),
    /// Общий `CClient` не смог отправить текущий owned snapshot.
    Send(ClientSendError),
}

impl fmt::Display for AuthClientIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => formatter.write_str("Auth-клиент LoginServer не подключён"),
            Self::Io(error) => write!(formatter, "ошибка I/O Auth-соединения: {error}"),
            Self::Receive(_) => formatter.write_str("Auth-соединение передало некорректный frame"),
            Self::Send(error) => write!(formatter, "ошибка отправки Auth-соединения: {error}"),
        }
    }
}

impl Error for AuthClientIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Send(error) => Some(error),
            Self::NotConnected | Self::Receive(_) => None,
        }
    }
}

/// Производный Auth-клиент LoginServer с общей send-очередью и Login FIFO.
pub(crate) struct CMyNetClientAuth {
    connection: Option<TcpStream>,
    connected_endpoint: Option<SocketAddrV4>,
    control_send: bool,
    send_queue: ClientSendQueue,
    receive_buffer: Vec<u8>,
    received_events: Arc<CMsgQueue<AuthClientEvent>>,
}

impl CMyNetClientAuth {
    /// Создаёт отключённое направление с исходной receive capacity.
    pub(crate) fn new() -> Self {
        Self {
            connection: None,
            connected_endpoint: None,
            control_send: false,
            send_queue: ClientSendQueue::new(),
            receive_buffer: Vec::with_capacity(INITIAL_RECEIVE_CAPACITY),
            received_events: Arc::new(CMsgQueue::new()),
        }
    }

    /// Подключает ранее созданный и bind-нутый IPv4 socket к AuthServer.
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

    /// Воспроизводит `OnClose -> HandleClose` и публикует `0xCF301`.
    pub(crate) fn handle_transport_close(&mut self) {
        self.connection = None;
        self.received_events
            .push(AuthClientEvent::Message(CMessage::new(CLOSE_MESSAGE_TYPE)));
    }

    /// Сообщает состояние исходного `m_bConnect`.
    pub(crate) fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    /// Возвращает подключённый stream будущему Login runtime-owner.
    pub(crate) fn connection(&self) -> Option<&TcpStream> {
        self.connection.as_ref()
    }

    /// Возвращает resolved IPv4 endpoint последнего успешного connect.
    pub(crate) const fn connected_endpoint(&self) -> Option<SocketAddrV4> {
        self.connected_endpoint
    }

    /// Включает исходный `m_bControlSend` после завершения init/reassign.
    pub(crate) fn enable_control_send(&mut self) {
        self.control_send = true;
    }

    /// Возвращает очередь, в которую `CMessage::SendToAS` копирует envelope.
    pub(crate) fn send_queue(&self) -> &ClientSendQueue {
        &self.send_queue
    }

    /// Пытается отправить текущий снимок Auth send-очереди.
    ///
    /// `None` буквально означает отсутствие подключённого socket; команды при
    /// этом остаются во владеющей очереди.
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
    pub(crate) async fn run_io_once(&mut self) -> Result<AuthClientIoStep, AuthClientIoError> {
        enum Ready {
            Read(io::Result<Vec<u8>>),
            Write(io::Result<()>),
        }

        let ready = {
            let stream = self
                .connection
                .as_ref()
                .ok_or(AuthClientIoError::NotConnected)?;
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
                Ok(AuthClientIoStep::Closed)
            }
            Ready::Read(Ok(received)) => {
                let messages = self
                    .accept_received_bytes(&received)
                    .map_err(AuthClientIoError::Receive)?;
                Ok(AuthClientIoStep::Received { messages })
            }
            Ready::Read(Err(error)) | Ready::Write(Err(error)) => Err(AuthClientIoError::Io(error)),
            Ready::Write(Ok(())) => {
                let stream = self
                    .connection
                    .as_ref()
                    .ok_or(AuthClientIoError::NotConnected)?;
                self.send_queue
                    .try_flush(stream)
                    .map(AuthClientIoStep::Sent)
                    .map_err(AuthClientIoError::Send)
            }
        }
    }

    /// Добавляет уже прочитанный TCP-фрагмент и разбирает все полные кадры.
    ///
    /// Возвращает число новых сообщений. При ошибке весь оставшийся вход
    /// отбрасывается, тогда как ранее опубликованные сообщения сохраняются.
    pub(crate) fn accept_received_bytes(&mut self, received: &[u8]) -> Result<usize, ReceiveError> {
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

            if (declared as i32) < 0 {
                // BLOCKED_MISSING_FACT: Login RVA 0x0006CB90 проверяет
                // отрицательную declared length только внутри ветки неполного
                // frame. При уже накопленных bytes путь уходит в `len - 12`.
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
                // BLOCKED_MISSING_FACT: 0..11 переполняет `total_len - 12`,
                // 13..27 передаёт CreateMessageWithoutRLE неполный header.
                return self.discard_pending(ReceiveError::ShortFrameReactionUnknown { declared });
            }

            let expected_content_crc = u32::from_le_bytes(
                frame[8..12]
                    .try_into()
                    .expect("наличие полного envelope уже проверено"),
            );
            let message_wire = &frame[SERVER_ENVELOPE_LEN..frame_len];
            let message = CMessage::create_without_rle(message_wire).map_err(ReceiveError::Message);
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

            self.received_events.push(AuthClientEvent::Message(message));
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

    /// Возвращает число сообщений, ожидающих доменного Login-прохода.
    pub(crate) fn pending_events(&self) -> i32 {
        self.received_events.get_size()
    }

    /// Передаёт старейшее Auth-событие фактическому владельцу `CGame`.
    pub(crate) fn pop_event(&self) -> Option<AuthClientEvent> {
        self.received_events.pop()
    }

    /// Публикует новый подключённый client в FIFO старого reconnect-owner.
    pub(crate) fn publish_reconnected(&self, client: CMyNetClientAuth) {
        self.received_events
            .push(AuthClientEvent::Reconnected(client));
    }

    /// Выдаёт reconnect-задаче право публикации только в FIFO этого клиента.
    pub(crate) fn event_publisher(&self) -> AuthClientEventPublisher {
        AuthClientEventPublisher {
            events: Arc::clone(&self.received_events),
        }
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

impl Default for CMyNetClientAuth {
    fn default() -> Self {
        Self::new()
    }
}
