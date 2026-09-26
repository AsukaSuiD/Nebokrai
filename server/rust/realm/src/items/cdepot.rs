//! Запираемый `CDepot` из `cdepot.cpp/.h`.
//!
//! Lock проверяется до Add, Find и Remove. `Clear` и `Release` сначала снимают
//! его; унаследованный volume decoder поэтому оставляет depot разблокированным
//! даже при коротком payload. Сам lock в wire не входит.
//!
//! `AddFromDB` сначала проверяет collision через virtual `GetGoods`, затем lock
//! и base-path. Volume `0xA1` назначается player decoder-ом между `Release` и
//! чтением контейнера, а не конструктором.

use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::items::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};
use crate::content::goodslistener::TraversedGoods;
use nebokrai_shared::values::CGuid;
use crate::content::cgoods::CGoods;

pub struct CDepot {
    volume_state: CVolumeLimitGoodsContainer,
    locked: bool,
}

impl CDepot {
    pub const fn with_constructor_defaults() -> Self {
        Self {
            volume_state: CVolumeLimitGoodsContainer::with_constructor_defaults(),
            locked: false,
        }
    }

    pub fn set_container_volume(&mut self, size: u32) {
        self.locked = false;
        self.volume_state.set_container_volume(size);
    }

    pub const fn get_goods_amount_limit(&self) -> u32 {
        self.volume_state.get_goods_amount_limit()
    }

    pub fn clear(&mut self) {
        self.locked = false;
        self.volume_state.clear();
    }

    pub fn release(&mut self) {
        self.locked = false;
        self.volume_state.release();
    }

    pub fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.locked {
            return Ok(Some(goods));
        }
        self.volume_state.add(goods, registry)
    }

    pub fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.locked {
            return Ok(Some(goods));
        }
        self.volume_state.add_at(position, goods, registry)
    }

    pub fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        if self.locked {
            return None;
        }
        self.volume_state.find(ex_id)
    }

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        if self.locked {
            return None;
        }
        self.volume_state.get_goods(position)
    }

    pub fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        if self.locked {
            return None;
        }
        self.volume_state.get_goods_mut(position)
    }

    pub fn remove(
        &mut self,
        ex_id: &CGuid,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.locked {
            return Ok(None);
        }
        self.volume_state.remove(ex_id, registry)
    }

    pub fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.get_goods(position).is_some() || self.locked {
            return Ok(Some(goods));
        }
        self.volume_state.add_from_db(position, goods, registry)
    }

    pub fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state.is_full(registry)
    }

    pub fn get_goods_amount(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, VolumeContainerCodecError> {
        self.volume_state.get_goods_amount(registry)
    }

    pub fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, crate::content::cgoods::GoodsDbSnapshotBlock> {
        self.volume_state.db_save_entries(registry)
    }

    pub fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state
            .serialize(destination, include_child, registry)
    }

    pub fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.locked = false;
        self.volume_state.unserialize(source, cursor, registry)
    }
}
