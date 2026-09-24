//! Принятое игровое client-соединение GameServer из `nets/netserver/myserverclient.cpp`,
//! перенесённое в Zone app — состояние принятого игрового клиента
//! game-направления. Источник контракта — та же точная пара, что у
//! [`crate::app::game_message`].
//!
//! Машинно подтверждённые точки (первая секция `.exe/gameserver.exe`,
//! дизассемблер этого прохода):
//! - ctor `CMyServerClient` `0x41C5C0`: base `0x41B850`, vtable `0x64D6EC`,
//!   receive capacity ровно `0x5000` (`push 0x5000` + `[+0x64] = 0x5000`),
//!   два компаньона `0xC800` в `+0x70/+0x74` (исходный send accumulator) и
//!   объект `0x2C` сообщения во всех synthetic вызовах;
//! - `OnReceive` `0x41C7F0`: gate owner `+0xA8`, цикл, пока накоплено `>= 0xC`;
//!   флаг `[owner+0x10C]`: предел полной длины (`declared > [owner+0x118]`) и
//!   CRC длины через общий `DataCrc32 0x47B0A0`, `declared > size` — останов
//!   без потери хвоста; флаг `[owner+0x10D]`: CRC по сжатым байтам
//!   `[+0xC, declared-0xC]`; create только RLE `0x413700`; opcode допустим в
//!   `(0x8F700, 0x9F600)` exclusive — точное `0x8F701..=0x9F5FF`; контекст
//!   `[+0x34]→[+0x24]` socket, `[+0x88]→[+0x20]` map, `[+0x2C]→[+0x28]` IPv4
//!   (CD-key у Game-получателя не присваивается); publish через push helper
//!   `0x4126D0`; consume `sub size, declared`; shrink к `0x100000` при
//!   возврате под лимит; reject очищает accumulator без отката опубликованного;
//! - доказаны ровно четыре пути с `AddForbidIP 0x417960` + `QUIT 0x415080`:
//!   предел длины (с virtual `OnTotalMessageSizeOver [+0x40]`, `0x41C9E3`),
//!   length CRC (`0x41CA19`), content CRC (`0x41CA49`), opcode вне диапазона
//!   (`0x41CA93` c deleting dtor `[edx]` с `push 1`); create-null — без ban
//!   (`0x41CABE`: очистка size и sprintf-log без forbid);
//! - `OnClose` `0x41C670`: при нулевом map identity — ни публикации, ни
//!   общего close (`je` сразу в эпилог); при ненулевом — `new(0x2C)`
//!   `CMessage(0x6FA01)` через ctor `0x4136D0`, `Add` map identity, `Add` общего
//!   writer-ом empty cstring `0x64BB41`, publish через `0x4126D0` и общий close
//!   `0x41AD10(0)` — точное совпадение прежнего `bool`-ветвления.
//!
//! Статус owner-а: `IMPLEMENTED` для constructor/destructor ownership,
//! `OnClose`, обе limit diagnostics и корректного/неполного `OnReceive`;
//! malformed length/RLE границы остаются локальными `BLOCKED_MISSING_FACT`.
//!
//! Общий `CServerClient` владеет обоими accumulators; `Vec` заменяет ручные
//! allocation/realloc/memmove, сохраняя frames и неполный TCP-хвост. Client
//! envelope имеет форму `[total_len, optional crc(total_len), optional
//! crc(rle), rle(message)]`; обе проверки независимо включаются setup-ом, а
//! предел одного frame проверяется только вместе с length CRC. Готовое
//! сообщение получает socket/map/IP и публикуется в netserver FIFO.
//!
//! Старые внутренности STL, deleting thunk, SEH и allocator unwind не имеют
//! самостоятельной семантики поверх материализованного владения Rust и удалены.

use nebokrai_shared::network::{CMsgQueue, CServerClient, RleDecodeError};
use nebokrai_shared::protocol::data_crc32;

use super::game_message::{CMessage, CreateMessageError};

const CLIENT_INITIAL_RECEIVE_CAPACITY: usize = 0x5000;
const CLIENT_ENVELOPE_LEN: usize = 12;
const CLIENT_MESSAGE_MIN: u32 = 0x0008_F701;
const CLIENT_MESSAGE_MAX: u32 = 0x0009_F5FF;
const CLIENT_DISCONNECTED: i32 = 0x0006_FA01;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientReceiveSettings {
    check_length_crc: bool,
    check_content_crc: bool,
    maximum_message_length: u32,
}

