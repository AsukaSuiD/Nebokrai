//! Изменение количества container-object `0xC0102` старого GameServer
//! перенесено в Zone items волной Z-C5 (build-only кодек полного wire).
//! Здесь реэкспорт для старого пакета; доставка идёт через общий sender-шов
//! `ContainerObjectMessageSender`, реализованный для `CGame` в соседнем shim
//! `cs2ccontainerobjectmove`.

pub(crate) use nebokrai_zone::items::cs2ccontainerobjectamountchange::*;
