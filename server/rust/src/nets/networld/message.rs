//! Сообщение направления Login/GameServer ↔ WorldServer перенесено в Realm.
//! Здесь реэкспорт для переходных потребителей World WorldServer-владельцев.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::world_message::{
    CMessage, SendMessageError, WorldMessageHandlers,
};
