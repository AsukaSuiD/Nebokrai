//! `CFairyContainer` из `cfairycontainer.cpp/.h`.
//!
//! Контейнер расширяет volume-owner пятью hatch-time. Serialize после каждого
//! значения обнуляет его в live-state; decoder сначала разбирает volume-wire,
//! затем последовательно присваивает пять `u32`. Поздняя ошибка сохраняет
//! очищенный контейнер, cursor и уже прочитанные hatch-time.
//!
//! `Clear`, `Release` и смена volume не меняют hatch-time. Add/find/remove
//! совпадают с battle-fairy volume-политикой; `AddFromDB` сохраняет отдельную
//! проверку collision перед base-вставкой.

use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::items::camountlimitgoodscontainer::AmountContainerCodecError;
use crate::items::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};
use crate::content::goodslistener::TraversedGoods;
use nebokrai_shared::values::CGuid;
use crate::content::cgoods::CGoods;

const HATCH_TIME_COUNT: usize = 5;

pub struct CFairyContainer {
    volume_state: CVolumeLimitGoodsContainer,
    hatch_times: [u32; HATCH_TIME_COUNT],
}

impl CFairyContainer {
    pub const fn with_constructor_defaults() -> Self {
        Self {
            volume_state: CVolumeLimitGoodsContainer::with_constructor_defaults(),
            hatch_times: [0; HATCH_TIME_COUNT],
        }
    }

    pub fn set_container_volume(&mut self, size: u32) {
        self.volume_state.set_container_volume(size);
    }

    pub fn clear(&mut self) {
        self.volume_state.clear();
    }

    pub fn release(&mut self) {
        self.volume_state.release();
    }

    pub fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.add(goods, registry)
    }

    pub fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.add_at(position, goods, registry)
    }

    pub fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        self.volume_state.find(ex_id)
    }

    pub fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.volume_state.get_goods_mut(position)
    }

    pub fn remove(
        &mut self,
        ex_id: &CGuid,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.remove(ex_id, registry)
    }

    pub fn add_from_db(
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

    pub fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, crate::content::cgoods::GoodsDbSnapshotBlock> {
        self.volume_state.db_save_entries(registry)
    }

    pub fn serialize(
        &mut self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        let base_result = self
            .volume_state
            .serialize(destination, include_child, registry)?;
        for hatch_time in &mut self.hatch_times {
            destination.extend_from_slice(&hatch_time.to_le_bytes());
            *hatch_time = 0;
        }
        Ok(base_result)
    }

    pub fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        let base_result = self.volume_state.unserialize(source, cursor, registry)?;
        for index in 0..HATCH_TIME_COUNT {
            self.hatch_times[index] = read_hatch_time(source, cursor)?;
        }
        Ok(base_result)
    }
}

fn read_hatch_time(source: &[u8], cursor: &mut usize) -> Result<u32, VolumeContainerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(4) else {
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field: "CFairyContainer hatch time",
            offset,
            needed: 4,
            available,
        }
        .into());
    };
    let Some(bytes) = source.get(offset..end) else {
 // Старый владелец не получал длину источника. При коротком буфере он
 // выходил за его границы; Rust не воспроизводит это UB.
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field: "CFairyContainer hatch time",
            offset,
            needed: 4,
            available,
        }
        .into());
    };
    *cursor = end;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("проверенный четырёхбайтовый диапазон hatch-time"),
    ))
}
