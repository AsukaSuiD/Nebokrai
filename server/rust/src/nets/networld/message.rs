//! Сообщение направления Login/GameServer ↔ WorldServer перенесено в Realm.
//! Здесь реэкспорт для переходных потребителей World WorldServer-владельцев.

pub(crate) use nebokrai_realm::app::world_message::{
    CMessage, CreateMessageError, SendMessageError, WorldMessageHandlers,
};
