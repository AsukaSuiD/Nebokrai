//! YuanBao-специализация однослотовой currency shadow GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cshadowyuanbao.cpp`. Собственные
//! executable отличия от `CShadowWallet` ограничены catalog
//! `YUANBAO` и source extend `5`; storage, partial-mutation order и listener
//! callbacks переиспользуют общий typed adapter.

use super::cshadowwallet::{CShadowCurrencyContainer, ShadowCurrencyKind};
use super::cyuanbao::YuanBaoCurrency;

impl ShadowCurrencyKind for YuanBaoCurrency {
    const SOURCE_CONTAINER_EXTEND_ID: i32 = 5;
}

pub(crate) type CShadowYuanBao = CShadowCurrencyContainer<YuanBaoCurrency>;
