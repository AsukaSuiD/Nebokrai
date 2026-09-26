//! YuanBao-вариант однослотового currency container исторического GameServer:
//! catalog selector `YUANBAO` поверх generic core `cwallet` marker-адаптера.
//!
//! Player-владелец публикует этот контейнер под extend-id
//! [`PlayerContainerKind::YuanBao`][crate::items::playercontainers::PlayerContainerKind]
//! единого каталога (дизайн D4).
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cyuanbao.cpp`. Layout и порядок
//! операций совпадают с `CWallet`, но допустимый catalog index берётся из
//! `YUANBAO`. Общий storage/lifecycle и достигнутый persisted codec реализованы
//! в `cwallet` marker-адаптером; собственная `CS2CContainerObjectMove` граница
//! ещё требует реконструкции.

use super::cwallet::{CSingleCurrencyContainer, CurrencyKind};
use crate::content::goodsfactory::CGoodsFactory;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct YuanBaoCurrency;

impl CurrencyKind for YuanBaoCurrency {
    const VALIDATE_EMPTY_GOODS: bool = true;

    fn goods_index(factory: &CGoodsFactory) -> u32 {
        factory.get_yuan_bao_index()
    }
}

pub type CYuanBao = CSingleCurrencyContainer<YuanBaoCurrency>;
