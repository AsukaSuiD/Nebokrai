//! Входное соединение игрового клиента LoginServer из
//! `nets/netlogin/mynetserverclient_client.cpp`; Realm — состояние принятого
//! игрока Login-направления. Источник контракта — та же точная пара,
//! что у [`crate::app::login_message`].
//!
//! Owner сохраняет условные (по owner-флагам) проверки полной длины и обеих
//! CRC, RLE create-путь, допустимый opcode-диапазон `(0x2FD00, 0x3FC00)`,
//! metadata, FIFO-публикацию, неполный TCP-хвост и synthetic disconnect только
//! по непустому CD-key: пустой CD-key не получает ни публикации, ни общего
//! close (машинный quirk). Небезопасные malformed-границы длины и RLE
//! детерминированно очищают accumulator и возвращают локальную ошибку без
//! воспроизведения UB.
//!
//! Превышение длины, обе ошибки CRC и opcode вне диапазона доказанно требуют
//! последовательности component diagnostic -> `AddForbidIP` ->
//! `QuitClientBySocketID`; этот файл классифицирует реакцию, а применяет её
//! фактический server-owner (`crate::app::login_server`). Ошибка создания с
//! доказанным нулевым результатом очищает accumulator без ban — отличие
//! ветки. Старые `PutDebugString` не превращаются в новый logging API;
//! параметры диагностик сохранены в типизированных ошибках для server-owner’а.
//!
//! Исходный второй buffer `0xC800` был техническим send accumulator
//! унаследованного server-client и не получает отдельного дублирующего поля.
//! При длине с sign bit либо `total_len < 12` x86-путь переходил к signed
//! сравнению и/или unsigned `len - 12`; как и trailing RLE marker или
//! декодированный header короче 16 bytes, safe Rust отбрасывает такой вход без
//! unsafe-чтения.
//!
//! Доказательства: docs/reconstruction/realm-services.md#login-принятое-клиентское-соединение

use nebokrai_shared::network::{CMsgQueue, CServerClient};
use nebokrai_shared::protocol::data_crc32;

use super::login_message::{CMessage, CreateMessageError};

const CLIENT_INITIAL_RECEIVE_CAPACITY: usize = 0x5000;
const CLIENT_ENVELOPE_LEN: usize = 12;
const CLIENT_MESSAGE_MIN: u32 = 0x0002_FD01;
const CLIENT_MESSAGE_MAX: u32 = 0x0003_FBFF;
const CLIENT_DISCONNECTED: i32 = 0x0001_0001;

/// Три setup-значения, которые исходный client receive-path читал у сервера.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientReceiveSettings {
    check_length_crc: bool,
    check_content_crc: bool,
    maximum_message_length: u32,
}

impl ClientReceiveSettings {
    /// Создаёт буквальный snapshot проверок одного вызова `OnReceive`.
    pub const fn new(
        check_length_crc: bool,
        check_content_crc: bool,
        maximum_message_length: u32,
    ) -> Self {
        Self {
            check_length_crc,
            check_content_crc,
            maximum_message_length,
        }
    }
}

/// Причина остановки Login client receive-path на текущем accumulator.
#[derive(Debug, Eq, PartialEq)]
pub enum ClientReceiveError {
    /// Объявленный размер превысил setup-предел при включённой length-проверке.
    MessageLengthExceeded { declared: u32, permitted: u32 },
    /// CRC little-endian слова `total_len` не совпал со вторым словом envelope.
    LengthChecksumMismatch { expected: u32, actual: u32 },
    /// CRC сжатой части кадра не совпал с третьим словом envelope.
    ContentChecksumMismatch { expected: u32, actual: u32 },
    /// Длина меньше envelope либо не представима положительным Windows `long`.
    InvalidFrameLength { declared: u32 },
    /// Конкретный message-owner не смог безопасно создать сообщение.
    Message(CreateMessageError),
    /// Полный тип сообщения находится вне разрешённого client-диапазона.
    OpcodeOutsideAllowedRange { opcode: u32 },
}

impl ClientReceiveError {
    /// Сообщает только о четырёх путях с доказанными `AddForbidIP` и `QUIT`.
    pub const fn requires_forbid_and_quit(&self) -> bool {
        matches!(
            self,
            Self::MessageLengthExceeded { .. }
                | Self::LengthChecksumMismatch { .. }
                | Self::ContentChecksumMismatch { .. }
                | Self::OpcodeOutsideAllowedRange { .. }
        )
    }
}

