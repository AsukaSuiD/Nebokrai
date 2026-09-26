//! Однослотовый `CJiFen` из `cjifen.cpp/.h`.
//!
//! Wire совпадает с `CWallet`: marker `0/1` и полный `CGoods`; decoder сначала
//! освобождает прежний slot. `Clear` сохраняет owner, `Release` обнуляет его.
//! GUID-query сравнивает идентификатор, а не адрес объекта.
//!
//! Пустой slot принимает товар без проверки, занятый складывает только JiFen;
//! индекс передаётся уже разрешённым из StringTable `WS0110`. Rust-владение
//! устраняет внутренние утечки, не меняя состояние и wire.

use crate::content::cgoods::{CGoods, GoodsCodecError};
use crate::content::cgoodsfactory::unserialize_goods;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::goodslistener::TraversedGoods;
use crate::items::ccontainer::{ContainerGuidStorage, find_by_object_guid};
use crate::items::ccontainerlistener::{CContainerListener, TraversedContainerObject};
use crate::items::cwallet::CWallet;
use nebokrai_shared::values::CGuid;

pub struct CJiFen {
    wallet_state: CWallet,
}

impl CJiFen {
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

    pub fn is_goods_existed(&self, base_properties_index: u32, ji_fen_index: u32) -> bool {
        self.wallet_state
            .is_goods_existed(base_properties_index, ji_fen_index)
    }

    pub fn get_the_first_goods(
        &self,
        base_properties_index: u32,
        ji_fen_index: u32,
    ) -> Option<&CGoods> {
        self.wallet_state
            .get_the_first_goods(base_properties_index, ji_fen_index)
    }

    pub fn get_goods_by_base_index(
        &self,
        base_properties_index: u32,
        ji_fen_index: u32,
    ) -> impl Iterator<Item = &CGoods> {
        self.wallet_state
            .get_goods_by_base_index(base_properties_index, ji_fen_index)
    }

    pub fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        ji_fen_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state
            .add_at(position, goods, ji_fen_index, registry)
    }

    pub fn add(
        &mut self,
        goods: Box<CGoods>,
        ji_fen_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state.add(goods, ji_fen_index, registry)
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
            "CJiFen marker",
        )
    }
}

impl CWallet {
    pub fn clear(&mut self) {
        self.gold_coins = None;
    }

    pub fn release(&mut self) {
        self.container_base.release();
        self.gold_coins = None;
    }

    pub fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        match &self.gold_coins {
            Some(goods) => Ok(goods.get_max_stack_number(registry)? <= goods.get_amount()),
            None => Ok(false),
        }
    }

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        (position == 0)
            .then_some(self.gold_coins.as_deref())
            .flatten()
    }

    pub fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        (position == 0)
            .then_some(self.gold_coins.as_deref_mut())
            .flatten()
    }

    pub const fn get_goods_amount(&self) -> u32 {
        self.gold_coins.is_some() as u32
    }

    pub fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        <Self as ContainerGuidStorage>::find_by_guid(self, ex_id)
    }

    pub fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        <Self as ContainerGuidStorage>::remove_by_guid(self, ex_id)
    }

    pub fn query_goods_position_by_object(&self, goods: Option<&CGoods>) -> Option<u32> {
        find_by_object_guid(self, goods.map(CGoods::get_ex_id)).map(|_| 0)
    }

    pub fn query_goods_position(&self, ex_id: &CGuid) -> Option<u32> {
        self.find(ex_id).map(|_| 0)
    }

    pub fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        let (Some(listener), Some(goods)) = (listener, self.gold_coins.as_deref()) else {
            return;
        };
        let _ = listener.on_traversing_container(TraversedContainerObject::Goods(goods));
    }

    pub fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        let Some(goods) = &self.gold_coins else {
            destination.push(0);
            return Ok(true);
        };
        destination.push(1);
        let _ = goods.serialize(destination, include_child)?;
        Ok(true)
    }

    pub fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.unserialize_with_marker_field(
            source,
            cursor,
            _include_child,
            registry,
            "CWallet marker",
        )
    }

    pub fn unserialize_with_marker_field(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
        marker_field: &'static str,
    ) -> Result<bool, GoodsCodecError> {
        self.release();
        let offset = *cursor;
        let Some(&marker) = source.get(offset) else {
 // Старый владелец не получал длину источника. При коротком буфере
 // он выходил за его границы; Rust не воспроизводит это UB.
            return Err(GoodsCodecError::UnexpectedEnd {
                field: marker_field,
                offset,
                needed: 1,
                available: source.len().saturating_sub(offset),
            });
        };
        *cursor = offset + 1;
        if marker != 0 {
            self.gold_coins = unserialize_goods(source, cursor, registry)?;
        }
        Ok(true)
    }
}
