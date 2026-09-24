//! Сеть: framing, RLE и буферы сообщений направлений.
//! Сессия персонажа, разбор смысла команды и момент отправки — у владельца роли.

mod basemessage;
mod msgqueue;
mod mysocket;
mod socketcommands;

pub use basemessage::{decode_rle, encode_rle, CBaseMessage, RleDecodeError, RleEncodeError};
pub use msgqueue::CMsgQueue;
pub use mysocket::{
    legacy_bind_endpoint, legacy_inet_addr, legacy_ipv4_word, SocketIdAllocator, DEFAULT_IP,
    DEFAULT_PORT, DEFAULT_SOCKET_TYPE,
};
pub use socketcommands::CSocketCommands;