/// Результат одного полного прохода `OnReceive` по накопленному TCP-потоку.
#[derive(Debug, Eq, PartialEq)]
pub struct ClientReceiveOutcome {
    messages: i32,
    pending_bytes: usize,
}

impl ClientReceiveOutcome {
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
pub struct CMyNetServerClientClient;

impl CMyNetServerClientClient {
    /// Создаёт общий client-state с доказанным начальным accumulator `0x5000`.
    pub fn new_state(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> CServerClient {
        CServerClient::with_receive_capacity(
            socket_id,
            peer_ipv4,
            now_ms,
            CLIENT_INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Публикует synthetic disconnect только для уже назначенного CD-key.
    ///
    /// Возвращает `true`, если сообщение было опубликовано и исходный общий
    /// close-state установлен. Пустой CD-key оставляет его без изменения.
    pub fn on_close(client: &mut CServerClient, messages: &CMsgQueue<CMessage>) -> bool {
        let cdkey = client.message_context().map_name.to_vec();
        if cdkey.is_empty() {
            return false;
        }

        let mut message = CMessage::new(CLIENT_DISCONNECTED);
        message.base_mut().add(&cdkey);
        message.base_mut().add_byte(0);
        messages.push(message);
        client.mark_closing();
        true
    }

    /// Разбирает все полные client-envelope из общего TCP accumulator.
    ///
    /// Входной fragment должен быть заранее добавлен через
    /// `CServerClient::add_receive_data`. Аргумент времени сохранён ради общей
    /// virtual-границы; исходный Login `OnReceive` его не использовал.
    pub fn on_receive(
        client: &mut CServerClient,
        _recv_time_ms: u32,
        settings: ClientReceiveSettings,
        messages: &CMsgQueue<CMessage>,
    ) -> Result<ClientReceiveOutcome, ClientReceiveError> {
        let mut produced = 0_i32;

        while client.pending_receive_bytes() >= CLIENT_ENVELOPE_LEN {
            let frame = client.receive_bytes();
            let declared_bytes: [u8; 4] = frame[..4]
                .try_into()
                .expect("наличие полного client envelope уже проверено");
            let declared = u32::from_le_bytes(declared_bytes);

            if settings.check_length_crc {
                if (settings.maximum_message_length as i32) < (declared as i32) {
                    return Err(ClientReceiveError::MessageLengthExceeded {
                        declared,
                        permitted: settings.maximum_message_length,
                    });
                }

                let expected = u32::from_le_bytes(
                    frame[4..8]
                        .try_into()
                        .expect("наличие полного client envelope уже проверено"),
                );
                let actual = data_crc32(&declared_bytes);
                if actual != expected {
                    return Err(ClientReceiveError::LengthChecksumMismatch { expected, actual });
                }
            }

            if (declared as i32) < 0 {
                client.discard_receive_data();
                return Err(ClientReceiveError::InvalidFrameLength { declared });
            }

            let frame_length = declared as usize;
            if frame.len() < frame_length {
                break;
            }
            if frame_length < CLIENT_ENVELOPE_LEN {
                client.discard_receive_data();
                return Err(ClientReceiveError::InvalidFrameLength { declared });
            }

            let compressed = &frame[CLIENT_ENVELOPE_LEN..frame_length];
            if settings.check_content_crc {
                let expected = u32::from_le_bytes(
                    frame[8..12]
                        .try_into()
                        .expect("наличие полного client envelope уже проверено"),
                );
                let actual = data_crc32(compressed);
                if actual != expected {
                    return Err(ClientReceiveError::ContentChecksumMismatch { expected, actual });
                }
            }

            let mut message = match CMessage::create(compressed) {
                Ok(message) => message,
                Err(error) => {
                    client.discard_receive_data();
                    return Err(ClientReceiveError::Message(error));
                }
            };

            let opcode = message.message_type() as u32;
            if !(CLIENT_MESSAGE_MIN..=CLIENT_MESSAGE_MAX).contains(&opcode) {
                return Err(ClientReceiveError::OpcodeOutsideAllowedRange { opcode });
            }

            message.apply_client_context(client.message_context());
            messages.push(message);
            produced = produced.wrapping_add(1);
            let consumed = client.consume_receive_prefix(frame_length);
            debug_assert!(consumed, "полный client frame уже проверен в accumulator");
        }

        Ok(ClientReceiveOutcome {
            messages: produced,
            pending_bytes: client.pending_receive_bytes(),
        })
    }
}
