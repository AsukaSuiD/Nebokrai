//! Входное соединение игрового клиента LoginServer из
//! `nets/netlogin/mynetserverclient_client.cpp`.
//!
//! Статус владельца: `IMPLEMENTED` для constructor state, условных проверок
//! длины/CRC, RLE create-пути, opcode-range, metadata, FIFO-публикации,
//! неполного TCP-хвоста и synthetic disconnect по непустому CD-key.
//! Небезопасные malformed-границы длины и RLE оставлены локальными
//! `BLOCKED_MISSING_FACT`, а не объявлены исходным fail-closed.
//!
//! Точная пара: `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`;
//! SHA-256 EXE
//! `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876`,
//! SHA-256 PDB
//! `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`.
//! Исходный путь PDB:
//! `d:\complite_version\fengyun_russia\trunk\nets\netlogin\mynetserverclient_client.cpp`.
//!
//! Существенные RVA: конструктор `0x0006E590`, деструктор `0x0006E610`,
//! `OnClose` `0x0006E660`, `OnOneMessageSizeOver` `0x0006E730`,
//! `OnTotalMessageSizeOver` `0x0006E7A0`, `OnReceive` `0x0006E810`.
//!
//! Производный конструктор выделял receive-buffer `0x5000` и второй buffer
//! `0xC800`. Первый представлен общей `CServerClient` capacity; второй был
//! техническим send accumulator унаследованного server-client и не получает
//! отдельного дублирующего поля. `Vec` заменяет ручные `new/delete`, realloc и
//! финальный `memmove`, сохраняя порядок кадров и неполный хвост. После разбора
//! общий accumulator уже уменьшает capacity больше `0x100000` обратно до
//! `0x100000`, как исходный класс.
//!
//! Client receive-envelope имеет форму
//! `[total_len, optional crc(total_len), optional crc(rle), rle(message)]`.
//! Обе CRC-проверки независимо включаются setup-полями LoginServer. Предел
//! одного кадра проверяется только вместе с length CRC, до самой CRC и ожидания
//! полного кадра. Content CRC считается по сжатым bytes до `CreateMessage`.
//! Готовое сообщение допустимо только в полном unsigned диапазоне
//! `0x2FD01..=0x3FBFF`, получает socket/map/CD-key/IP соединения и передаётся
//! общей FIFO.
//!
//! Превышение длины, обе ошибки CRC и opcode вне диапазона доказанно требуют
//! последовательности component diagnostic -> `AddForbidIP` ->
//! `QuitClientBySocketID`; этот файл классифицирует реакцию, а применит её
//! следующий фактический `CMyNetServer_Client`. Ошибка создания с доказанным
//! нулевым результатом очищает accumulator без ban. Старые `PutDebugString`
//! не превращаются здесь в новый logging API; параметры диагностик сохранены
//! в типизированных ошибках для будущего владельца.
//!
//! `OnClose` публиковал `0x10001` с NUL-terminated CD-key и вызывал общий close
//! только при непустой строке. Эта странность сохранена буквально; фактическое
//! удаление соединения по-прежнему принадлежит общему `CServer`.
//!
//! При длине с sign bit либо `total_len < 12` x86-путь переходил к signed
//! сравнению и/или unsigned `len - 12`; безопасная наблюдаемая реакция не
//! доказана. Аналогично trailing RLE marker и декодированный header короче 16
//! bytes не получают придуманной реакции. SEH, deleting-destructor thunks,
//! allocator-копии и ошибочно приписанный World deleting-destructor удалены
//! как compiler/library noise.

use crate::nets::basemessage::RleDecodeError;
use crate::nets::msgqueue::CMsgQueue;
use crate::nets::netlogin::message::{CMessage, CreateMessageError};
use crate::nets::serverclient::CServerClient;
use crate::public::crc32static::data_crc32;

