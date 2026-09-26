//! YuanBao-специализация однослотовой currency shadow GameServer, перенесённая
//! в Zone `items/` — владельца типов контейнеров и операций над ними.
//!
//! Тело перенесено буквально из прежнего
//! `src/gameserver/appserver/container/cshadowyuanbao.rs` (волна Z-C3); отличия —
//! нормализация `pub(crate)`→`pub` на границе crate и швы переноса (не
//! расхождения): generic shadow core — Zone `items/cshadowwallet.rs`,
//! `YuanBaoCurrency` — Zone `items/cyuanbao.rs`. Extend-литерал source-гейта
//! выражен вариантом `PlayerContainerKind::YuanBao` единого каталога
//! `items/playercontainers.rs` (дизайн D4).
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cshadowyuanbao.cpp`. Собственные
//! executable отличия от `CShadowWallet` ограничены catalog
//! `YUANBAO` и source extend `5`; storage, partial-mutation order и listener
//! callbacks переиспользуют общий typed adapter.

use super::cshadowwallet::{CShadowCurrencyContainer, ShadowCurrencyKind};
use super::cyuanbao::YuanBaoCurrency;
use super::playercontainers::PlayerContainerKind;

impl ShadowCurrencyKind for YuanBaoCurrency {
    const SOURCE_CONTAINER_EXTEND_ID: i32 = PlayerContainerKind::YuanBao.extend_id();
}

pub type CShadowYuanBao = CShadowCurrencyContainer<YuanBaoCurrency>;
