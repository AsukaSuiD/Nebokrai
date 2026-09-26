//! Контейнер товаров с ячейками из `cvolumelimitgoodscontainer.cpp/.h`.
//!
//! Размер контейнера определяет вектор GUID-ячеек и amount-limit. `Clear`
//! сохраняет размер и заново создаёт пустые ячейки; `Release` обнуляет всё.
//! Positional `Add` либо заполняет пустую ячейку, либо складывает совместимый
//! stack с 32-битной wrapping-арифметикой. Автоматический `Add` сначала ищет
//! подходящий stack, затем первую свободную ячейку.
//!
//! Serialize пишет только товары с base-properties и найденной ячейкой.
//! Decoder сохраняет ранний `Clear` и уже добавленные записи при поздней ошибке.
//! Locked-товар занимает ячейку, хотя обычный `GetGoods` его скрывает.
//!
//! GUID-remove сначала удаляет товар из amount-owner и лишь затем очищает
//! ячейку. `AddFromDB` обходит stacking и напрямую заполняет map/cell после
//! собственных проверок. `BTreeMap` и `Box` заменяют hash-map и сырое владение;
//! порядок выбора между несколькими равными stack-ами остаётся внешней границей.

use std::error::Error;
use std::fmt;

use crate::content::goodslistener::TraversedGoods;
use nebokrai_shared::values::CGuid;
use crate::items::ccontainerlistener::CContainerListener;

use crate::content::cgoods::{CGoods, GoodsCodecError, GoodsDbSnapshotBlock};
use crate::content::cgoodsfactory::query_goods_base_properties;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::items::camountlimitgoodscontainer::{AmountContainerCodecError, CAmountLimitGoodsContainer};
use crate::items::cgoodscontainer::add_to_occupied_position;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VolumeContainerCodecError {
    Amount(AmountContainerCodecError),
    Goods(GoodsCodecError),
    ValidGoodsCountOutsideLegacyRange { count: usize },
}

impl fmt::Display for VolumeContainerCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Amount(error) => error.fmt(formatter),
            Self::Goods(error) => error.fmt(formatter),
            Self::ValidGoodsCountOutsideLegacyRange { count } => write!(
                formatter,
                "volume-container содержит {count} валидных cells вне 32-битного legacy-диапазона"
            ),
        }
    }
}

impl Error for VolumeContainerCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Amount(error) => Some(error),
            Self::Goods(error) => Some(error),
            Self::ValidGoodsCountOutsideLegacyRange { .. } => None,
        }
    }
}

impl From<AmountContainerCodecError> for VolumeContainerCodecError {
    fn from(error: AmountContainerCodecError) -> Self {
        Self::Amount(error)
    }
}

impl From<GoodsCodecError> for VolumeContainerCodecError {
    fn from(error: GoodsCodecError) -> Self {
        Self::Goods(error)
    }
}

pub struct CVolumeLimitGoodsContainer {
    amount_base: CAmountLimitGoodsContainer,
    size: u32,
    cells: Vec<CGuid>,
}

impl CVolumeLimitGoodsContainer {
    pub const fn with_constructor_defaults() -> Self {
        Self {
            amount_base: CAmountLimitGoodsContainer::with_constructor_defaults(),
            size: 0,
            cells: Vec::new(),
        }
    }

    pub fn set_container_volume_2d(&mut self, width: u32, height: u32) {
        self.set_container_volume(width.wrapping_mul(height));
    }

    pub fn set_container_volume(&mut self, size: u32) {
        self.release();
        self.size = size;
        self.cells.resize(size as usize, CGuid::GUID_INVALID);
        self.amount_base.set_goods_amount_limit(size);
    }

    pub const fn get_goods_amount_limit(&self) -> u32 {
        self.amount_base.get_goods_amount_limit()
    }

    pub fn clear(&mut self) {
        self.amount_base.clear();
        self.cells.clear();
        self.cells.resize(self.size as usize, CGuid::GUID_INVALID);
    }

    pub fn release(&mut self) {
        self.amount_base.release();
        self.size = 0;
        self.cells.clear();
    }

    pub fn ai(&mut self, on_goods_ai: impl FnMut(&mut CGoods)) {
        self.amount_base.ai(on_goods_ai);
    }

    pub fn query_goods_position(&self, ex_id: &CGuid) -> Option<u32> {
        self.cells
            .iter()
            .position(|cell| cell == ex_id)
            .and_then(|position| u32::try_from(position).ok())
    }