impl ClientReceiveSettings {
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

#[derive(Debug, Eq, PartialEq)]
pub enum ClientReceiveError {
    MessageLengthExceeded { declared: u32, permitted: u32 },
    LengthChecksumMismatch { expected: u32, actual: u32 },
    ContentChecksumMismatch { expected: u32, actual: u32 },
    SignedFrameLengthReactionUnknown { declared: u32 },
    ShortFrameReactionUnknown { declared: u32 },
    Message(CreateMessageError),
    OpcodeOutsideAllowedRange { opcode: u32 },
}

impl ClientReceiveError {
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

#[derive(Debug, Eq, PartialEq)]
pub struct ClientReceiveOutcome {
    messages: i32,
    pending_bytes: usize,
}

impl ClientReceiveOutcome {
    pub const fn messages(&self) -> i32 {
        self.messages
    }

    pub const fn pending_bytes(&self) -> usize {
        self.pending_bytes
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ClientNetworkNotice {
    OneMessageSizeExceeded {
        player_id: i32,
        peer_ipv4: u32,
        declared: i32,
        permitted: i32,
    },
    TotalMessageSizeExceeded {
        player_id: i32,
        peer_ipv4: u32,
        actual: i32,
        permitted: i32,
    },
}

pub struct CMyServerClient;

pub trait ClientMessageSink {
    fn publish_message(&self, message: CMessage);
}

impl ClientMessageSink for CMsgQueue<CMessage> {
    fn publish_message(&self, message: CMessage) {
        self.push(message);
    }
}

impl CMyServerClient {
    pub fn new_state(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> CServerClient {
        CServerClient::with_receive_capacity(
            socket_id,
            peer_ipv4,
            now_ms,
            CLIENT_INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Публикует synthetic disconnect только после назначения player/map ID.
    pub fn on_close(client: &mut CServerClient, messages: &dyn ClientMessageSink) -> bool {
        let map_id = client.message_context().map_id;
        if map_id == 0 {
            return false;
        }
        let mut message = CMessage::new(CLIENT_DISCONNECTED);
        message.base_mut().add_long(map_id);
        message.base_mut().add_byte(0);
        messages.publish_message(message);
        client.mark_closing();
        true
    }

    pub fn on_receive(
        client: &mut CServerClient,
        settings: ClientReceiveSettings,
        messages: &dyn ClientMessageSink,
    ) -> Result<ClientReceiveOutcome, ClientReceiveError> {
        let mut produced = 0_i32;
        while client.pending_receive_bytes() >= CLIENT_ENVELOPE_LEN {
            let frame = client.receive_bytes();
            let declared_bytes: [u8; 4] = frame[..4]
                .try_into()
                .expect("наличие полного Game client envelope уже проверено");
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
                        .expect("наличие полного Game client envelope уже проверено"),
                );
                let actual = data_crc32(&declared_bytes);
                if actual != expected {
                    return Err(ClientReceiveError::LengthChecksumMismatch { expected, actual });
                }
            }

            if (declared as i32) < 0 {
                // BLOCKED_MISSING_FACT: signed compare пропускает sign-bit к
                // unsigned `total_len - 12`; safe реакция не доказана.
                return Err(ClientReceiveError::SignedFrameLengthReactionUnknown { declared });
            }
            let frame_length = declared as usize;
            if frame.len() < frame_length {
                break;
            }
            if frame_length < CLIENT_ENVELOPE_LEN {
                return Err(ClientReceiveError::ShortFrameReactionUnknown { declared });
            }

            let compressed = &frame[CLIENT_ENVELOPE_LEN..frame_length];
            if settings.check_content_crc {
                let expected = u32::from_le_bytes(
                    frame[8..12]
                        .try_into()
                        .expect("наличие полного Game client envelope уже проверено"),
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
            messages.publish_message(message);
            produced = produced.wrapping_add(1);
            let consumed = client.consume_receive_prefix(frame_length);
            debug_assert!(consumed, "полный Game client frame уже проверен");
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
