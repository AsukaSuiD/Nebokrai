//! JiFen-вариант однослотового currency container исторического GameServer:
//! catalog selector `JIFEN` поверх generic core `cwallet`.
//!
//! Player-владелец публикует этот контейнер под extend-id
//! [`PlayerContainerKind::JiFen`][crate::items::playercontainers::PlayerContainerKind]
//! единого каталога (дизайн D4).
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cjifen.cpp`. Общие query, GUID,
//! listener и lifecycle функции code-folded с `CWallet`; catalog selector —
//! `JIFEN`. Отличающийся exact `Add` при пустом контейнере принимает первый
//! `CGoods` без проверки catalog id, а после заполнения разрешает stack только
//! для `JIFEN`. Этот legacy quirk сохранён как marker-policy общего core
//! (`VALIDATE_EMPTY_GOODS = false`), а не исправлен молча. Persisted codec
//! исполняет достигнутый generic owner из `cwallet`; собственная message-граница
//! ещё требует реконструкции.

use super::cwallet::{CSingleCurrencyContainer, CurrencyKind};
use crate::content::goodsfactory::CGoodsFactory;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JiFenCurrency;

impl CurrencyKind for JiFenCurrency {
    const VALIDATE_EMPTY_GOODS: bool = false;

    fn goods_index(factory: &CGoodsFactory) -> u32 {
        factory.get_ji_fen_index()
    }
}

pub type CJiFen = CSingleCurrencyContainer<JiFenCurrency>;
