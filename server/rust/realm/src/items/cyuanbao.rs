//! Однослотовый `CYuanBao` из `cyuanbao.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`. Перенесён в Realm `items/`.
//!
//! Это отдельный nominal-класс с тем же состоянием и marker-wire, что `CWallet`.
//! Пустой slot принимает товар без проверки, занятый складывает только YuanBao;
//! индекс валюты передаётся уже разрешённым из StringTable `WS0109`.
//! `Option<Box<CGoods>>` заменяет nullable pointer и ручное удаление. Бесполезный
//! vector-by-value query представлен обычным iterator-ом с тем же фильтром.

use crate::content::cgoods::{CGoods, GoodsCodecError};
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::items::ccontainerlistener::CContainerListener;
use crate::items::cwallet::CWallet;
use crate::content::goodslistener::TraversedGoods;
use nebokrai_shared::values::CGuid;

pub struct CYuanBao {
    wallet_state: CWallet,
}

impl CYuanBao {
    pub const fn with_constructor_defaults() -> Self {
        Self {
            wallet_state: CWallet::with_constructor_defaults(),
        }
    }

    pub fn clear(&mut self) {
        self.wallet_state.clear();
    }

    pub fn release(&mut self) {
        self.wallet_state.release();
    }

    pub fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.is_full(registry)
    }

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        self.wallet_state.get_goods(position)
    }

    pub fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.wallet_state.get_goods_mut(position)
    }

    pub const fn get_goods_amount(&self) -> u32 {
        self.wallet_state.get_goods_amount()
    }

    pub fn is_goods_existed(&self, base_properties_index: u32, yuan_bao_index: u32) -> bool {
        self.wallet_state
            .is_goods_existed(base_properties_index, yuan_bao_index)
    }

    pub fn get_the_first_goods(
        &self,
        base_properties_index: u32,
        yuan_bao_index: u32,
    ) -> Option<&CGoods> {
        self.wallet_state
            .get_the_first_goods(base_properties_index, yuan_bao_index)
    }

    pub fn get_goods_by_base_index(
        &self,
        base_properties_index: u32,
        yuan_bao_index: u32,
    ) -> impl Iterator<Item = &CGoods> {
        self.wallet_state
            .get_goods_by_base_index(base_properties_index, yuan_bao_index)
    }

    pub fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        yuan_bao_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state
            .add_at(position, goods, yuan_bao_index, registry)
    }

    pub fn add(
        &mut self,
        goods: Box<CGoods>,
        yuan_bao_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state.add(goods, yuan_bao_index, registry)
    }

    pub fn add_from_db(&mut self, position: u32, goods: Box<CGoods>) -> Option<Box<CGoods>> {
        self.wallet_state.add_from_db(position, goods)
    }

    pub fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        self.wallet_state.find(ex_id)
    }

    pub fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        self.wallet_state.remove(ex_id)
    }

    pub fn query_goods_position_by_object(&self, goods: Option<&CGoods>) -> Option<u32> {
        self.wallet_state.query_goods_position_by_object(goods)
    }

    pub fn query_goods_position(&self, ex_id: &CGuid) -> Option<u32> {
        self.wallet_state.query_goods_position(ex_id)
    }

    pub fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        self.wallet_state.traversing_container(listener);
    }

    pub fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, crate::content::cgoods::GoodsDbSnapshotBlock> {
        self.wallet_state.db_save_entries(registry)
    }

    pub fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.serialize(destination, include_child)
    }

    pub fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.unserialize_with_marker_field(
            source,
            cursor,
            include_child,
            registry,
            "CYuanBao marker",
        )
    }
}
