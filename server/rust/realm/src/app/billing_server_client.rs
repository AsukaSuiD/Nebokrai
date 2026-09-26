//! Принятое GameServer-соединение BillingServer из
//! `nets/netbilling/clientforgs.cpp`; Realm — владелец состояния принятого
//! клиента направления Billing. Источник контракта — та же точная пара, что у
//! [`crate::app::billing_message`].
//!
//! Owner реализует закрытие и разбор корректного либо неполного `OnReceive`.
//! Небезопасные malformed-границы длины и короткого внутренного header
//! детерминированно очищают accumulator и возвращают локальную ошибку.
//! `SetSendRevBuf` остаётся отдельной transport-границей: совместимый Linux
//! socket option неизвестен.
//!
//! Receive-envelope имеет форму
//! `[total_len, crc(total_len), crc(normalized_message), message]`. Length CRC
//! проверяется сразу после появления 12 bytes, до ожидания полного кадра.
//! Внутреннее сообщение не сжато: `CreateMessageWithoutRLE` нормализует первое
//! слово header фактической длиной, после чего content CRC считается по
//! созданному объекту. Только прошедшее обе проверки сообщение получает
//! socket/map/CD-key/IP metadata соединения и передаётся общей FIFO. Ошибка
//! любой CRC и доказанный null create-путь очищают весь ещё не разобранный
//! вход; IP-ban и `QUIT` этот компонент не ставил. Старые diagnostics ошибочно
//! называли peer `WorldServer`; Rust сохраняет факты ошибки typed-значениями,
//! не превращая опечатку в новый logging API.
//!
//! `OnClose` всегда публикует `0x10EF01`, добавляя текущий map ID, dotted peer
//! IPv4 с NUL (accept-path через `inet_ntoa`) и constructor port `5000`
//! (`Ipv4Addr` и `DEFAULT_PORT` воспроизводят значения без WinSock) до общего
//! close-state. Socket ID, CD-key и числовой IP сообщению не присваиваются.
//!
//! Malformed-границы длины (sign bit, `total_len < 12`) и короткого header,
//! где x86 уходил в signed/unsigned арифметику или чтение за границей, safe
//! Rust отбрасывает весь текущий accumulator без воспроизведения UB.
//!
//! Доказательства: docs/reconstruction/realm-services.md#billing-принятое-game-соединение

use std::net::Ipv4Addr;

use nebokrai_shared::network::{CMsgQueue, CServerClient, DEFAULT_PORT};
use nebokrai_shared::protocol::data_crc32;

use super::billing_message::{CMessage, CreateMessageError};

const BILLING_INITIAL_RECEIVE_CAPACITY: usize = 0xA0_0000;
const SERVER_ENVELOPE_LEN: usize = 12;
const GAME_SERVER_DISCONNECTED: i32 = 0x0010_EF01;

/// Причина остановки Billing receive-path на текущем accumulator.
#[derive(Debug, Eq, PartialEq)]
pub enum BillingReceiveError {
    /// CRC little-endian слова `total_len` не совпал со вторым словом envelope.
    LengthChecksumMismatch { expected: u32, actual: u32 },
    /// Длина меньше envelope либо не представима положительным Windows `long`.
    InvalidFrameLength { declared: u32 },
    /// Конкретный message-owner не смог безопасно создать сообщение.
    Message(CreateMessageError),
    /// CRC нормализованного сообщения не совпал с третьим словом envelope.
    ContentChecksumMismatch {
        message_type: i32,
        expected: u32,
        actual: u32,
    },
}

/// Результат одного полного прохода `OnReceive` по накопленному TCP-потоку.
#[derive(Debug, Eq, PartialEq)]
pub struct BillingReceiveOutcome {
    messages: i32,
    pending_bytes: usize,
}

impl BillingReceiveOutcome {
    /// Возвращает число сообщений, переданных в общую FIFO.
    pub const fn messages(&self) -> i32 {
        self.messages
    }

    /// Возвращает длину сохранённого неполного TCP-хвоста.
    pub const fn pending_bytes(&self) -> usize {
        self.pending_bytes
    }
}

/// Статический component-owner virtual callbacks старого производного класса.
pub struct CClientForGS;

