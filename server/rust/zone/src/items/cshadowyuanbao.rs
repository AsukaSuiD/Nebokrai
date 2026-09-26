//! YuanBao-специализация однослотовой currency shadow исторического
//! GameServer: catalog `YUANBAO` и source extend `5` поверх generic core
//! `cshadowwallet`.
//!
//! Source-гейт выражен вариантом `PlayerContainerKind::YuanBao` единого
//! каталога `items/playercontainers.rs` (дизайн D4).
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
