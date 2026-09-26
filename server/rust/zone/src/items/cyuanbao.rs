//! YuanBao-вариант однослотового currency container GameServer, перенесённый
//! в Zone `items/` — владельца типов контейнеров и операций над ними.
//!
//! Тело перенесено буквально из прежнего
//! `src/gameserver/appserver/container/cyuanbao.rs` (волна Z-C1); отличия —
//! нормализация `pub(crate)`→`pub` на границе crate и швы переноса: generic
//! core — Zone `items/cwallet.rs`, реестр `CGoodsFactory` — Zone
//! `content/goodsfactory.rs` (волна Z-G0b). Player-владелец публикует этот
//! контейнер под extend-id
//! [`PlayerContainerKind::YuanBao`][crate::items::playercontainers::PlayerContainerKind]
//! единого каталога (дизайн D4).
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cyuanbao.cpp`. Layout и порядок
//! операций совпадают с `CWallet`, но допустимый catalog index берётся из
//! `YUANBAO`. Общий storage/lifecycle и достигнутый persisted codec реализованы
//! в `cwallet` marker-адаптером; собственная `CS2CContainerObjectMove` граница
//! ещё требует реконструкции; полный декомпилят хранится локально.

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