impl CClientForGS {
    /// Создаёт общий client-state с доказанным Billing accumulator `0xA00000`.
    pub fn new_state(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> CServerClient {
        CServerClient::with_receive_capacity(
            socket_id,
            peer_ipv4,
            now_ms,
            BILLING_INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Публикует unconditional `0x10EF01` и устанавливает общий close-state.
    pub fn on_close(client: &mut CServerClient, messages: &CMsgQueue<CMessage>) {
        let context = client.message_context();
        let map_id = context.map_id;
        let peer_ip = Ipv4Addr::from(context.peer_ipv4.to_le_bytes())
            .to_string()
            .into_bytes();

        let mut message = CMessage::new(GAME_SERVER_DISCONNECTED);
        message.base_mut().add_long(map_id);
        message.base_mut().add(&peer_ip);
        message.base_mut().add_byte(0);
        message.base_mut().add_long(DEFAULT_PORT as i32);
        messages.push(message);
        client.mark_closing();
    }

    /// Разбирает все полные GameServer-envelope из общего TCP accumulator.
    ///
    /// Входной fragment должен быть заранее добавлен через
    /// `CServerClient::add_receive_data`. Аргумент времени сохранён ради общей
    /// virtual-границы; исходный Billing `OnReceive` его не использовал.
    pub fn on_receive(
        client: &mut CServerClient,
        _recv_time_ms: u32,
        messages: &CMsgQueue<CMessage>,
    ) -> Result<BillingReceiveOutcome, BillingReceiveError> {
        let mut produced = 0_i32;

        while client.pending_receive_bytes() >= SERVER_ENVELOPE_LEN {
            let frame = client.receive_bytes();
            let declared_bytes: [u8; 4] = frame[..4]
                .try_into()
                .expect("наличие полного Billing envelope уже проверено");
            let declared = u32::from_le_bytes(declared_bytes);
            let expected_length_crc = u32::from_le_bytes(
                frame[4..8]
                    .try_into()
                    .expect("наличие полного Billing envelope уже проверено"),
            );
            let actual_length_crc = data_crc32(&declared_bytes);
            if actual_length_crc != expected_length_crc {
                client.discard_receive_data();
                return Err(BillingReceiveError::LengthChecksumMismatch {
                    expected: expected_length_crc,
                    actual: actual_length_crc,
                });
            }

            if (declared as i32) < 0 {
                client.discard_receive_data();
                return Err(BillingReceiveError::InvalidFrameLength { declared });
            }

            let frame_length = declared as usize;
            if frame.len() < frame_length {
                break;
            }
            if frame_length < SERVER_ENVELOPE_LEN {
                client.discard_receive_data();
                return Err(BillingReceiveError::InvalidFrameLength { declared });
            }

            let message_wire = &frame[SERVER_ENVELOPE_LEN..frame_length];
            let message = match CMessage::create_without_rle(message_wire) {
                Ok(message) => message,
                Err(error) => {
                    client.discard_receive_data();
                    return Err(BillingReceiveError::Message(error));
                }
            };

            let expected_content_crc = u32::from_le_bytes(
                frame[8..12]
                    .try_into()
                    .expect("наличие полного Billing envelope уже проверено"),
            );
            let actual_content_crc = data_crc32(message.as_wire_bytes());
            if actual_content_crc != expected_content_crc {
                let message_type = message.message_type();
                client.discard_receive_data();
                return Err(BillingReceiveError::ContentChecksumMismatch {
                    message_type,
                    expected: expected_content_crc,
                    actual: actual_content_crc,
                });
            }

            let mut message = message;
            message.apply_client_context(client.message_context());
            messages.push(message);
            produced = produced.wrapping_add(1);
            let consumed = client.consume_receive_prefix(frame_length);
            debug_assert!(consumed, "полный Billing frame уже проверен в accumulator");
        }

        Ok(BillingReceiveOutcome {
            messages: produced,
            pending_bytes: client.pending_receive_bytes(),
        })
    }
}

// `SetSendRevBuf` задавал Windows `SO_SNDBUF=0`. Совместимая наблюдаемая
// семантика backpressure для Linux неизвестна, поэтому отличающийся socket
// option в server-owner не назначен.
