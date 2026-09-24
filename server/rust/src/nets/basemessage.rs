//! Базовый 16-байтовый wire-буфер сообщений перенесён в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::{
    decode_rle, encode_rle, CBaseMessage, RleDecodeError, RleEncodeError,
};
