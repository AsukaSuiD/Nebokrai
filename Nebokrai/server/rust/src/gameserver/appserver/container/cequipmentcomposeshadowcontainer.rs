//! Трёхслотовая shadow-сессия compose GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cequipmentcomposeshadowcontainer.cpp`.
//! `BTreeMap<ComposeEquipmentCell, CGuid>` сохраняет enum-order слотов
//! base/sub/result `0..2`. Обычный add резервирует только пустой
//! слот, затем фиксирует already-placed source goods через
//! count-limit shadow core и откатывает slot при отказе.
//!
//! Отдельный exact `AddShadow` намеренно обходит owner/extend/
//! capacity validation, пишет previous position без auto-normalization и
//! фиксирует amount `1`. `Clear` восстанавливает limit `3`,
//! `Release` оставляет `0`. Source resolver и packet/listener dispatch
//! остаются typed boundaries. Слитые EXE COMDAT-тела принадлежат base owners
//! и не дублируются в этом Rust-owner-е.

use std::collections::BTreeMap;

use super::camountlimitgoodsshadowcontainer::{
    AmountShadowAdded, CAmountLimitGoodsShadowContainer,
};
use super::ccontainer::PreviousContainer;
use super::cgoodsshadowcontainer::{
    GoodsShadow, PlacedShadowGoods, ShadowPresenceReport, ShadowRecordBlock, ShadowRecorded,
    ShadowRemovedReport, ShadowSourceChangeOutcome,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::public::guid::CGuid;

const GOODS_LIMIT: u32 = 3;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ComposeEquipmentCell {
    BaseEquipment = 0,
    SubEquipment = 1,
    ComposeEquipment = 2,
}

impl ComposeEquipmentCell {
    const ALL: [Self; 3] = [
        Self::BaseEquipment,
        Self::SubEquipment,
        Self::ComposeEquipment,
    ];

    pub(crate) const fn position(self) -> u32 {
        self as u32
    }

    pub(crate) const fn from_position(position: u32) -> Option<Self> {
        match position {
            0 => Some(Self::BaseEquipment),
            1 => Some(Self::SubEquipment),
            2 => Some(Self::ComposeEquipment),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComposeShadowAddBlock {
    NoResolvableCell,
    InvalidPosition { position: u32 },
    Occupied { cell: ComposeEquipmentCell },
    Shadow(ShadowRecordBlock),
}

#[must_use = "add содержит AddShadow/listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ComposeShadowAdded {
    pub(crate) cell: ComposeEquipmentCell,
    pub(crate) shadow: AmountShadowAdded,
}

#[must_use = "direct AddShadow может заменить metadata и slot"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ComposeShadowInserted {
    pub(crate) cell: ComposeEquipmentCell,
    pub(crate) displaced_position: Option<CGuid>,
    pub(crate) recorded: ShadowRecorded,
    pub(crate) presence: ShadowPresenceReport,
}

#[must_use = "callback outcome определяет shadow и positional эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ComposeShadowSourceRemoved {
    pub(crate) shadow: ShadowSourceChangeOutcome,
    pub(crate) erased_cell: Option<ComposeEquipmentCell>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CEquipmentComposeShadowContainer {
    base: CAmountLimitGoodsShadowContainer,
    positions: BTreeMap<ComposeEquipmentCell, CGuid>,
}

impl Default for CEquipmentComposeShadowContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CEquipmentComposeShadowContainer {
    pub(crate) const fn new() -> Self {
        let mut base = CAmountLimitGoodsShadowContainer::new();
        base.set_goods_amount_limit(GOODS_LIMIT);
        Self {
            base,
            positions: BTreeMap::new(),
        }
    }

    pub(crate) const fn base(&self) -> &CAmountLimitGoodsShadowContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CAmountLimitGoodsShadowContainer {
        &mut self.base
    }

    pub(crate) fn positions(&self) -> &BTreeMap<ComposeEquipmentCell, CGuid> {
        &self.positions
    }

    pub(crate) fn goods_id(&self, cell: ComposeEquipmentCell) -> Option<CGuid> {
        self.positions.get(&cell).copied()
    }

    pub(crate) fn original_container_information(
        &self,
        goods_id: CGuid,
    ) -> Option<PreviousContainer> {
        self.base.base().original_container_information(goods_id)
    }

    pub(crate) fn select_cell<'a, Resolve>(
        &self,
        mut resolve: Resolve,
    ) -> Result<ComposeEquipmentCell, ComposeShadowAddBlock>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        ComposeEquipmentCell::ALL
            .into_iter()
            .find(|cell| self.get_goods(*cell, &mut resolve).is_none())
            .ok_or(ComposeShadowAddBlock::NoResolvableCell)
    }

    pub(crate) fn record_placed_goods(
        &mut self,
        cell: ComposeEquipmentCell,
        goods: &CGoods,
        previous: PreviousContainer,
        placed_identity: CGuid,
        placed_position: u32,
    ) -> Result<ComposeShadowAdded, ComposeShadowAddBlock> {
        if self.positions.contains_key(&cell) {
            return Err(ComposeShadowAddBlock::Occupied { cell });
        }
        self.positions.insert(cell, placed_identity);
        let placed = PlacedShadowGoods {
            identity: placed_identity,
            position: placed_position,
            base_properties_index: goods.base_properties_index(),
            amount: goods.amount(),
        };
        match self.base.record_placed_goods(previous, placed) {
            Ok(shadow) => Ok(ComposeShadowAdded { cell, shadow }),
            Err(block) => {
                self.positions.remove(&cell);
                Err(ComposeShadowAddBlock::Shadow(block))
            }
        }
    }

    pub(crate) fn record_placed_goods_at(
        &mut self,
        position: u32,
        goods: &CGoods,
        previous: PreviousContainer,
        placed_identity: CGuid,
        placed_position: u32,
    ) -> Result<ComposeShadowAdded, ComposeShadowAddBlock> {
        let cell = ComposeEquipmentCell::from_position(position)
            .ok_or(ComposeShadowAddBlock::InvalidPosition { position })?;
        self.record_placed_goods(cell, goods, previous, placed_identity, placed_position)
    }

    pub(crate) fn insert_shadow(
        &mut self,
        cell: ComposeEquipmentCell,
        goods: &CGoods,
        previous: PreviousContainer,
    ) -> ComposeShadowInserted {
        let goods_id = goods.identity().ex_id;
        let record = GoodsShadow {
            original_container_type: previous.container_type,
            original_container_id: previous.container_id,
            original_container_extend_id: previous.container_extend_id,
            original_goods_position: previous.goods_position,
            goods_id,
            goods_base_properties_index: goods.base_properties_index(),
            goods_amount: 1,
        };
        let recorded = self.base.base_mut().insert_shadow_record(record);
        let displaced_position = self.positions.insert(cell, goods_id);
        let presence = self
            .base
            .base()
            .add_shadow_report(goods_id)
            .expect("direct compose shadow записан до AddShadow report");
        ComposeShadowInserted {
            cell,
            displaced_position,
            recorded,
            presence,
        }
    }

    pub(crate) fn find<'a, Resolve>(&self, goods_id: CGuid, resolve: Resolve) -> Option<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.cell_for_goods(goods_id)?;
        self.base.base().find(goods_id, resolve)
    }

    pub(crate) fn get_goods<'a, Resolve>(
        &self,
        cell: ComposeEquipmentCell,
        resolve: Resolve,
    ) -> Option<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.find(*self.positions.get(&cell)?, resolve)
    }

    pub(crate) fn query_goods_position(&self, goods_id: CGuid) -> Option<u32> {
        self.cell_for_goods(goods_id)
            .map(ComposeEquipmentCell::position)
    }

    pub(crate) fn get_first_goods_by_base_properties<'a, Resolve>(
        &self,
        base_properties_index: u32,
        mut resolve: Resolve,
    ) -> Option<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.positions.values().find_map(|goods_id| {
            self.base
                .base()
                .find(*goods_id, &mut resolve)
                .filter(|goods| goods.base_properties_index() == base_properties_index)
        })
    }

    pub(crate) fn is_goods_existed<'a, Resolve>(
        &self,
        base_properties_index: u32,
        resolve: Resolve,
    ) -> bool
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.get_first_goods_by_base_properties(base_properties_index, resolve)
            .is_some()
    }

    /// Traversal сохраняет slot-order и передаёт listener-у null,
    /// если source resolver не нашёл живой goods.
    pub(crate) fn traversing_goods<'a, Resolve>(
        &self,
        mut resolve: Resolve,
    ) -> Vec<(ComposeEquipmentCell, Option<&'a CGoods>)>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.positions
            .iter()
            .map(|(cell, goods_id)| (*cell, self.base.base().find(*goods_id, &mut resolve)))
            .collect()
    }

    pub(crate) fn remove_at(&mut self, position: u32, _amount: u32) -> Option<ShadowRemovedReport> {
        let cell = ComposeEquipmentCell::from_position(position)?;
        self.remove_shadow(*self.positions.get(&cell)?)
    }

    pub(crate) fn remove_shadow(&mut self, goods_id: CGuid) -> Option<ShadowRemovedReport> {
        let cell = self.cell_for_goods(goods_id)?;
        let removed = self.base.base_mut().remove_shadow(goods_id);
        self.positions.remove(&cell);
        removed
    }

    pub(crate) fn on_source_added(
        &mut self,
        goods_id: CGuid,
        amount: u32,
    ) -> ShadowSourceChangeOutcome {
        self.base.base_mut().on_source_added(goods_id, amount)
    }

    pub(crate) fn on_source_removed(
        &mut self,
        goods_id: Option<CGuid>,
        amount: u32,
    ) -> ComposeShadowSourceRemoved {
        let goods_id = goods_id.unwrap_or(CGuid::GUID_INVALID);
        let shadow = self.base.base_mut().on_source_removed(goods_id, amount);
        let erased_cell = self.cell_for_goods(goods_id).inspect(|cell| {
            self.positions.remove(cell);
        });
        ComposeShadowSourceRemoved {
            shadow,
            erased_cell,
        }
    }

    pub(crate) fn clear(&mut self) -> usize {
        let count = self.base.clear();
        self.positions.clear();
        self.base.set_goods_amount_limit(GOODS_LIMIT);
        count
    }

    pub(crate) fn release(&mut self) -> usize {
        let count = self.base.release();
        self.positions.clear();
        count
    }

    fn cell_for_goods(&self, goods_id: CGuid) -> Option<ComposeEquipmentCell> {
        self.positions
            .iter()
            .find_map(|(cell, candidate)| (*candidate == goods_id).then_some(*cell))
    }
}
