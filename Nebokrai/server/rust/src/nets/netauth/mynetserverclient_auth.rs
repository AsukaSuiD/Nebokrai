//! Component-поведение принятого LoginServer-соединения, восстановленное из
//! `nets/netauth/mynetserverclient_auth.cpp`.
//!
//! Статус владельца: `IMPLEMENTED` для constructor state, `OnAccept`,
//! `OnClose` и корректного/неполного `OnReceive` envelope. Malformed-границы,
//! на которых исходный x86 уходил в небезопасную арифметику, оставлены
//! локальными ошибками `BLOCKED_MISSING_FACT`, а не объявлены исходным
//! fail-closed.
//!
//! Точная пара: `AuthServer/authserver.exe + AuthServer/authserver.pdb`;
//! SHA-256 EXE
//! `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15`,
//! SHA-256 PDB
//! `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`.
//! Исходный путь PDB:
//! `h:\fengyun\fy_russia\src\nets\netauth\mynetserverclient_auth.cpp`.
//!
//! Существенные RVA: конструктор `0x00015A10`, деструктор `0x00015A90`,
//! `OnAccept` `0x00015AD0`, `OnClose` `0x00015B60`, `OnReceive` `0x00015C10`.
//!
//! Производный конструктор выделял receive-buffer `0xA00000` и send-buffer
//! `0x100000`. Rust использует общий `CServerClient` с Auth-capacity для
//! receive; `Vec` send-buffer растёт сам, поэтому его исходная резервная
//! capacity не имеет отдельной наблюдаемой семантики. После разобранного кадра
//! receive capacity больше `0x100000` снова уменьшалась до `0x100000`; это уже
//! сохраняет общий accumulator-owner.
//!
//! `OnAccept` снимает close flag и ставит сообщение `0x0CF401`; `OnClose`
//! сначала ставит `0x0CF402`, затем выполняет общий close. Оба сообщения несут
//! socket ID и peer IPv4. `OnReceive` разбирает
//! `[total_len, crc(total_len), crc(normalized_message), message]`, создаёт
//! только несжатый Auth `CMessage`, копирует socket/map/CD-key/IP metadata и
//! передаёт владение общей FIFO-очереди.
//!
//! Length CRC проверяется до ожидания полного кадра. Content CRC считается по
//! нормализованному `CMessage`, а не обязательно по буквальному входному
//! header: create-путь перезаписывает его первое слово фактической длиной.
//! `Vec::drain` заменяет ручные realloc/memmove, сохраняя порядок и неполный
//! хвост. Второй мегабайтный buffer не читается этим исходником иначе, чем как
//! общий send accumulator; allocator, fill/vector internals, SEH и deleting-
//! destructor удалены как технический шум.

use crate::nets::msgqueue::CMsgQueue;
use crate::nets::netauth::message::{CMessage, CreateMessageError};
use crate::nets::serverclient::CServerClient;
use crate::public::crc32static::data_crc32;

const AUTH_INITIAL_RECEIVE_CAPACITY: usize = 0xA0_0000;
const SERVER_ENVELOPE_LEN: usize = 12;
const MESSAGE_HEADER_LEN: usize = 16;
const LOGIN_SERVER_CONNECTED: i32 = 0x000C_F401;
const LOGIN_SERVER_DISCONNECTED: i32 = 0x000C_F402;

/// Причина остановки Auth receive-path на конкретном TCP accumulator.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AuthReceiveError {
    /// CRC четырёх bytes `total_len` не совпал; оригинал очищал accumulator.
    LengthChecksumMismatch,
    /// CRC нормализованного внутреннего сообщения не совпал; вход отброшен.
    MessageChecksumMismatch,
    /// Безопасная реакция оригинала для этой длины envelope не доказана.
    EnvelopeLengthReactionUnknown(u32),
    /// Внутренний Auth message-owner отказался создавать сообщение.
    CreateMessage(CreateMessageError),
}

/// Результат одного полного прохода `OnReceive` по уже накопленным bytes.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct AuthReceiveOutcome {
    messages: i32,
    pending_bytes: usize,
}

impl AuthReceiveOutcome {
    /// Возвращает число переданных в FIFO сообщений.
    pub(crate) const fn messages(&self) -> i32 {
        self.messages
    }

