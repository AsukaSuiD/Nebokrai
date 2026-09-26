//! Lock-gate `CBank` поверх однослотового `CWallet` GameServer, перенесённый в
//! Zone `items/` — владельца типов контейнеров и операций над ними.
//!
//! Тело перенесено буквально из прежнего
//! `src/gameserver/appserver/container/cbank.rs` (волна Z-C1); отличия —
//! нормализация `pub(crate)`→`pub` на границе crate и швы переноса: owner base —
//! Zone `items/cgoodscontainer.rs`, currency core — Zone `items/cwallet.rs`,
//! `CGoods` — Zone `items/cgoods.rs`, реестр `CGoodsFactory` — Zone
//! `content/goodsfactory.rs` (волна Z-G0b), `CGuid` — Shared. Player-владелец
//! публикует этот контейнер под extend-id
//! [`PlayerContainerKind::Bank`][crate::items::playercontainers::PlayerContainerKind]
//! единого каталога (дизайн D4).
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cbank.cpp`. Constructor, clear и
//! release оставляют bank locked; locked gate скрывает `Find`, positional
//! full/partial `Remove` и обе `Add` перегрузки. Сам password lookup принадлежит
//! player/game owner-у и передаётся сюда уже как результат аутентификации:
//! container сохраняет только подтверждённый state transition. Достигнутый
//! `0x90301` caller переносит
//! owned gold между wallet и bank с rollback и клиентским move. Persisted
//! restore временно открывает внутренний wallet только на время decode и
//! обязательно возвращает bank в locked-состояние, включая ошибочный вход.

use super::cgoodscontainer::CGoodsContainer;
use super::cwallet::{
    CWallet, CurrencyCodecError, CurrencyGoodsAddOutcome, CurrencyGoodsCollected,
    CurrencyGoodsRemoved, CurrencyGoodsTaken,
};
use super::cgoods::CGoods;
use crate::content::goodsfactory::CGoodsFactory;
use nebokrai_shared::values::CGuid;

#[must_use = "locked bank и wallet add имеют разные последующие эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BankGoodsAddOutcome {
    Locked,
    Wallet(CurrencyGoodsAddOutcome),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CBank {
    wallet: CWallet,
    locked: bool,
}

impl Default for CBank {
    fn default() -> Self {
        Self::new()
    }
}

impl CBank {
    pub const fn new() -> Self {
        Self {
            wallet: CWallet::new(),
            locked: true,
        }
    }

    pub const fn base(&self) -> &CGoodsContainer {
        self.wallet.base()
    }

    pub const fn base_mut(&mut self) -> &mut CGoodsContainer {
        self.wallet.base_mut()
    }

    pub const fn is_locked(&self) -> bool {
        self.locked
    }

    /// `GetDepotMoney` читает amount owned gold stack независимо от UI lock.
    pub fn gold_coins_amount(&self) -> u32 {
        self.wallet.currency_amount()
    }

    pub fn find(&self, ex_id: CGuid) -> Option<&CGoods> {
        if self.locked {
            return None;
        }
        self.wallet.find(ex_id)
    }

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        if self.locked {
            return None;
        }
        self.wallet.get_goods(position)
    }

    /// Снимок открытия склада читает денежный предмет после установки
    /// блокировки паролем; эта операция не разрешает последующие изменения.
    pub fn snapshot_goods(&self, position: u32) -> Option<&CGoods> {
        self.wallet.get_goods(position)
    }

    pub fn add_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> BankGoodsAddOutcome {
        if self.locked {
            return BankGoodsAddOutcome::Locked;
        }
        BankGoodsAddOutcome::Wallet(self.wallet.add_goods(
            position,
            incoming,
            factory,
            owner_progress_allows,
        ))
    }

    pub fn remove_goods(&mut self, ex_id: CGuid) -> Option<CurrencyGoodsRemoved> {
        if self.locked {
            return None;
        }
        self.wallet.remove_goods(ex_id)
    }

    pub fn take_goods<Create>(
        &mut self,
        position: u32,
        requested: u32,
        factory: &CGoodsFactory,
        create_goods: Create,
    ) -> Option<CurrencyGoodsTaken>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        if self.locked {
            return None;
        }
        self.wallet
            .take_goods(position, requested, factory, create_goods)
    }

    pub fn lock(&mut self) -> bool {
        self.locked = true;
        true
    }

    /// Player/game owner выполняет exact owner-id lookup и byte-exact password
    /// comparison; неуспех не меняет уже существующее lock-состояние.
    pub fn unlock_if_authenticated(&mut self, authenticated: bool) -> bool {
        if !authenticated {
            return false;
        }
        self.locked = false;
        true
    }

    pub fn clear_goods(&mut self) -> CurrencyGoodsCollected {
        self.locked = true;
        self.wallet.clear_goods()
    }

    pub fn release(&mut self) -> CurrencyGoodsCollected {
        self.locked = true;
        self.wallet.release()
    }

    pub fn serialize(&self, destination: &mut Vec<u8>) -> bool {
        self.wallet.serialize(destination)
    }

    pub fn unserialize<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        factory: &CGoodsFactory,
        ordinary_threshold: OrdinaryThreshold,
        battle_threshold: BattleThreshold,
    ) -> Result<(), CurrencyCodecError>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        self.locked = false;
        let result = self.wallet.unserialize(
            source,
            cursor,
            "CBank marker",
            factory,
            ordinary_threshold,
            battle_threshold,
        );
        self.locked = true;
        result
    }
}
