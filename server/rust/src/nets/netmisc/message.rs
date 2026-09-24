//! Сообщение направления MiscServer перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::misc_message::{
    CMessage, MessageHandlers, MessageSender, SendMessageError,
};