    /// Возвращает длину сохранённого неполного TCP-хвоста.
    pub(crate) const fn pending_bytes(&self) -> usize {
        self.pending_bytes
    }
}

/// Статический component-owner virtual callbacks старого производного класса.
pub(crate) struct CMyNetServerClientAuth;

impl CMyNetServerClientAuth {
    /// Создаёт общий client-state с доказанным Auth receive accumulator.
    pub(crate) fn new_state(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> CServerClient {
        CServerClient::with_receive_capacity(
            socket_id,
            peer_ipv4,
            now_ms,
            AUTH_INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Публикует исходное synthetic connect-сообщение.
    pub(crate) fn on_accept(client: &mut CServerClient, messages: &CMsgQueue<CMessage>) {
        client.mark_open();
        let mut message = CMessage::new(LOGIN_SERVER_CONNECTED);
        let context = client.message_context();
        message.apply_socket_context(context.socket_id, context.peer_ipv4);
        messages.push(message);
    }

    /// Публикует disconnect-сообщение до изменения общего close-state.
    pub(crate) fn on_close(client: &mut CServerClient, messages: &CMsgQueue<CMessage>) {
        let mut message = CMessage::new(LOGIN_SERVER_DISCONNECTED);
        let context = client.message_context();
        message.apply_socket_context(context.socket_id, context.peer_ipv4);
        messages.push(message);
        client.mark_closing();
    }

    /// Разбирает все полные Auth-envelope из общего TCP accumulator.
    ///
    /// Входной fragment должен быть заранее добавлен через
    /// `CServerClient::add_receive_data`, как в старом completion-worker.
    pub(crate) fn on_receive(
        client: &mut CServerClient,
        recv_time_ms: u32,
        messages: &CMsgQueue<CMessage>,
    ) -> Result<AuthReceiveOutcome, AuthReceiveError> {
        let mut produced = 0_i32;

        while client.pending_receive_bytes() >= SERVER_ENVELOPE_LEN {
            let frame = client.receive_bytes();
            let total_length = u32::from_le_bytes(
                frame[..4]
                    .try_into()
                    .expect("длина server envelope уже проверена"),
            );
            let length_checksum = u32::from_le_bytes(
                frame[4..8]
                    .try_into()
                    .expect("длина server envelope уже проверена"),
            );
            if data_crc32(&frame[..4]) != length_checksum {
                client.discard_receive_data();
                return Err(AuthReceiveError::LengthChecksumMismatch);
            }

            let Ok(total_length_usize) = usize::try_from(total_length) else {
                unreachable!("u32 всегда представим как usize на целевой Linux-платформе");
            };
            let minimum_length = SERVER_ENVELOPE_LEN + MESSAGE_HEADER_LEN;
            if total_length_usize < minimum_length || total_length > i32::MAX as u32 {
                // BLOCKED_MISSING_FACT: Auth RVA 0x00015C10 смешивает signed
                // compare с `uint total_len`, затем вычисляет `len - 12` и
                // передаёт его фабрике. Для <28 и sign-bit реакция процесса
                // требует отдельной машинной проверки.
                return Err(AuthReceiveError::EnvelopeLengthReactionUnknown(
                    total_length,
                ));
            }
            if frame.len() < total_length_usize {
                break;
            }

            let expected_message_checksum = u32::from_le_bytes(
                frame[8..12]
                    .try_into()
                    .expect("длина server envelope уже проверена"),
            );
            let wire = &frame[SERVER_ENVELOPE_LEN..total_length_usize];
            let mut message = CMessage::create_without_rle(wire, recv_time_ms)
                .map_err(AuthReceiveError::CreateMessage)?;
            if data_crc32(message.as_wire_bytes()) != expected_message_checksum {
                client.discard_receive_data();
                return Err(AuthReceiveError::MessageChecksumMismatch);
            }

            message.apply_client_context(client.message_context());
            messages.push(message);
            produced = produced.wrapping_add(1);
            let consumed = client.consume_receive_prefix(total_length_usize);
            debug_assert!(consumed, "полный Auth frame уже проверен в accumulator");
        }

        Ok(AuthReceiveOutcome {
            messages: produced,
            pending_bytes: client.pending_receive_bytes(),
        })
    }
}
