//! Исходящее World/Billing-направление GameServer перенесено в Zone app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_zone::app::game_client::{
    CMyNetClient, GameClientIoError, GameClientIoStep, ServerType,
};
