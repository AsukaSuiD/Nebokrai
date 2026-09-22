//! `CBattleFairyContainer` из `cbattlefairycontainer.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Класс не добавляет полей и делегирует wire, Add/find/remove и lifecycle
//! volume-owner-у. Ранний `Clear`, cursor и частично добавленные товары
//! сохраняются при поздней ошибке.
//!
//! Volume `0x11` назначается player decoder-ом между `Release` и decode.
//! `AddFromDB` выполняет derived collision lookup, после чего base повторяет
//! проверку и вставляет товар напрямую.

use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use nebokrai_shared::values::CGuid;
use crate::worldserver::appworld::goods::cgoods::CGoods;

pub(crate) struct CBattleFairyContainer {
    volume_state: CVolumeLimitGoodsContainer,
}

impl CBattleFairyContainer {
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            volume_state: CVolumeLimitGoodsContainer::with_constructor_defaults(),
        }
    }

    pub(crate) fn set_container_volume(&mut self, size: u32) {
        self.volume_state.set_container_volume(size);
    }

    pub(crate) fn clear(&mut self) {
        self.volume_state.clear();
    }

    pub(crate) fn release(&mut self) {
        self.volume_state.release();
    }

    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.add(goods, registry)
    }

    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.add_at(position, goods, registry)
    }

    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        self.volume_state.find(ex_id)
    }

    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.volume_state.get_goods_mut(position)
    }

    pub(crate) fn remove(
        &mut self,
        ex_id: &CGuid,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.remove(ex_id, registry)
    }

    pub(crate) fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.volume_state.get_goods(position).is_some() {
            return Ok(Some(goods));
        }
        self.volume_state.add_from_db(position, goods, registry)
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
        self.volume_state.unserialize(source, cursor, registry)
    }
}
