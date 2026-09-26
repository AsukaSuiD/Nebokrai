//! Общий реестр игровых сессий `CSessionFactory` старого GameServer перенесён
//! в Zone `interactions/` волной Z-C6; coherent-impl шва `AroundSessionLookup`
//! живёт у трейта в Zone `replication/around` — обёртка владения под этот impl
//! ликвидирована. Здесь остаётся реэкспорт реестра и отчётных типов для
//! старого пакета, все call-site-ы через поле и accessor-ы `CGame` компилируются
//! тем же путём. Разрешение player-owner-ов session (`session_player_ids`)
//! остаётся в Zone-реестре и доходит до кадров `0xC0101/0xC0102` через hub
//! `ContainerObjectMessageSender` у `CGame`.

pub(crate) use nebokrai_zone::interactions::csessionfactory::*;
