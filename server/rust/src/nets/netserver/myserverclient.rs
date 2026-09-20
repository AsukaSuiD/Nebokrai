//! Принятое игровое client-соединение GameServer из
//! `nets/netserver/myserverclient.cpp`.
//!
//! Статус owner-а: `IMPLEMENTED` для constructor/destructor ownership,
//! `OnClose`, обе limit diagnostics и корректного/неполного `OnReceive`;
//! malformed length/RLE границы остаются локальными `BLOCKED_MISSING_FACT`.
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`;
//! существенные RVA: constructor `0x0001C5C0`, destructor `0x0001C640`,
//! `OnClose` `0x0001C670`, diagnostics `0x0001C710/0x0001C770`, `OnReceive`
//! `0x0001C7F0`.
//!
//! Derived constructor задавал receive capacity `0x5000` и send capacity
//! `0xC800`. Общий `CServerClient` владеет обоими accumulators; `Vec` заменяет
//! ручные allocation/realloc/memmove, сохраняя frames и неполный TCP-хвост.
//! Client envelope имеет форму `[total_len, optional crc(total_len), optional
//! crc(rle), rle(message)]`; обе проверки независимо включаются setup-ом, а
//! предел одного frame проверяется только вместе с length CRC. Допустим полный
//! unsigned opcode range `0x8F701..=0x9F5FF`; готовое сообщение получает
//! socket/map/IP и публикуется в netserver FIFO.
//!
//! Превышение длины, обе ошибки CRC и opcode вне диапазона требуют исходных
//! diagnostic -> forbid IP -> quit; owner классифицирует эту реакцию, а общий
//! `CServer` применяет её после callback. Null create очищает accumulator без
//! ban. `OnClose` только при ненулевом map ID публикует `0x6FA01 + map ID +
//! empty C-string` и вызывает base close; нулевой ID остаётся no-op.
//!
//! Старые внутренности STL, deleting thunk, SEH и allocator unwind не имеют
//! самостоятельной семантики поверх материализованного владения Rust и удалены.

use crate::nets::basemessage::RleDecodeError;
use crate::nets::msgqueue::CMsgQueue;
use crate::nets::serverclient::CServerClient;
use crate::public::crc32static::data_crc32;

use super::message::{CMessage, CreateMessageError};

const CLIENT_INITIAL_RECEIVE_CAPACITY: usize = 0x5000;
const CLIENT_ENVELOPE_LEN: usize = 12;
const CLIENT_MESSAGE_MIN: u32 = 0x0008_F701;
const CLIENT_MESSAGE_MAX: u32 = 0x0009_F5FF;
const CLIENT_DISCONNECTED: i32 = 0x0006_FA01;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ClientReceiveSettings {
    check_length_crc: bool,
    check_content_crc: bool,
    maximum_message_length: u32,
}

impl ClientReceiveSettings {
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ClientReceiveError {
    MessageLengthExceeded { declared: u32, permitted: u32 },
    LengthChecksumMismatch { expected: u32, actual: u32 },
    ContentChecksumMismatch { expected: u32, actual: u32 },
    SignedFrameLengthReactionUnknown { declared: u32 },
    ShortFrameReactionUnknown { declared: u32 },
    Message(CreateMessageError),
    OpcodeOutsideAllowedRange { opcode: u32 },
}

impl ClientReceiveError {
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ClientReceiveOutcome {
    messages: i32,
    pending_bytes: usize,
}

impl ClientReceiveOutcome {
    pub(crate) const fn messages(&self) -> i32 {
        self.messages
    }

    pub(crate) const fn pending_bytes(&self) -> usize {
        self.pending_bytes
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ClientNetworkNotice {
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

pub(crate) struct CMyServerClient;

pub(crate) trait ClientMessageSink {
    fn publish_message(&self, message: CMessage);
}

impl ClientMessageSink for CMsgQueue<CMessage> {
    fn publish_message(&self, message: CMessage) {
        self.push(message);
    }
}

impl CMyServerClient {
    pub(crate) fn new_state(socket_id: i32, peer_ipv4: u32, now_ms: u32) -> CServerClient {
        CServerClient::with_receive_capacity(
            socket_id,
            peer_ipv4,
            now_ms,
            CLIENT_INITIAL_RECEIVE_CAPACITY,
        )
    }

    /// Публикует synthetic disconnect только после назначения player/map ID.
    pub(crate) fn on_close(client: &mut CServerClient, messages: &dyn ClientMessageSink) -> bool {
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

    pub(crate) fn on_receive(
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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\nets\netserver\myserverclient.cpp

// IMPLEMENTED: constructor/destructor ownership, `OnClose`, обе diagnostics
// и `OnReceive` материализованы выше.
// CLASSIFIED_TECHNICAL_NOISE: STL vector/fill internals, deleting thunk, SEH
// и unwind заменены `Vec`, общим `CServerClient` и обычным Rust ownership.

// COMPONENT_VARIANT_END: GameServer
