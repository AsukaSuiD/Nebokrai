//! Сообщение трёх сетевых направлений LoginServer перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::login_message::{
    CMessage, CreateMessageError, LoginMessageHandlers, SendMessageError,
};
