//! Запираемый `CBank` из `cbank.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Lock проверяется до wallet Add, Find и Remove. `Clear` и `Release` сначала
//! снимают его; wallet decoder поэтому оставляет bank разблокированным даже
//! при коротком marker-wire. Lock не сериализуется.
//!
//! `AddFromDB` проверяет collision до lock. Статически вызванный
//! `CWallet::AddGoldCoinOfLargess` намеренно обходит bank lock. Gold index
//! передаётся уже разрешённым, а Rust-владение заменяет nullable slot.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::cwallet::CWallet;
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;

pub(crate) struct CBank {
    wallet_state: CWallet,
    locked: bool,
}

impl CBank {
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            wallet_state: CWallet::with_constructor_defaults(),
            locked: false,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.locked = false;
        self.wallet_state.clear();
    }

    pub(crate) fn release(&mut self) {
        self.locked = false;
        self.wallet_state.release();
    }

    pub(crate) fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.is_full(registry)
    }

    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        self.wallet_state.get_goods(position)
    }

    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.wallet_state.get_goods_mut(position)
    }

    pub(crate) const fn get_goods_amount(&self) -> u32 {
        self.wallet_state.get_goods_amount()
    }

    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        (!self.locked)
            .then(|| self.wallet_state.find(ex_id))
            .flatten()
    }

    pub(crate) fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        if self.locked {
            return None;
        }
        self.wallet_state.remove(ex_id)
    }

    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        gold_coin_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        if self.locked {
            return Ok(Some(goods));
        }
        self.wallet_state.add(goods, gold_coin_index, registry)
    }

    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        gold_coin_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        if self.locked {
            return Ok(Some(goods));
        }
        self.wallet_state
            .add_at(position, goods, gold_coin_index, registry)
    }

    pub(crate) fn add_from_db(&mut self, position: u32, goods: Box<CGoods>) -> Option<Box<CGoods>> {
        if self.wallet_state.get_goods(position).is_some() || self.locked {
            return Some(goods);
        }
        self.wallet_state.add_from_db(position, goods)
    }

    pub(crate) fn add_gold_coin_of_largess(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        gold_coin_limit: u32,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state
            .add_gold_coin_of_largess(position, goods, gold_coin_limit)
    }

    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.wallet_state.db_save_entries(registry)
    }

    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.serialize(destination, include_child)
    }

    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.locked = false;
        self.wallet_state.unserialize_with_marker_field(
            source,
            cursor,
            include_child,
            registry,
            "CBank marker",
        )
    }
}
