//! Принятое GameServer-соединение WorldServer из `myserverclient.cpp`.
//! Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Receive разбирает все полные CRC-envelope frames, сохраняет неполный хвост
//! и назначает принятому сообщению socket/map/IP metadata. Ошибка очищает
//! оставшийся accumulator без отката ранее опубликованных сообщений. `OnClose`
//! публикует `0x3FC02 + map_id` до общего socket close. Owned buffers и typed
//! errors заменяют fixed allocations и небезопасные malformed-frame ветви,
//! не меняя корректный wire, FIFO или порядок lifecycle effects.

use crate::nets::serverclient::CServerClient;
use crate::public::crc32static::data_crc32;

use super::message::{CMessage, CreateMessageError};

const WORLD_INITIAL_RECEIVE_CAPACITY: usize = 0x140_0000;
const SERVER_ENVELOPE_LEN: usize = 12;
const GAME_SERVER_DISCONNECTED: i32 = 0x0003_FC02;

/// Причина остановки World GameServer receive-path на текущем accumulator.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum GameServerReceiveError {
    /// CRC little-endian слова `total_len` не совпал со вторым словом envelope.
    LengthChecksumMismatch { expected: u32, actual: u32 },
    /// Длина меньше envelope либо не представима положительным Windows `long`.
    InvalidFrameLength { declared: u32 },
    /// Конкретный World message-owner не смог безопасно создать сообщение.
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
pub(crate) struct GameServerReceiveOutcome {
    messages: i32,
    pending_bytes: usize,
}

impl GameServerReceiveOutcome {
    /// Возвращает число сообщений, переданных в будущую общую World FIFO.
    pub(crate) const fn messages(&self) -> i32 {
        self.messages
    }

    /// Возвращает длину сохранённого неполного TCP-хвоста.
    pub(crate) const fn pending_bytes(&self) -> usize {
        self.pending_bytes
    }
}

/// Статический component-owner virtual callbacks старого производного класса.
pub(crate) struct CMyServerClient;

/// Узкая producer-граница общей World FIFO.
///
/// Конкретный server-owner оборачивает обычное wire-сообщение в свой typed
/// event, чтобы внутрипроцессный reconnect мог занимать ту же очередь без
/// integer-pointer payload.
pub(crate) trait WorldMessageSink {
    /// Публикует обычное GameServer либо synthetic close-сообщение.
    fn publish_message(&self, message: CMessage);
}

impl CMyServerClient {
    /// Создаёт общий client-state с доказанным World accumulator `0x1400000`.
    pub(crate) fn new_state(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> CServerClient {
        CServerClient::with_receive_capacity(
            socket_id,
            peer_ipv4,
            now_ms,
            WORLD_INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Публикует unconditional `0x3FC02 + map ID`, затем начинает close.
    pub(crate) fn on_close(client: &mut CServerClient, messages: &impl WorldMessageSink) {
        let mut message = CMessage::new(GAME_SERVER_DISCONNECTED);
        message.base_mut().add_long(client.message_context().map_id);
        messages.publish_message(message);
        client.mark_closing();
    }

    /// Разбирает все полные GameServer-envelope из общего TCP accumulator.
    ///
    /// Входной fragment должен быть заранее добавлен через
    /// `CServerClient::add_receive_data`, как в старом completion-worker.
    pub(crate) fn on_receive(
        client: &mut CServerClient,
        recv_time_ms: u32,
        messages: &impl WorldMessageSink,
    ) -> Result<GameServerReceiveOutcome, GameServerReceiveError> {
        let mut produced = 0_i32;

        while client.pending_receive_bytes() >= SERVER_ENVELOPE_LEN {
            let frame = client.receive_bytes();
            let declared_bytes: [u8; 4] = frame[..4]
                .try_into()
                .expect("наличие полного World GameServer envelope уже проверено");
            let declared = u32::from_le_bytes(declared_bytes);
            let expected_length_crc = u32::from_le_bytes(
                frame[4..8]
                    .try_into()
                    .expect("наличие полного World GameServer envelope уже проверено"),
            );
            let actual_length_crc = data_crc32(&declared_bytes);
            if actual_length_crc != expected_length_crc {
                client.discard_receive_data();
                return Err(GameServerReceiveError::LengthChecksumMismatch {
                    expected: expected_length_crc,
                    actual: actual_length_crc,
                });
            }

            if (declared as i32) < 0 {
                client.discard_receive_data();
                return Err(GameServerReceiveError::InvalidFrameLength { declared });
            }

            let frame_length = declared as usize;
            if frame.len() < frame_length {
                break;
            }
            if frame_length < SERVER_ENVELOPE_LEN {
                client.discard_receive_data();
                return Err(GameServerReceiveError::InvalidFrameLength { declared });
            }

            let message_wire = &frame[SERVER_ENVELOPE_LEN..frame_length];
            let message = match CMessage::create_without_rle(message_wire, recv_time_ms) {
                Ok(message) => message,
                Err(error) => {
                    client.discard_receive_data();
                    return Err(GameServerReceiveError::Message(error));
                }
            };

            let expected_content_crc = u32::from_le_bytes(
                frame[8..12]
                    .try_into()
                    .expect("наличие полного World GameServer envelope уже проверено"),
            );
            let actual_content_crc = data_crc32(message.as_wire_bytes());
            if actual_content_crc != expected_content_crc {
                let message_type = message.message_type();
                client.discard_receive_data();
                return Err(GameServerReceiveError::ContentChecksumMismatch {
                    message_type,
                    expected: expected_content_crc,
                    actual: actual_content_crc,
                });
            }

            let mut message = message;
            message.apply_client_context(client.message_context());
            messages.publish_message(message);
            produced = produced.wrapping_add(1);
            let consumed = client.consume_receive_prefix(frame_length);
            debug_assert!(
                consumed,
                "полный World GameServer frame уже проверен в accumulator"
            );
        }

        Ok(GameServerReceiveOutcome {
            messages: produced,
            pending_bytes: client.pending_receive_bytes(),
        })
    }
}