    pub fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        self.amount_base.find(ex_id)
    }

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        let ex_id = self.cells.get(position as usize)?;
        if ex_id.is_invalid() {
            return None;
        }
        self.find(ex_id)
    }

    pub fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        let ex_id = *self.get_goods(position)?.get_ex_id();
        self.amount_base.goods_mut(&ex_id)
    }

    pub fn query_goods_position_by_object(&self, goods: Option<&CGoods>) -> Option<u32> {
        self.query_goods_position(goods?.get_ex_id())
    }

    pub fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        self.amount_base.traversing_container(listener);
    }

    pub fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, GoodsDbSnapshotBlock> {
        self.amount_base
            .goods()
            .map(|goods| {
                Ok(TraversedGoods {
                    goods: goods.db_save_snapshot(registry)?,
                    position: self
                        .query_goods_position_by_object(Some(goods))
                        .unwrap_or(0) as u8,
                })
            })
            .collect()
    }

    pub fn is_space_enough(&self, position: u32) -> bool {
        position < self.size
            && self
                .cells
                .get(position as usize)
                .is_some_and(|cell| cell.is_invalid())
    }

    pub fn get_goods_amount(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, VolumeContainerCodecError> {
        let mut count = 0usize;
        for goods in self.amount_base.goods() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if query_goods_base_properties(registry, index).is_some()
                && self.query_goods_position_by_object(Some(goods)).is_some()
            {
                count += 1;
            }
        }
        u32::try_from(count)
            .map_err(|_| VolumeContainerCodecError::ValidGoodsCountOutsideLegacyRange { count })
    }

    pub fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        if self.amount_base.is_full(registry)? {
            return Ok(true);
        }
        Ok(!self.cells.iter().any(|cell| cell.is_invalid()))
    }

    pub fn find_position_for_goods(
        &self,
        goods: Option<&CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<u32>, GoodsCodecError> {
        let Some(goods) = goods else {
            return Ok(None);
        };

        if goods.get_max_stack_number(registry)? > 1 {
            for existing in self.amount_base.goods() {
                if existing.get_base_properties_index() == goods.get_base_properties_index()
                    && goods.get_amount().wrapping_add(existing.get_amount())
                        <= goods.get_max_stack_number(registry)?
                    && let Some(position) = self.query_goods_position_by_object(Some(existing))
                {
                    return Ok(Some(position));
                }
            }
        }

        Ok(self
            .cells
            .iter()
            .position(|cell| cell.is_invalid())
            .and_then(|position| u32::try_from(position).ok()))
    }

    pub fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        let Some(position) = self.find_position_for_goods(Some(goods.as_ref()), registry)? else {
            return Ok(Some(goods));
        };
        self.add_at(position, goods, registry)
    }

    pub fn remove(
        &mut self,
        ex_id: &CGuid,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        let Some(goods) = self.amount_base.remove(ex_id) else {
            return Ok(None);
        };

        let Some(index) = goods.get_base_properties_index() else {
            return Err(GoodsCodecError::MissingBasePropertiesIndex.into());
        };
        if query_goods_base_properties(registry, index).is_none() {
            return Ok(None);
        }

        let Some(position) = self.query_goods_position(ex_id) else {
            return Ok(None);
        };
        self.cells[position as usize] = CGuid::GUID_INVALID;
        Ok(Some(goods))
    }

    pub fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.get_goods(position).is_some() || !self.is_space_enough(position) {
            return Ok(Some(goods));
        }

        let Some(index) = goods.get_base_properties_index() else {
            return Err(GoodsCodecError::MissingBasePropertiesIndex.into());
        };
        if query_goods_base_properties(registry, index).is_none() {
            return Ok(Some(goods));
        }

        let ex_id = *goods.get_ex_id();
        self.amount_base.insert_unchecked(goods);
        self.cells[position as usize] = ex_id;
        Ok(None)
    }

    pub fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.amount_base.is_full(registry)? || position >= self.size {
            return Ok(Some(goods));
        }
        let Some(&cell) = self.cells.get(position as usize) else {
            return Ok(Some(goods));
        };
        if !cell.is_invalid() {
            let Some(existing) = self.amount_base.goods_mut(&cell) else {
                return Ok(Some(goods));
            };
            return add_to_occupied_position(existing, goods, registry).map_err(Into::into);
        }
        let index = goods
            .get_base_properties_index()
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        if query_goods_base_properties(registry, index).is_none() {
            return Ok(Some(goods));
        }
        let ex_id = *goods.get_ex_id();
        if let Some(rejected) = self.amount_base.add(goods, registry)? {
            return Ok(Some(rejected));
        }
        self.cells[position as usize] = ex_id;
        Ok(None)
    }

    pub fn clone_into(
        &self,
        target: &mut CVolumeLimitGoodsContainer,
    ) -> Result<bool, GoodsCodecError> {
        let _ = self.amount_base.clone_into(&mut target.amount_base)?;
        target.size = self.size;
        target.cells.clone_from(&self.cells);
        Ok(true)
    }

    pub fn serialize(
        &self,
        destination: &mut Vec<u8>,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        let count = self.get_goods_amount(registry)?;
        destination.extend_from_slice(&count.to_le_bytes());
        for goods in self.amount_base.goods() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if query_goods_base_properties(registry, index).is_some()
                && let Some(position) = self.query_goods_position_by_object(Some(goods))
            {
                destination.extend_from_slice(&position.to_le_bytes());
                let _ = goods.serialize(destination, true)?;
            }
        }
        Ok(true)
    }
}
