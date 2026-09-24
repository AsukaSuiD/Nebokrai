//! Сеть: framing, RLE и буферы сообщений направлений.
//! Сессия персонажа, разбор смысла команды и момент отправки — у владельца роли.

mod basemessage;

pub use basemessage::{decode_rle, encode_rle, CBaseMessage, RleDecodeError, RleEncodeError};
