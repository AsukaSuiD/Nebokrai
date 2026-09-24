//! Network/send owner GameServer CMyNetServer перенесён в Zone app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_zone::app::game_server::{
    CMyNetServer, GameServerEvent, GameServerEventPublisher,
};
