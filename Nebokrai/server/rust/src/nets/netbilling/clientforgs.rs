//! Принятое GameServer-соединение BillingServer из
//! `nets/netbilling/clientforgs.cpp`.
//!
//! Статус владельца: `IMPLEMENTED` для constructor state, `OnClose` и
//! корректного/неполного `OnReceive`. Небезопасные malformed-границы длины и
//! короткого внутреннего header детерминированно очищают accumulator и
//! возвращают локальную ошибку. `SetSendRevBuf` остаётся отдельной
//! transport-границей до доказательства совместимого Linux socket option.
//!
//! Точная пара: `BillingServer/billingserver.exe + BillingServer/billingserver.pdb`;
//! SHA-256 EXE
//! `FA32E3C043CB49965686129696A4EB34B733ACA1D60CAF57D369F97D5E68FB19`,
//! SHA-256 PDB
//! `F900CD0330BEFF32AC071B107AB653FD403CD18746896B3C0187C5751ACA0B21`.
//! Исходный путь PDB:
//! `h:\fengyun\fy_russia\src\nets\netbilling\clientforgs.cpp`.
//!
//! Существенные RVA: конструктор `0x0000F180`, деструктор `0x0000F200`,
//! `OnClose` `0x0000F230`, `SetSendRevBuf` `0x0000F2D0` и
//! `OnReceive` `0x0000F320`.
//!
//! Производный конструктор выделял receive-buffer `0xA00000` и send-buffer
//! `0x100000`. Rust использует общий `CServerClient` с Billing-capacity для
//! receive; его owned send accumulator растёт сам и не получает второго
//! component-поля. `Vec::drain` заменяет ручные realloc/memmove, сохраняя
//! порядок кадров и неполный TCP-хвост. После разобранного кадра общий owner
//! уже уменьшает capacity больше `0x100000` обратно до `0x100000`.
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
//! IPv4 с NUL и унаследованный `CMySocket` port. Accept-path заменял IP через
//! `inet_ntoa`, но не менял constructor port `5000`; `Ipv4Addr` и
//! `DEFAULT_PORT` воспроизводят эти значения без WinSock. Затем устанавливается
//! общий close-state. Socket ID, CD-key и числовой IP сообщению не присваиваются.
//!
//! `SetSendRevBuf` передавал ноль в Windows `SO_SNDBUF`. На Linux нулевое
//! значение не гарантирует ту же семантику и обычно преобразуется ядром в
//! минимальный buffer, поэтому похожий `setsockopt` не придуман до достижения
//! фактического server/transport-owner.
//!
//! Для длины с sign bit либо `total_len < 12` x86-путь достигал signed
//! сравнения и/или unsigned `len - 12`; внутреннее сообщение длиной 1..15
//! bytes также читало header за границей. Safe Rust отбрасывает весь текущий
//! accumulator без воспроизведения UB. SEH, allocator-копии,
//! security cookie и деструкторные механизмы удалены как compiler/library
//! noise; освобождение обоих buffers выражено владением и `Drop`.

use std::net::Ipv4Addr;

use crate::nets::msgqueue::CMsgQueue;
use crate::nets::mysocket::DEFAULT_PORT;
use crate::nets::netbilling::message::{CMessage, CreateMessageError};
use crate::nets::serverclient::CServerClient;
use crate::public::crc32static::data_crc32;

const BILLING_INITIAL_RECEIVE_CAPACITY: usize = 0xA0_0000;
const SERVER_ENVELOPE_LEN: usize = 12;
const GAME_SERVER_DISCONNECTED: i32 = 0x0010_EF01;

/// Причина остановки Billing receive-path на текущем accumulator.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum BillingReceiveError {
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
pub(crate) struct BillingReceiveOutcome {
    messages: i32,
    pending_bytes: usize,
}

impl BillingReceiveOutcome {
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
pub(crate) struct CClientForGS;

impl CClientForGS {
    /// Создаёт общий client-state с доказанным Billing accumulator `0xA00000`.
    pub(crate) fn new_state(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> CServerClient {
        CServerClient::with_receive_capacity(
            socket_id,
            peer_ipv4,
            now_ms,
            BILLING_INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Публикует unconditional `0x10EF01` и устанавливает общий close-state.
    pub(crate) fn on_close(client: &mut CServerClient, messages: &CMsgQueue<CMessage>) {
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
    pub(crate) fn on_receive(
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

// BLOCKED_MISSING_FACT: `SetSendRevBuf` RVA `0x0000F2D0` задавал Windows
// `SO_SNDBUF=0`. Нужно доказать требуемую наблюдаемую семантику backpressure,
// прежде чем выбирать отличающийся Linux socket option в server-owner.
