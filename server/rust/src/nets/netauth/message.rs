//! Сообщение направления LoginServer → AuthServer перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::auth_message::{
    AuthMessageHandler, AuthMessageKind, CMessage, DispatchError, SendMessageError,
};
