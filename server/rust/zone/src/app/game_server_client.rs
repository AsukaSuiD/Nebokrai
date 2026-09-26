//! Принятое игровое client-соединение GameServer — состояние принятого игрового
//! клиента game-направления. Исходник `nets/netserver/myserverclient.cpp`;
//! источник контракта — та же точная пара, что у [`crate::app::game_message`].
//!
//! Client envelope имеет форму `[total_len, optional crc(total_len), optional
//! crc(rle), rle(message)]`; обе проверки независимо включаются setup-ом, а
//! предел одного frame проверяется только вместе с length CRC. `Vec` заменяет
//! ручные allocation/realloc/memmove, сохраняя frames и неполный TCP-хвост;
//! готовое сообщение получает socket/map/IP и публикуется в netserver FIFO.
//! Статус owner-а: `IMPLEMENTED` для constructor/destructor ownership, `OnClose`,
//! обеих limit diagnostics и корректного/неполного `OnReceive`; malformed
//! length/RLE границы — локальные `BLOCKED_MISSING_FACT`.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#сетевой-край-gameserver

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
