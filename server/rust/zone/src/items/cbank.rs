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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp

// ============================================================================
// FUNCTION: CBank::CBank
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:16
// RVA: 0x000FD260
// ADDRESS: 004fd260
// PROTOTYPE: undefined __thiscall CBank(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:36
// RVA: 0x000FD280
// ADDRESS: 004fd280
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:48
// RVA: 0x000FD290
// ADDRESS: 004fd290
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:62
// RVA: 0x000FD2A0
// ADDRESS: 004fd2a0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:75
// RVA: 0x000FD2B0
// ADDRESS: 004fd2b0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Lock
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:88
// RVA: 0x000FD2C0
// ADDRESS: 004fd2c0
// PROTOTYPE: int __thiscall Lock(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:119
// RVA: 0x000FD2D0
// ADDRESS: 004fd2d0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:130
// RVA: 0x000FD2E0
// ADDRESS: 004fd2e0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:138
// RVA: 0x000FD2F0
// ADDRESS: 004fd2f0
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::~CBank
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:29
// RVA: 0x000FD330
// ADDRESS: 004fd330
// PROTOTYPE: void __thiscall ~CBank(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Unlock
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp:97
// RVA: 0x000FD3B0
// ADDRESS: 004fd3b0
// PROTOTYPE: int __thiscall Unlock(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::tagCompose::~tagCompose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbank.cpp
// RVA: 0x000FDC30
// ADDRESS: 004fdc30
// PROTOTYPE: void __thiscall ~tagCompose(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
