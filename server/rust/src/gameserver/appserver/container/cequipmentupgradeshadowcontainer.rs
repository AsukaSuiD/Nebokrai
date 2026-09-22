//! Пятислотовая shadow-сессия улучшения снаряжения GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cequipmentupgradeshadowcontainer.cpp`.
//! `BTreeMap<UpgradeEquipmentCell, CGuid>` заменяет MSVC map и
//! сохраняет enum-order: upgradeable equipment в слоте `0`, base gem
//! типа `1` в слоте `1`, до трёх gems типа `2` в слотах `2..4`.
//! Gem обязан иметь amount `1` и не иметь `GAP_BF_GEM == 1`.
//!
//! Slot резервируется до base add и откатывается при отказе.
//! Exact source move остаётся caller-boundary; сюда передаётся
//! уже фактически placed GUID/position. При stack merge slot привязывается
//! к живому target GUID вместо legacy dangling incoming GUID. `Clear`
//! восстанавливает limit `5`, `Release` оставляет его равным `0`. Реальная
//! source-player move/delete-транзакция замкнута equipment-upgrade caller-ом;
//! сам shadow хранит только исходную позицию и listener metadata.

use std::collections::BTreeMap;

use super::camountlimitgoodsshadowcontainer::{
    AmountShadowAdded, CAmountLimitGoodsShadowContainer,
};
use super::ccontainer::PreviousContainer;
use super::cgoodsshadowcontainer::{
    GoodsShadow, PlacedShadowGoods, ShadowRecordBlock, ShadowRemovedReport,
    ShadowSourceChangeOutcome,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{GAP_BF_GEM, GAP_GEM_TYPE};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use nebokrai_shared::values::CGuid;

const GOODS_LIMIT: u32 = 5;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum UpgradeEquipmentCell {
    Equipment = 0,
    BaseGem = 1,
    GemOne = 2,
    GemTwo = 3,
    GemThree = 4,
}

impl UpgradeEquipmentCell {
    const UPGRADE_GEMS: [Self; 3] = [Self::GemOne, Self::GemTwo, Self::GemThree];

    pub(crate) const fn position(self) -> u32 {
        self as u32
    }

    pub(crate) const fn from_position(position: u32) -> Option<Self> {
        match position {
            0 => Some(Self::Equipment),
            1 => Some(Self::BaseGem),
            2 => Some(Self::GemOne),
            3 => Some(Self::GemTwo),
            4 => Some(Self::GemThree),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UpgradeShadowAddBlock {
    UnsupportedGoods,
    InvalidPosition { position: u32 },
    Occupied { cell: UpgradeEquipmentCell },
    NoFreeUpgradeGemCell,
    InvalidEquipment,
    InvalidBaseGem,
    InvalidUpgradeGem,
    Shadow(ShadowRecordBlock),
}

#[must_use = "add report содержит AddShadow/listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UpgradeShadowAdded {
    pub(crate) cell: UpgradeEquipmentCell,
    pub(crate) shadow: AmountShadowAdded,
}

#[must_use = "callback outcome определяет amount/remove и slot-map эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UpgradeShadowSourceRemoved {
    pub(crate) shadow: ShadowSourceChangeOutcome,
    pub(crate) erased_cell: Option<UpgradeEquipmentCell>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CEquipmentUpgradeShadowContainer {
    base: CAmountLimitGoodsShadowContainer,
    positions: BTreeMap<UpgradeEquipmentCell, CGuid>,
}

impl Default for CEquipmentUpgradeShadowContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CEquipmentUpgradeShadowContainer {
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

    pub(crate) fn positions(&self) -> &BTreeMap<UpgradeEquipmentCell, CGuid> {
        &self.positions
    }

    pub(crate) fn select_cell(
        &self,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> Result<UpgradeEquipmentCell, UpgradeShadowAddBlock> {
        if goods.can_upgraded(factory) {
            return Ok(UpgradeEquipmentCell::Equipment);
        }
        match goods.addon_property_value(factory, GAP_GEM_TYPE, 1) {
            1 => Ok(UpgradeEquipmentCell::BaseGem),
            2 => UpgradeEquipmentCell::UPGRADE_GEMS
                .into_iter()
                .find(|cell| !self.positions.contains_key(cell))
                .ok_or(UpgradeShadowAddBlock::NoFreeUpgradeGemCell),
            _ => Err(UpgradeShadowAddBlock::UnsupportedGoods),
        }
    }

    pub(crate) fn record_placed_goods(
        &mut self,
        cell: UpgradeEquipmentCell,
        goods: &CGoods,
        previous: PreviousContainer,
        placed_identity: CGuid,
        placed_position: u32,
        factory: &CGoodsFactory,
    ) -> Result<UpgradeShadowAdded, UpgradeShadowAddBlock> {
        if self.positions.contains_key(&cell) {
            return Err(UpgradeShadowAddBlock::Occupied { cell });
        }
        self.validate_cell_goods(cell, goods, factory)?;

        // Exact owner вставлял slot до base add. После source stack
        // используем живой placed GUID, а не уничтоженный incoming.
        self.positions.insert(cell, placed_identity);
        let placed = PlacedShadowGoods {
            identity: placed_identity,
            position: placed_position,
            base_properties_index: goods.base_properties_index(),
            amount: goods.amount(),
        };
        match self.base.record_placed_goods(previous, placed) {
            Ok(shadow) => Ok(UpgradeShadowAdded { cell, shadow }),
            Err(block) => {
                self.positions.remove(&cell);
                Err(UpgradeShadowAddBlock::Shadow(block))
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
        factory: &CGoodsFactory,
    ) -> Result<UpgradeShadowAdded, UpgradeShadowAddBlock> {
        let cell = UpgradeEquipmentCell::from_position(position)
            .ok_or(UpgradeShadowAddBlock::InvalidPosition { position })?;
        self.record_placed_goods(
            cell,
            goods,
            previous,
            placed_identity,
            placed_position,
            factory,
        )
    }

    pub(crate) fn get_goods<'a, Resolve>(
        &self,
        cell: UpgradeEquipmentCell,
        resolve: Resolve,
    ) -> Option<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.base.base().find(*self.positions.get(&cell)?, resolve)
    }

    /// Exact plural overload возвращал не более одного goods.
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

    /// Derived `Remove(GUID)` не трогает base shadow, если GUID
    /// отсутствует в positional map.
    pub(crate) fn remove_shadow(&mut self, goods_id: CGuid) -> Option<ShadowRemovedReport> {
        let cell = self.cell_for_goods(goods_id)?;
        let removed = self.base.base_mut().remove_shadow(goods_id);
        self.positions.remove(&cell);
        removed
    }

    pub(crate) fn on_source_removed(
        &mut self,
        goods_id: CGuid,
        amount: u32,
    ) -> UpgradeShadowSourceRemoved {
        let shadow = self.base.base_mut().on_source_removed(goods_id, amount);
        // Listener callback исходного base возвращал success и при missing;
        // derived после него независимо удалял первое совпадение slot-map.
        let erased_cell = self.cell_for_goods(goods_id).inspect(|cell| {
            self.positions.remove(cell);
        });
        UpgradeShadowSourceRemoved {
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

    fn validate_cell_goods(
        &self,
        cell: UpgradeEquipmentCell,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> Result<(), UpgradeShadowAddBlock> {
        match cell {
            UpgradeEquipmentCell::Equipment => goods
                .can_upgraded(factory)
                .then_some(())
                .ok_or(UpgradeShadowAddBlock::InvalidEquipment),
            UpgradeEquipmentCell::BaseGem => self
                .is_gem(goods, factory, 1)
                .then_some(())
                .ok_or(UpgradeShadowAddBlock::InvalidBaseGem),
            UpgradeEquipmentCell::GemOne
            | UpgradeEquipmentCell::GemTwo
            | UpgradeEquipmentCell::GemThree => self
                .is_gem(goods, factory, 2)
                .then_some(())
                .ok_or(UpgradeShadowAddBlock::InvalidUpgradeGem),
        }
    }

    fn is_gem(&self, goods: &CGoods, factory: &CGoodsFactory, gem_type: i32) -> bool {
        goods.addon_property_value(factory, GAP_GEM_TYPE, 1) == gem_type
            && goods.amount() == 1
            && goods.addon_property_value(factory, GAP_BF_GEM, 1) != 1
    }

    fn cell_for_goods(&self, goods_id: CGuid) -> Option<UpgradeEquipmentCell> {
        self.positions
            .iter()
            .find_map(|(cell, candidate)| (*candidate == goods_id).then_some(*cell))
    }
}
