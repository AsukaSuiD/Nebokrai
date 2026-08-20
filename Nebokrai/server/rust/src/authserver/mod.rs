//! Владельцы исторического процесса AuthServer.

pub(crate) mod appauth;
pub(crate) mod src;

/// `CGame`, связанный с конкретными обработчиками сообщений AuthServer.
pub(crate) type AuthGame = src::cgame::CGame<appauth::message::message_func::AuthMessageHandlers>;
