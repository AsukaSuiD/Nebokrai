//! Входное соединение игрового клиента LoginServer из
//! `nets/netlogin/mynetserverclient_client.cpp`, перенесённое в Realm — состояние
//! принятого игрока Login-направления. Источник контракта — та же точная пара,
//! что у [`crate::app::login_message`].
//!
//! Машинно подтверждённые точки (первая секция `.exe/loginserver.exe`):
//! - ctor `CMyNetServerClient_Client` `0x46E590`: base `0x46F160`, vtable
//!   `0x49DB10`, receive buffer ровно `0x5000` (`push 0x5000` + alloc),
//!   два компаньона `0xC800` в `+0x70/+0x74` (исходный send accumulator —
//!   без выделенного Rust-поля);
//! - `OnReceive` `0x46E810`: gate owner `+0xA8`, цикл, пока накоплено `>= 0xC`;
//!   при включённом флаге `[owner+0x10C]` сначала предел полной длины
//!   (`declared > [owner+0x118]` → виртуальное `OnTotalMessageSizeOver`
//!   `[vtable+0x40]`), затем CRC длины `0x47F050`; `declared > size` — останов
//!   без потери хвоста; при включённом флаге `[owner+0x10D]` — CRC по сжатым
//!   байтам `[+0xC, declared-0xC]` через `0x47F050`; create только RLE
//!   `0x4655C0`; opcode допустим в `(0x2FD00, 0x3FC00)` exclusive —
//!   контекст `[+0x34]→[+0x24]` socket, `[+0x88]→[+0x20]` map, byte-string
//!   `[+0x90]→[+0x2C]` CD-key, `[+0x2C]→[+0x28]` IPv4; publish через push
//!   helper `0x46AE90`; consume `sub size, declared`; shrink к `0x100000`
//!   при возврате под лимит; reject очищает accumulator без отката
//!   опубликованного;
//! - четыре пути с `AddForbidIP 0x469390` + `QUIT 0x466730`: предел длины,
//!   length CRC (`0x46EA79`), content CRC (`0x46EAD4`), opcode вне диапазона
//!   (`0x46EB1F`, плюс deleting dtor `[edx]` с `push 1`); create-null
//!   без ban — точное отличие прежнего Rust, теперь прямое;
//! - `OnClose` `0x46E660`: при пустом CD-key — НИ публикации, НИ вызова
//!   общего close (`je` сразу в эпилог); при непустом — `new(0x48)`
//!   `CMessage(0x10001)` через ctor `0x465540`, `Add` CD-key с NUL,
//!   publish owner `+0xDC` через `0x46AE90` и общий close `0x46C740(0)` —
//!   странность буквально совпадает с прежним Rust `bool`-ветвлением;
//!   фактическое удаление соединения остаётся общему `CServer`.
//!
//! Owner сохраняет проверки длины/CRC, RLE create-путь, opcode-range,
//! metadata, FIFO-публикацию, неполного TCP-хвоста и synthetic disconnect по
//! непустому CD-key. Небезопасные malformed-границы длины и RLE
//! детерминированно очищают accumulator и возвращают локальную ошибку без
//! воспроизведения UB.
//!
//! Производный конструктор выделял receive-buffer `0x5000` и второй buffer
//! `0xC800`. Первый представлен общей `CServerClient` capacity; второй был
//! техническим send accumulator унаследованного server-client и не получает
//! отдельного дублирующего поля. `Vec` заменяет ручные `new/delete`, realloc и
//! финальный `memmove`, сохраняя порядок кадров и неполный хвост. Превышение
//! длины, обе ошибки CRC и opcode вне диапазона доказанно требуют
//! последовательности component diagnostic -> `AddForbidIP` ->
//! `QuitClientBySocketID`; этот файл классифицирует реакцию, а применит её
//! следующий фактический `CMyNetServer_Client`. Ошибка создания с
//! доказанным нулевым результатом очищает accumulator без ban. Старые
//! `PutDebugString` не превращаются здесь в новый logging API; параметры
//! диагностик сохранены в типизированных ошибках для server-owner’а.
//!
//! При длине с sign bit либо `total_len < 12` x86-путь переходил к signed
//! сравнению и/или unsigned `len - 12`. Как и trailing RLE marker или
//! декодированный header короче 16 bytes, safe Rust отбрасывает такой вход без
//! unsafe-чтения. SEH, deleting-destructor thunks,
//! allocator-копии и ошибочно приписанный World deleting-destructor удалены
//! как compiler/library noise.

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