const CLIENT_INITIAL_RECEIVE_CAPACITY: usize = 0x5000;
const CLIENT_ENVELOPE_LEN: usize = 12;
const CLIENT_MESSAGE_MIN: u32 = 0x0002_FD01;
const CLIENT_MESSAGE_MAX: u32 = 0x0003_FBFF;
const CLIENT_DISCONNECTED: i32 = 0x0001_0001;

/// Три setup-значения, которые исходный client receive-path читал у сервера.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ClientReceiveSettings {
    check_length_crc: bool,
    check_content_crc: bool,
    maximum_message_length: u32,
}

impl ClientReceiveSettings {
    /// Создаёт буквальный snapshot проверок одного вызова `OnReceive`.
    pub(crate) const fn new(
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
pub(crate) enum ClientReceiveError {
    /// Объявленный размер превысил setup-предел при включённой length-проверке.
    MessageLengthExceeded { declared: u32, permitted: u32 },
    /// CRC little-endian слова `total_len` не совпал со вторым словом envelope.
    LengthChecksumMismatch { expected: u32, actual: u32 },
    /// CRC сжатой части кадра не совпал с третьим словом envelope.
    ContentChecksumMismatch { expected: u32, actual: u32 },
    /// Signed-ветвление оригинала не задаёт безопасную реакцию на эту длину.
    SignedFrameLengthReactionUnknown { declared: u32 },
    /// `total_len < 12` приводит к недоказанному unsigned вычитанию.
    ShortFrameReactionUnknown { declared: u32 },
    /// Конкретный message-owner не смог безопасно создать сообщение.
    Message(CreateMessageError),
    /// Полный тип сообщения находится вне разрешённого client-диапазона.
    OpcodeOutsideAllowedRange { opcode: u32 },
}

impl ClientReceiveError {
    /// Сообщает только о четырёх путях с доказанными `AddForbidIP` и `QUIT`.
    pub(crate) const fn requires_forbid_and_quit(&self) -> bool {
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
pub(crate) struct ClientReceiveOutcome {
    messages: i32,
    pending_bytes: usize,
}

impl ClientReceiveOutcome {
    /// Возвращает число сообщений, переданных в общую FIFO.
    pub(crate) const fn messages(&self) -> i32 {
        self.messages
    }

    /// Возвращает длину сохранённого неполного TCP-хвоста.
    pub(crate) const fn pending_bytes(&self) -> usize {
        self.pending_bytes
    }
}

/// Статический component-owner virtual callbacks старого производного класса.
pub(crate) struct CMyNetServerClientClient;

impl CMyNetServerClientClient {
    /// Создаёт общий client-state с доказанным начальным accumulator `0x5000`.
    pub(crate) fn new_state(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> CServerClient {
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
    pub(crate) fn on_close(client: &mut CServerClient, messages: &CMsgQueue<CMessage>) -> bool {
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
    pub(crate) fn on_receive(
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

            // BLOCKED_MISSING_FACT: Login RVA 0x0006E810 сравнивает
            // `m_nSize < (int)total_len`, а отрицательность проверяет только
            // внутри этой ветки. Для sign-bit при положительном accumulator
            // машинная реакция перед `len - 12` требует точечной проверки.
            if (declared as i32) < 0 {
                return Err(ClientReceiveError::SignedFrameLengthReactionUnknown { declared });
            }

            let frame_length = declared as usize;
            if frame.len() < frame_length {
                break;
            }
            if frame_length < CLIENT_ENVELOPE_LEN {
                // BLOCKED_MISSING_FACT: исходный `total_len - 12` был
                // unsigned и использовался и для CRC, и для CreateMessage.
                return Err(ClientReceiveError::ShortFrameReactionUnknown { declared });
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
                    if is_proven_null_create(&error) {
                        client.discard_receive_data();
                    }
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

fn is_proven_null_create(error: &CreateMessageError) -> bool {
    matches!(
        error,
        CreateMessageError::EmptyInput
            | CreateMessageError::Rle(
                RleDecodeError::EmptyInput | RleDecodeError::OutputCapacityReached
            )
    )
}
