//! Позиционный слой goods shadow container исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cvolumelimitgoodsshadowcontainer.cpp`.
//! `Vec<Option<CGuid>>` заменяет legacy vector с `GUID_INVALID`, а metadata и
//! raw GUID-order остаются у `CAmountLimitGoodsShadowContainer`/`BTreeMap`.
//!
//! Volume lifecycle, position lookup, full/space checks, stack-position quirk
//! и synchronized `RemoveShadow` материализованы. Exact stack selection не
//! сравнивает `GAP_PARTICULAR_ATTRIBUTE` и использует wrapping `u32` addition.
//! Source player move, no-trade validation, add packet и clone target ниже
//! остаются RAW до межконтейнерного dispatcher-а.

use super::camountlimitgoodsshadowcontainer::CAmountLimitGoodsShadowContainer;
use super::cgoodsshadowcontainer::{GoodsShadow, ShadowRemovedReport};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CVolumeLimitGoodsShadowContainer {
    base: CAmountLimitGoodsShadowContainer,
    size: u32,
    cells: Vec<Option<CGuid>>,
}

impl Default for CVolumeLimitGoodsShadowContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CVolumeLimitGoodsShadowContainer {
    pub(crate) const fn new() -> Self {
        Self {
            base: CAmountLimitGoodsShadowContainer::new(),
            size: 0,
            cells: Vec::new(),
        }
    }

    pub(crate) const fn base(&self) -> &CAmountLimitGoodsShadowContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CAmountLimitGoodsShadowContainer {
        &mut self.base
    }

    pub(crate) const fn size(&self) -> u32 {
        self.size
    }

    pub(crate) fn set_container_volume(&mut self, size: u32) -> usize {
        let released = self.release();
        self.size = size;
        self.cells.resize(size as usize, None);
        self.base.set_goods_amount_limit(size);
        released
    }

    pub(crate) fn set_container_dimensions(&mut self, width: u32, height: u32) -> usize {
        self.set_container_volume(width.wrapping_mul(height))
    }

    pub(crate) fn is_full(&self) -> bool {
        self.base.is_full() || !self.cells.contains(&None)
    }

    pub(crate) fn query_goods_position(&self, goods_id: CGuid) -> Option<u32> {
        self.cells
            .iter()
            .position(|cell| *cell == Some(goods_id))
            .map(|position| position as u32)
    }

    pub(crate) fn get_goods<'a, Resolve>(
        &self,
        position: u32,
        resolve: Resolve,
    ) -> Option<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        let goods_id = self.cells.get(position as usize)?.as_ref().copied()?;
        self.base.base().find(goods_id, resolve)
    }

    pub(crate) fn is_space_enough(&self, position: u32) -> bool {
        position < self.size && self.cells.get(position as usize) == Some(&None)
    }

    pub(crate) fn find_position_for_goods(
        &self,
        incoming: &CGoods,
        factory: &CGoodsFactory,
    ) -> Option<u32> {
        let maximum = incoming.max_stack_number(factory);
        if 1 < maximum {
            for (goods_id, shadow) in self.base.base().shadows() {
                if shadow.goods_base_properties_index == incoming.base_properties_index()
                    && incoming.amount().wrapping_add(shadow.goods_amount) <= maximum
                {
                    if let Some(position) = self.query_goods_position(*goods_id) {
                        return Some(position);
                    }
                }
            }
        }
        self.cells
            .iter()
            .position(Option::is_none)
            .map(|position| position as u32)
    }

    pub(crate) fn occupy_cell(&mut self, position: u32, goods_id: CGuid) -> bool {
        if !self.is_space_enough(position) {
            return false;
        }
        self.cells[position as usize] = Some(goods_id);
        true
    }

    pub(crate) fn remove_shadow(&mut self, goods_id: CGuid) -> Option<ShadowRemovedReport> {
        if let Some(position) = self.query_goods_position(goods_id) {
            self.cells[position as usize] = None;
        }
        self.base.base_mut().remove_shadow(goods_id)
    }

    pub(crate) fn clear(&mut self) -> usize {
        let count = self.base.clear();
        self.cells.clear();
        self.cells.resize(self.size as usize, None);
        count
    }

    pub(crate) fn release(&mut self) -> usize {
        let count = self.base.release();
        self.size = 0;
        self.cells.clear();
        count
    }
}
