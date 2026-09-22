//! Запираемый `CDepot` из `cdepot.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Lock проверяется до Add, Find и Remove. `Clear` и `Release` сначала снимают
//! его; унаследованный volume decoder поэтому оставляет depot разблокированным
//! даже при коротком payload. Сам lock в wire не входит.
//!
//! `AddFromDB` сначала проверяет collision через virtual `GetGoods`, затем lock
//! и base-path. Volume `0xA1` назначается player decoder-ом между `Release` и
//! чтением контейнера, а не конструктором.

use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use nebokrai_shared::values::CGuid;
use crate::worldserver::appworld::goods::cgoods::CGoods;

pub(crate) struct CDepot {
    volume_state: CVolumeLimitGoodsContainer,
    locked: bool,
}

impl CDepot {
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            volume_state: CVolumeLimitGoodsContainer::with_constructor_defaults(),
            locked: false,
        }
    }

    pub(crate) fn set_container_volume(&mut self, size: u32) {
        self.locked = false;
        self.volume_state.set_container_volume(size);
    }

    pub(crate) const fn get_goods_amount_limit(&self) -> u32 {
        self.volume_state.get_goods_amount_limit()
    }

    pub(crate) fn clear(&mut self) {
        self.locked = false;
        self.volume_state.clear();
    }

    pub(crate) fn release(&mut self) {
        self.locked = false;
        self.volume_state.release();
    }

    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.locked {
            return Ok(Some(goods));
        }
        self.volume_state.add(goods, registry)
    }

    pub(crate) fn add_at(
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

    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        if self.locked {
            return None;
        }
        self.volume_state.find(ex_id)
    }

    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        if self.locked {
            return None;
        }
        self.volume_state.get_goods(position)
    }

    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        if self.locked {
            return None;
        }
        self.volume_state.get_goods_mut(position)
    }

    pub(crate) fn remove(
        &mut self,
        ex_id: &CGuid,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.locked {
            return Ok(None);
        }
        self.volume_state.remove(ex_id, registry)
    }

    pub(crate) fn add_from_db(
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

    pub(crate) fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state.is_full(registry)
    }

    pub(crate) fn get_goods_amount(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, VolumeContainerCodecError> {
        self.volume_state.get_goods_amount(registry)
    }

    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.volume_state.db_save_entries(registry)
    }

    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state
            .serialize(destination, include_child, registry)
    }

    pub(crate) fn unserialize(
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
