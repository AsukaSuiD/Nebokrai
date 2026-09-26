//! Входное соединение WorldServer у LoginServer из
//! `nets/netlogin/mynetserverclient_world.cpp`; Realm — состояние принятого
//! World-соединения Login-направления. Источник контракта — та же точная
//! пара, что у [`crate::app::login_message`].
//!
//! Owner реализует закрытие и разбор корректного либо неполного `OnReceive`.
//! World receive-envelope всегда несжатый `[total_len, crc(total_len),
//! crc(normalized_message), message]`: length CRC сразу после появления
//! 12 bytes, несжатый create нормализует первое слово header фактической
//! длиной, затем content CRC считается по этому объекту. Только прошедшее обе
//! проверки сообщение получает socket/map/name/IP metadata и передаётся общей
//! FIFO. Ошибка любой CRC и доказанный `nullptr` create-пути очищают весь ещё
//! не разобранный вход; IP-ban и `QUIT` исходник не ставил.
//! Malformed-границы длины и короткого внутреннего header, где x86 переходил
//! к signed/unsigned арифметике или чтению за границей, детерминированно
//! очищают accumulator без UB; неполный TCP-хвост сохраняется.
//!
//! `OnClose` безусловно создаёт `0xFF01` только с текущим map ID, публикует
//! его и выполняет общий close; отсутствующие socket/name/IP metadata
//! специально не заполняются. `SetSendRevBuf` остаётся локальной
//! transport-границей: совместимый Linux socket option неизвестен (UNKNOWN).
//!
//! Доказательства: docs/reconstruction/realm-services.md#login-принятое-world-соединение

use nebokrai_shared::network::{CMsgQueue, CServerClient};
use nebokrai_shared::protocol::data_crc32;

use super::login_message::{CMessage, CreateMessageError};

const WORLD_INITIAL_RECEIVE_CAPACITY: usize = 0xA0_0000;
const SERVER_ENVELOPE_LEN: usize = 12;
const WORLD_DISCONNECTED: i32 = 0x0000_FF01;

/// Причина остановки Login WorldServer receive-path на текущем accumulator.
#[derive(Debug, Eq, PartialEq)]
pub enum WorldReceiveError {
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
pub struct WorldReceiveOutcome {
    messages: i32,
    pending_bytes: usize,
}

impl WorldReceiveOutcome {
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
pub struct CMyNetServerClientWorld;

impl CMyNetServerClientWorld {
    /// Создаёт общий client-state с доказанным World accumulator `0xA00000`.
    pub fn new_state(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> CServerClient {
        CServerClient::with_receive_capacity(
            socket_id,
            peer_ipv4,
            now_ms,
            WORLD_INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Публикует unconditional `0xFF01` с map ID и устанавливает close-state.
    pub fn on_close(client: &mut CServerClient, messages: &CMsgQueue<CMessage>) {
        let mut message = CMessage::new(WORLD_DISCONNECTED);
        message.apply_map_id(client.message_context().map_id);
        messages.push(message);
        client.mark_closing();
    }

    /// Разбирает все полные WorldServer-envelope из общего TCP accumulator.
    ///
    /// Входной fragment должен быть заранее добавлен через
    /// `CServerClient::add_receive_data`. Аргумент времени сохранён ради общей
    /// virtual-границы; исходный Login `OnReceive` его не использовал.
    pub fn on_receive(
        client: &mut CServerClient,
        _recv_time_ms: u32,
        messages: &CMsgQueue<CMessage>,
    ) -> Result<WorldReceiveOutcome, WorldReceiveError> {
        let mut produced = 0_i32;

        while client.pending_receive_bytes() >= SERVER_ENVELOPE_LEN {
            let frame = client.receive_bytes();
            let declared_bytes: [u8; 4] = frame[..4]
                .try_into()
                .expect("наличие полного WorldServer envelope уже проверено");
            let declared = u32::from_le_bytes(declared_bytes);
            let expected_length_crc = u32::from_le_bytes(
                frame[4..8]
                    .try_into()
                    .expect("наличие полного WorldServer envelope уже проверено"),
            );
            let actual_length_crc = data_crc32(&declared_bytes);
            if actual_length_crc != expected_length_crc {
                client.discard_receive_data();
                return Err(WorldReceiveError::LengthChecksumMismatch {
                    expected: expected_length_crc,
                    actual: actual_length_crc,
                });
            }

            if (declared as i32) < 0 {
                client.discard_receive_data();
                return Err(WorldReceiveError::InvalidFrameLength { declared });
            }

            let frame_length = declared as usize;
            if frame.len() < frame_length {
                break;
            }
            if frame_length < SERVER_ENVELOPE_LEN {
                client.discard_receive_data();
                return Err(WorldReceiveError::InvalidFrameLength { declared });
            }

            let message_wire = &frame[SERVER_ENVELOPE_LEN..frame_length];
            let message = match CMessage::create_without_rle(message_wire) {
                Ok(message) => message,
                Err(error) => {
                    client.discard_receive_data();
                    return Err(WorldReceiveError::Message(error));
                }
            };

            let expected_content_crc = u32::from_le_bytes(
                frame[8..12]
                    .try_into()
                    .expect("наличие полного WorldServer envelope уже проверено"),
            );
            let actual_content_crc = data_crc32(message.as_wire_bytes());
            if actual_content_crc != expected_content_crc {
                let message_type = message.message_type();
                client.discard_receive_data();
                return Err(WorldReceiveError::ContentChecksumMismatch {
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
            debug_assert!(
                consumed,
                "полный WorldServer frame уже проверен в accumulator"
            );
        }

        Ok(WorldReceiveOutcome {
            messages: produced,
            pending_bytes: client.pending_receive_bytes(),
        })
    }
}

// `SetSendRevBuf` задавал Windows `SO_SNDBUF=0`. Совместимая наблюдаемая
// семантика backpressure для Linux неизвестна, поэтому отличающийся socket
// option в server-owner не назначен.
