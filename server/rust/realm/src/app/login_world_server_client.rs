//! Входное соединение WorldServer у LoginServer из
//! `nets/netlogin/mynetserverclient_world.cpp`, перенесённое в Realm — состояние
//! принятого World-соединения Login-направления. Источник контракта — та же
//! точная пара, что у [`crate::app::login_message`].
//!
//! Машинно подтверждённые точки (первая секция `.exe/loginserver.exe`):
//! - ctor `CMyNetServerClient_World` `0x46EC60`: base `0x46F160`, vtable
//!   `0x49DC98`, receive buffer ровно `0xA00000` (`push 0xA00000` + alloc),
//!   send-компаньон `0x100000` (`+0x70` family);
//! - `OnReceive` `0x46DDF0`: цикл, пока накоплено `>= 0xC`; CRC длины через
//!   `DataCrc32 0x47F050`, `declared > size` — останов без потери хвоста,
//!   create через `CreateMessageWithoutRLE 0x465710`, повторный CRC
//!   содержимого; контекст `[+0x34]→[+0x24]` socket, `[+0x88]→[+0x20]` map,
//!   byte-string `[+0x90]→[+0x2C]` CD-key (assign `0x401440`),
//!   `[+0x2C]→[+0x28]` IPv4; publish owner `+0xDC` через `0x46AE90`; consume;
//!   shrink к `0x100000` при возврате под лимит; reject очищает accumulator
//!   без отката опубликованного. IP-ban и `QUIT` у исходника нет;
//! - `OnClose` `0x46ED10`: `new(0x48)` `CMessage(0xFF01)` через ctor
//!   `0x465540`, `[client+0x88]→[+0x20]` (map identity), publish owner `+0xDC`
//!   и затем общий close `0x46C740(0)` — безусловно, всегда.
//!
//! Owner реализует закрытие и разбор корректного либо неполного `OnReceive`.
//! Небезопасные malformed-границы длины и короткого внутреннего header
//! детерминированно очищают accumulator и возвращают локальную ошибку.
//! `SetSendRevBuf` остаётся отдельной transport-границей: совместимый Linux
//! socket option неизвестен.
//!
//! Производный конструктор выделял receive-buffer `0xA00000` и send-buffer
//! `0x100000`. Rust использует общий `CServerClient` с World-capacity для
//! receive; его owned send accumulator растёт сам и не получает второго
//! component-поля. `Vec::drain` заменяет ручные realloc/memmove, сохраняя
//! порядок кадров и неполный TCP-хвост.
//!
//! World receive-envelope всегда имеет форму
//! `[total_len, crc(total_len), crc(normalized_message), message]`.
//! Length CRC проверяется сразу после появления 12 bytes, до ожидания полного
//! кадра. Внутреннее сообщение не сжато: сначала `CreateMessageWithoutRLE`
//! нормализует первое слово header фактической длиной, затем content CRC
//! считается по этому объекту. Только прошедшее обе проверки сообщение
//! получает socket/map/name/IP metadata и передаётся общей FIFO. Ошибка любой
//! CRC и доказанный `nullptr` create-пути очищают весь ещё не разобранный вход;
//! IP-ban и `QUIT` этот исходник не ставил.
//!
//! `OnClose` всегда создаёт сообщение `0xFF01`, присваивает ему только текущий
//! map ID, публикует его и затем выполняет общий close. Rust не заполняет
//! отсутствующие socket/name/IP metadata по аналогии с receive-путём.
//!
//! Для длины с sign bit либо `total_len < 12` x86-путь переходил к signed
//! сравнению и/или unsigned `len - 12`; внутреннее сообщение длиной 1..15
//! bytes также приводило к чтению header за границей. Safe Rust очищает такой
//! вход без воспроизведения UB. SEH, allocator-
//! копии и deleting-destructor удалены как compiler/library noise.

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
