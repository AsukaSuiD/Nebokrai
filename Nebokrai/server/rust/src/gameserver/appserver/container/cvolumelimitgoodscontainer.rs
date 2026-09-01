//! Cell/storage core `CVolumeLimitGoodsContainer` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cvolumelimitgoodscontainer.cpp`.
//! Container хранит owned goods в `CAmountLimitGoodsContainer`, а позиционный
//! слой различает available, inactive и occupied cells. `Vec` заменяет legacy
//! vector, `IndexMap` в base owner-е отвечает за GUID-index и insertion order;
//! выбор позиции, stacking, lock visibility и partial remove остаются точным
//! GameServer-адаптером.
//!
//! Constructor, volume reset, scalar и batch space/cell queries, add/remove и
//! lifecycle материализованы. Batch query клонирует container для точной
//! последовательной stack/cell simulation без изменения живого owner-а.
//! Базовый expansion получает setup-policy явно и сохраняет
//! exact release→resize→restore-owner order. Persisted codec и player
//! packet expansion достигнуты общим GameSave owner-ом; listener messages,
//! clone и auction-scale mutation ниже остаются RAW до замыкания
//! соответствующих player/message/goods owners.

use super::camountlimitgoodscontainer::{
    AmountLimitGoodsAdded, AmountLimitGoodsCleared, AmountLimitGoodsCodecError,
    AmountLimitGoodsRelease, AmountLimitGoodsTaken, CAmountLimitGoodsContainer,
};
use super::cgoodscontainer::GoodsStackMergeOutcome;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_GOODS_AUCTION_SCALE, GAP_PARTICULAR_ATTRIBUTE,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::legacycodec::{LegacyReader, LegacyWriter};
use crate::public::guid::CGuid;
use thiserror::Error;

const EXPANSION_BASE_CELL: usize = 48;
const EXPANSION_CELL_COUNT: usize = 48;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum VolumeCell {
    #[default]
    Available,
    Inactive,
    Goods(CGuid),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VolumeGoodsAddBlock {
    MissingGoods,
    InvalidCurrency,
    PositionUnavailable,
    MissingBaseProperties,
    AmountLimitReached,
    NoSpace,
}

#[must_use = "результат add определяет ownership и последующие listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VolumeGoodsAddOutcome {
    Added(AmountLimitGoodsAdded),
    Stack(GoodsStackMergeOutcome),
    Rejected(VolumeGoodsAddBlock),
}

#[must_use = "результат remove определяет ownership и последующие listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VolumeGoodsRemoveOutcome {
    Removed(AmountLimitGoodsTaken),
    RemovedButCellMissing(AmountLimitGoodsTaken),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VolumeGoodsSwapRemoval {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: u32,
    pub(crate) identity: crate::gameserver::appserver::shape::ShapeIdentity,
    pub(crate) amount: u32,
    pub(crate) listeners: Vec<super::ccontainer::ContainerListenerHandle>,
}

#[must_use = "swap outcome сохраняет displaced ownership и rollback partial effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VolumeGoodsSwapOutcome {
    Swapped {
        outgoing: CGoods,
        removed: VolumeGoodsSwapRemoval,
        added: AmountLimitGoodsAdded,
    },
    RejectedAndRestored {
        incoming: VolumeGoodsAddOutcome,
        removed: VolumeGoodsSwapRemoval,
        restored: VolumeGoodsAddOutcome,
    },
    RejectedAndOldGoodsCollected {
        incoming: VolumeGoodsAddOutcome,
        removed: VolumeGoodsSwapRemoval,
        rollback: VolumeGoodsAddOutcome,
        garbage_collected: crate::gameserver::appserver::shape::ShapeIdentity,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VolumeExpandBlock {
    Disabled,
    ZeroAmount,
    ExceedsMaximum { current: u32, requested: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum VolumeGoodsCodecError {
    #[error(transparent)]
    Amount(#[from] AmountLimitGoodsCodecError),
    #[error("volume container обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[must_use = "успешный expansion содержит release ownership-эффект"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VolumeExpandOutcome {
    Expanded {
        size: u32,
        released: AmountLimitGoodsRelease,
    },
    Rejected(VolumeExpandBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CVolumeLimitGoodsContainer {
    base: CAmountLimitGoodsContainer,
    size: u32,
    cells: Vec<VolumeCell>,
}

impl Default for CVolumeLimitGoodsContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CVolumeLimitGoodsContainer {
    pub(crate) fn new() -> Self {
        Self {
            base: CAmountLimitGoodsContainer::new(),
            size: 0,
            cells: Vec::new(),
        }
    }

    pub(crate) const fn base(&self) -> &CAmountLimitGoodsContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CAmountLimitGoodsContainer {
        &mut self.base
    }

    pub(crate) const fn size(&self) -> u32 {
        self.size
    }

    pub(crate) fn set_container_volume(&mut self, size: u32) -> AmountLimitGoodsRelease {
        let released = self.release();
        self.size = size;
        self.cells.resize(size as usize, VolumeCell::Available);
        self.base.set_goods_amount_limit(size);
        released
    }

    pub(crate) fn set_container_dimensions(
        &mut self,
        width: u32,
        height: u32,
    ) -> AmountLimitGoodsRelease {
        self.set_container_volume(width.wrapping_mul(height))
    }

    pub(crate) fn check_space(&self, requested: u32) -> bool {
        let mut available = 0u32;
        for cell in &self.cells {
            if *cell == VolumeCell::Available {
                available = available.wrapping_add(1);
                if requested <= available {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn space(&self) -> u32 {
        self.cells.iter().fold(0u32, |available, cell| {
            available.wrapping_add(u32::from(*cell == VolumeCell::Available))
        })
    }

    pub(crate) fn find_empty_space_for_goods(&self) -> Option<u32> {
        self.cells
            .iter()
            .position(|cell| *cell == VolumeCell::Available)
            .map(|position| position as u32)
    }

    pub(crate) fn is_space_enough(&self, position: u32) -> bool {
        position < self.size && self.cells.get(position as usize) == Some(&VolumeCell::Available)
    }

    /// Exact vector-overload `IsSpaceEnough`: проверяет весь batch на временной
    /// копии packet owner-а, поэтому учитывает существующие и уже размещённые
    /// в этом batch stack-и, inactive cells и duplicate GUID до mutation.
    pub(crate) fn is_space_enough_for_goods(
        &self,
        goods: &[CGoods],
        factory: &CGoodsFactory,
    ) -> bool {
        let mut simulated = self.clone();
        for goods in goods {
            if simulated.base.find(goods.identity().ex_id).is_some() {
                return false;
            }
            let mut incoming = Some(goods.clone());
            match simulated.add_goods(&mut incoming, factory, true) {
                VolumeGoodsAddOutcome::Added(_) => {}
                VolumeGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged { .. }) => {}
                VolumeGoodsAddOutcome::Stack(_) | VolumeGoodsAddOutcome::Rejected(_) => {
                    return false;
                }
            }
        }
        true
    }

    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        let VolumeCell::Goods(ex_id) = *self.cells.get(position as usize)? else {
            return None;
        };
        self.base.find(ex_id)
    }

    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        let VolumeCell::Goods(ex_id) = *self.cells.get(position as usize)? else {
            return None;
        };
        self.base.find_mut(ex_id)
    }

    pub(crate) fn query_goods_position(&self, ex_id: CGuid) -> Option<u32> {
        self.cells
            .iter()
            .position(|cell| *cell == VolumeCell::Goods(ex_id))
            .map(|position| position as u32)
    }

    pub(crate) fn goods_amount(&self, factory: &CGoodsFactory) -> u32 {
        self.base.traversing_goods().fold(0u32, |amount, goods| {
            amount.wrapping_add(u32::from(
                factory
                    .query_goods_base_properties(goods.base_properties_index())
                    .is_some()
                    && self.query_goods_position(goods.identity().ex_id).is_some(),
            ))
        })
    }

    /// Exact `GetScaleGoods`: только положительные auction-scale значения
    /// уменьшаются на единицу и попадают в GUID-вектор в insertion order.
    pub(crate) fn get_scale_goods(&mut self, factory: &CGoodsFactory) -> Vec<CGuid> {
        let mut goods_ids = Vec::new();
        for goods in self.base.traversing_goods_mut() {
            let scale = goods.addon_property_value(factory, GAP_GOODS_AUCTION_SCALE, 1);
            if 0 < scale {
                let _legacy_ignored = goods.set_addon_property_value_core(
                    GAP_GOODS_AUCTION_SCALE,
                    1,
                    scale.wrapping_sub(1),
                );
                goods_ids.push(goods.identity().ex_id);
            }
        }
        goods_ids
    }

    pub(crate) fn is_full(&self, factory: &CGoodsFactory) -> bool {
        self.base.is_full(factory) || !self.cells.contains(&VolumeCell::Available)
    }

    pub(crate) fn find_position_for_goods(
        &self,
        incoming: &CGoods,
        factory: &CGoodsFactory,
    ) -> Option<u32> {
        let maximum = incoming.max_stack_number(factory);
        if 1 < maximum {
            for stored in self.base.traversing_goods() {
                if stored.base_properties_index() == incoming.base_properties_index()
                    && incoming.amount().wrapping_add(stored.amount()) <= maximum
                    && incoming.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1)
                        == stored.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1)
                {
                    if let Some(position) = self.query_goods_position(stored.identity().ex_id) {
                        return Some(position);
                    }
                }
            }
        }
        self.find_empty_space_for_goods()
    }

    pub(crate) fn add_goods(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> VolumeGoodsAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return VolumeGoodsAddOutcome::Rejected(VolumeGoodsAddBlock::MissingGoods);
        };
        let Some(position) = self.find_position_for_goods(goods, factory) else {
            return VolumeGoodsAddOutcome::Rejected(VolumeGoodsAddBlock::NoSpace);
        };
        self.add_goods_at(position, incoming, factory, owner_progress_allows)
    }

    pub(crate) fn add_goods_at(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> VolumeGoodsAddOutcome {
        self.add_goods_at_cell(position, incoming, factory, owner_progress_allows, false)
    }

    /// Derived depot owner может занимать специальный inactive anchor; все
    /// остальные storage/factory/amount-limit контракты остаются общими.
    pub(crate) fn add_goods_at_available_or_inactive(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> VolumeGoodsAddOutcome {
        self.add_goods_at_cell(position, incoming, factory, owner_progress_allows, true)
    }

    fn add_goods_at_cell(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        allow_inactive: bool,
    ) -> VolumeGoodsAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return VolumeGoodsAddOutcome::Rejected(VolumeGoodsAddBlock::MissingGoods);
        };
        let base_properties_index = goods.base_properties_index();
        if base_properties_index == factory.get_gold_coin_index()
            || base_properties_index == factory.get_yuan_bao_index()
        {
            return VolumeGoodsAddOutcome::Rejected(VolumeGoodsAddBlock::InvalidCurrency);
        }
        if let Some(VolumeCell::Goods(ex_id)) = self.cells.get(position as usize).copied() {
            return VolumeGoodsAddOutcome::Stack(self.base.merge_goods_by_ex_id(
                ex_id,
                incoming,
                factory,
                owner_progress_allows,
            ));
        }
        let cell_allowed = self.cells.get(position as usize).is_some_and(|cell| {
            *cell == VolumeCell::Available || allow_inactive && *cell == VolumeCell::Inactive
        });
        if position >= self.size || !cell_allowed {
            return VolumeGoodsAddOutcome::Rejected(VolumeGoodsAddBlock::PositionUnavailable);
        }
        if factory
            .query_goods_base_properties(base_properties_index)
            .is_none()
        {
            return VolumeGoodsAddOutcome::Rejected(VolumeGoodsAddBlock::MissingBaseProperties);
        }

        let goods = incoming.take().expect("incoming проверен до storage add");
        let ex_id = goods.identity().ex_id;
        let mut added = match self.base.add_goods(goods, factory) {
            Ok(added) => added,
            Err(goods) => {
                *incoming = Some(goods);
                return VolumeGoodsAddOutcome::Rejected(VolumeGoodsAddBlock::AmountLimitReached);
            }
        };
        self.cells[position as usize] = VolumeCell::Goods(ex_id);
        added.position = Some(position);
        VolumeGoodsAddOutcome::Added(added)
    }

    pub(crate) fn remove_goods(&mut self, ex_id: CGuid) -> Option<VolumeGoodsRemoveOutcome> {
        let position = self.query_goods_position(ex_id);
        let mut removed = self.base.remove_goods(ex_id)?;
        removed.position = position;
        let taken = AmountLimitGoodsTaken::Removed(removed);
        let Some(position) = position else {
            return Some(VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken));
        };
        self.cells[position as usize] = VolumeCell::Available;
        Some(VolumeGoodsRemoveOutcome::Removed(taken))
    }

    pub(crate) fn take_goods<Create>(
        &mut self,
        position: u32,
        requested_amount: u32,
        factory: &CGoodsFactory,
        create_goods: Create,
    ) -> Option<VolumeGoodsRemoveOutcome>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        let VolumeCell::Goods(ex_id) = *self.cells.get(position as usize)? else {
            return None;
        };
        let taken = self.base.take_goods_by_ex_id(
            ex_id,
            position,
            requested_amount,
            factory,
            create_goods,
        )?;
        if matches!(&taken, AmountLimitGoodsTaken::Removed(_)) {
            self.cells[position as usize] = VolumeCell::Available;
        }
        Some(VolumeGoodsRemoveOutcome::Removed(taken))
    }

    /// Exact `Swap` удаляет занятый destination, делает positional `Add`
    /// incoming и при отказе тем же owner-ом возвращает displaced goods.
    /// Последний rollback failure в оригинале garbage-collect-ил displaced.
    pub(crate) fn swap_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> Option<VolumeGoodsSwapOutcome> {
        let incoming_id = incoming.as_ref()?.identity().ex_id;
        if self.base.find(incoming_id).is_some() {
            return None;
        }
        let displaced_id = self.get_goods(position)?.identity().ex_id;
        let VolumeGoodsRemoveOutcome::Removed(removed) = self.remove_goods(displaced_id)? else {
            return None;
        };
        let AmountLimitGoodsTaken::Removed(removed_goods) = removed else {
            unreachable!("Swap удаляет destination целиком")
        };
        let outgoing_identity = removed_goods.goods.identity();
        let removed = VolumeGoodsSwapRemoval {
            owner_type: removed_goods.owner_type,
            owner_id: removed_goods.owner_id,
            position: removed_goods.position.unwrap_or(position),
            identity: outgoing_identity,
            amount: removed_goods.amount,
            listeners: removed_goods.listeners,
        };
        let mut outgoing = Some(removed_goods.goods);
        let added = self.add_goods_at(position, incoming, factory, owner_progress_allows);
        if let VolumeGoodsAddOutcome::Added(added) = added {
            return Some(VolumeGoodsSwapOutcome::Swapped {
                outgoing: outgoing.take().expect("displaced goods сохранён"),
                removed,
                added,
            });
        }

        let rejected = added;
        let rollback = self.add_goods_at(position, &mut outgoing, factory, true);
        if outgoing.is_none() {
            return Some(VolumeGoodsSwapOutcome::RejectedAndRestored {
                incoming: rejected,
                removed,
                restored: rollback,
            });
        }
        let _collected = outgoing.take();
        Some(VolumeGoodsSwapOutcome::RejectedAndOldGoodsCollected {
            incoming: rejected,
            removed,
            rollback,
            garbage_collected: outgoing_identity,
        })
    }

    pub(crate) fn clean_cell(&mut self) {
        for cell in &mut self.cells {
            if *cell == VolumeCell::Inactive {
                *cell = VolumeCell::Available;
            }
        }
    }

    pub(crate) fn have_cell(&mut self, mut count: i32) {
        if count <= 0 {
            return;
        }
        for cell in self.cells.iter_mut().rev() {
            if *cell == VolumeCell::Available {
                *cell = VolumeCell::Inactive;
                count -= 1;
                if count == 0 {
                    return;
                }
            }
        }
    }

    pub(crate) fn set_all_inactive(&mut self) {
        for cell in self.cells.iter_mut().skip(EXPANSION_BASE_CELL) {
            if *cell == VolumeCell::Available {
                *cell = VolumeCell::Inactive;
            }
        }
    }

    pub(crate) fn expansion_available_space(&self) -> u32 {
        self.cells
            .iter()
            .skip(EXPANSION_BASE_CELL)
            .fold(0u32, |available, cell| {
                available.wrapping_add(u32::from(*cell == VolumeCell::Available))
            })
    }

    /// `SetExpantePosInvalid`: после базового `SetAllInactive` equipment
    /// expansion открывает prefix дополнительных packet-ячеек.
    pub(crate) fn apply_player_expansion_limit(&mut self, expanded: u32) {
        self.set_all_inactive();
        let active = EXPANSION_BASE_CELL
            .saturating_add(expanded as usize)
            .min(self.cells.len());
        for cell in self
            .cells
            .iter_mut()
            .skip(EXPANSION_BASE_CELL)
            .take(active.saturating_sub(EXPANSION_BASE_CELL))
        {
            if *cell == VolumeCell::Inactive {
                *cell = VolumeCell::Available;
            }
        }
    }

    pub(crate) fn is_cell_inactive(&self, position: u32) -> bool {
        self.cells.get(position as usize) == Some(&VolumeCell::Inactive)
    }

    pub(crate) fn set_cell_inactive(&mut self, position: u32) -> bool {
        let Some(cell) = self.cells.get_mut(position as usize) else {
            return false;
        };
        *cell = VolumeCell::Inactive;
        true
    }

    pub(crate) fn expand(
        &mut self,
        requested: u32,
        expansion_enabled: bool,
    ) -> VolumeExpandOutcome {
        if !expansion_enabled {
            return VolumeExpandOutcome::Rejected(VolumeExpandBlock::Disabled);
        }
        if requested == 0 {
            return VolumeExpandOutcome::Rejected(VolumeExpandBlock::ZeroAmount);
        }
        let current = self.size;
        let size = current.wrapping_add(requested);
        if 0xff < size {
            return VolumeExpandOutcome::Rejected(VolumeExpandBlock::ExceedsMaximum {
                current,
                requested,
            });
        }
        let owner_type = self.base.base().owner_type();
        let owner_id = self.base.base().owner_id();
        let released = self.set_container_volume(size);
        self.base.set_owner(owner_type, owner_id);
        VolumeExpandOutcome::Expanded { size, released }
    }

    pub(crate) fn activated_but_unused_count(&self, pack_add_enabled: bool) -> u32 {
        if !pack_add_enabled {
            return 0;
        }
        self.cells
            .iter()
            .skip(EXPANSION_BASE_CELL)
            .fold(0u32, |count, cell| {
                count.wrapping_add(u32::from(*cell == VolumeCell::Available))
            })
    }

    /// Legacy `CanSwap`: базовая зона разрешена без cell bounds-check, а при
    /// выключенном `bPackAdd` сохраняется странное разрешение ровно позиции 48.
    pub(crate) fn can_swap(
        &self,
        position: u32,
        expanded_cells: u32,
        pack_add_enabled: bool,
    ) -> bool {
        if position < EXPANSION_BASE_CELL as u32 {
            return true;
        }
        if EXPANSION_BASE_CELL.wrapping_add(EXPANSION_CELL_COUNT) as u32 <= position {
            return false;
        }
        if !pack_add_enabled {
            return position == EXPANSION_BASE_CELL as u32;
        }
        let occupied = self
            .cells
            .iter()
            .skip(EXPANSION_BASE_CELL)
            .take(EXPANSION_CELL_COUNT)
            .fold(0u32, |count, cell| {
                count.wrapping_add(u32::from(matches!(cell, VolumeCell::Goods(_))))
            });
        occupied <= expanded_cells.min(EXPANSION_CELL_COUNT as u32)
    }

    /// Добавляет wire-коды к уже накопленному вектору: exact owner не очищал
    /// аргумент. `0` — available, `1` — goods, `2` — inactive.
    pub(crate) fn compute_cell(&self, cells: &mut Vec<u32>) -> i32 {
        cells.reserve(self.cells.len());
        let mut active_count = 0i32;
        for cell in &self.cells {
            let code = match cell {
                VolumeCell::Available => {
                    active_count = active_count.wrapping_add(1);
                    0
                }
                VolumeCell::Goods(_) => {
                    active_count = active_count.wrapping_add(1);
                    1
                }
                VolumeCell::Inactive => 2,
            };
            cells.push(code);
        }
        active_count
    }

    pub(crate) fn clear_goods(&mut self) -> AmountLimitGoodsCleared {
        let mut cleared = self.base.clear_goods();
        for removed in &mut cleared.removed {
            removed.position = self.query_goods_position(removed.goods.identity().ex_id);
        }
        self.cells.clear();
        self.cells.resize(self.size as usize, VolumeCell::Available);
        cleared
    }

    pub(crate) fn release(&mut self) -> AmountLimitGoodsRelease {
        let released = self.base.release();
        self.size = 0;
        self.cells.clear();
        released
    }

    /// Volume wire хранит только реально размещённые factory goods: count,
    /// DWORD position и полный `CGoods` в amount-container insertion order.
    pub(crate) fn serialize(&self, destination: &mut Vec<u8>, factory: &CGoodsFactory) -> bool {
        let count = self
            .base
            .traversing_goods()
            .filter(|goods| {
                factory
                    .query_goods_base_properties(goods.base_properties_index())
                    .is_some()
                    && self.query_goods_position(goods.identity().ex_id).is_some()
            })
            .count() as u32;
        LegacyWriter::new(destination).write_u32(count);
        for goods in self.base.traversing_goods() {
            if factory
                .query_goods_base_properties(goods.base_properties_index())
                .is_some()
                && let Some(position) = self.query_goods_position(goods.identity().ex_id)
            {
                LegacyWriter::new(destination).write_u32(position);
                if !goods.serialize(destination, true) {
                    return false;
                }
            }
        }
        true
    }

    /// Exact clear/resize/count loop `CVolumeLimitGoodsContainer::Unserialize`.
    /// Add-result исторический owner игнорировал, поэтому валидно декодированный
    /// goods с недоступной позицией просто не становится частью container-а.
    pub(crate) fn unserialize<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        factory: &CGoodsFactory,
        mut ordinary_threshold: OrdinaryThreshold,
        mut battle_threshold: BattleThreshold,
    ) -> Result<(), VolumeGoodsCodecError>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        let _cleared = self.clear_goods();
        let count = read_volume_wire_u32(source, cursor, "goods count")?;
        for _ in 0..count {
            let position = read_volume_wire_u32(source, cursor, "goods position")?;
            let mut goods = CGoods::default();
            goods
                .unserialize(
                    source,
                    cursor,
                    true,
                    factory,
                    &mut ordinary_threshold,
                    &mut battle_threshold,
                )
                .map_err(AmountLimitGoodsCodecError::Goods)?;
            let mut incoming = Some(goods);
            let _legacy_ignored = self.add_goods_at(position, &mut incoming, factory, true);
        }
        Ok(())
    }
}

pub(super) fn read_volume_wire_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, VolumeGoodsCodecError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        VolumeGoodsCodecError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader.read_u32().map_err(|block| VolumeGoodsCodecError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })?;
    *cursor = reader.position();
    Ok(value)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:387
// RVA: 0x000DD260
// ADDRESS: 004dd260
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:489
// RVA: 0x000DD270
// ADDRESS: 004dd270
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:494
// RVA: 0x000DD280
// ADDRESS: 004dd280
// PROTOTYPE: CBaseObject * __thiscall Remove(long param_1, CGUID * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:509
// RVA: 0x000DD2B0
// ADDRESS: 004dd2b0
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::TraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:514
// RVA: 0x000DD2C0
// ADDRESS: 004dd2c0
// PROTOTYPE: void __thiscall TraversingContainer(CContainerListener * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::CSerializeContainer::~CSerializeContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:595
// RVA: 0x000DD2D0
// ADDRESS: 004dd2d0
// PROTOTYPE: void __thiscall ~CSerializeContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::CSerializeContainer::OnTraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:601
// RVA: 0x000DD2E0
// ADDRESS: 004dd2e0
// PROTOTYPE: int __thiscall OnTraversingContainer(CContainer * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::IsOpenExpantion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:725
// RVA: 0x000DD540
// ADDRESS: 004dd540
// PROTOTYPE: int __thiscall IsOpenExpantion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:519
// RVA: 0x000DD5D0
// ADDRESS: 004dd5d0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::CheckSpace
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:51
// RVA: 0x000DD650
// ADDRESS: 004dd650
// PROTOTYPE: bool __thiscall CheckSpace(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::GetSpace
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:66
// RVA: 0x000DD6C0
// ADDRESS: 004dd6c0
// PROTOTYPE: ulong __thiscall GetSpace(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::FindEmptySpaceForGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:107
// RVA: 0x000DD710
// ADDRESS: 004dd710
// PROTOTYPE: int __thiscall FindEmptySpaceForGoods(ulong * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::IsSpaceEnough
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:121
// RVA: 0x000DD770
// ADDRESS: 004dd770
// PROTOTYPE: int __thiscall IsSpaceEnough(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:207
// RVA: 0x000DD7D0
// ADDRESS: 004dd7d0
// PROTOTYPE: CGoods * __thiscall GetGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:287
// RVA: 0x000DD840
// ADDRESS: 004dd840
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::IsFull
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:393
// RVA: 0x000DD8B0
// ADDRESS: 004dd8b0
// PROTOTYPE: int __thiscall IsFull(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::QueryGoodsPosition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:412
// RVA: 0x000DD900
// ADDRESS: 004dd900
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGUID * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::CleanCell
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:626
// RVA: 0x000DD960
// ADDRESS: 004dd960
// PROTOTYPE: void __thiscall CleanCell(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::HaveCell
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:637
// RVA: 0x000DD9C0
// ADDRESS: 004dd9c0
// PROTOTYPE: void __thiscall HaveCell(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::SetAllInactive
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:649
// RVA: 0x000DDA40
// ADDRESS: 004dda40
// PROTOTYPE: void __thiscall SetAllInactive(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::SetExpantePosInvalid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:654
// RVA: 0x000DDA60
// ADDRESS: 004dda60
// PROTOTYPE: void __thiscall SetExpantePosInvalid(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::GetActivedButNoUseNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:688
// RVA: 0x000DDB30
// ADDRESS: 004ddb30
// PROTOTYPE: uint __thiscall GetActivedButNoUseNum(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::CanSwap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:697
// RVA: 0x000DDB60
// ADDRESS: 004ddb60
// PROTOTYPE: bool __thiscall CanSwap(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004ddb89
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:709
// RVA: 0x000DDB89
// ADDRESS: 004ddb89
// PROTOTYPE: undefined FUN_004ddb89()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::FindPositionForGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:78
// RVA: 0x000DDBE0
// ADDRESS: 004ddbe0
// PROTOTYPE: int __thiscall FindPositionForGoods(CGoods * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:234
// RVA: 0x000DDCA0
// ADDRESS: 004ddca0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::GetGoodsAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:425
// RVA: 0x000DDD10
// ADDRESS: 004ddd10
// PROTOTYPE: ulong __thiscall GetGoodsAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:260
// RVA: 0x000DDDD0
// ADDRESS: 004dddd0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::~CVolumeLimitGoodsContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:29
// RVA: 0x000DDDF0
// ADDRESS: 004dddf0
// PROTOTYPE: void __thiscall ~CVolumeLimitGoodsContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::CVolumeLimitGoodsContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:23
// RVA: 0x000DE0A0
// ADDRESS: 004de0a0
// PROTOTYPE: undefined __thiscall CVolumeLimitGoodsContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::SetContainerVolume
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:35
// RVA: 0x000DE100
// ADDRESS: 004de100
// PROTOTYPE: void __thiscall SetContainerVolume(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::SetContainerVolume
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:43
// RVA: 0x000DE160
// ADDRESS: 004de160
// PROTOTYPE: void __thiscall SetContainerVolume(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:252
// RVA: 0x000DE1B0
// ADDRESS: 004de1b0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Clone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:268
// RVA: 0x000DE200
// ADDRESS: 004de200
// PROTOTYPE: int __thiscall Clone(CGoodsContainer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:539
// RVA: 0x000DE250
// ADDRESS: 004de250
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Expant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:570
// RVA: 0x000DE2F0
// ADDRESS: 004de2f0
// PROTOTYPE: int __thiscall Expant(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::ComputeCell
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:729
// RVA: 0x000DE500
// ADDRESS: 004de500
// PROTOTYPE: long __thiscall ComputeCell(vector<unsigned_long,std::allocator<unsigned_long>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::GetScaleGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:754
// RVA: 0x000DE680
// ADDRESS: 004de680
// PROTOTYPE: void __thiscall GetScaleGoods(vector<CGUID,std::allocator<CGUID>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:307
// RVA: 0x000DE6E0
// ADDRESS: 004de6e0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsContainer::QueryGoodsPosition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:224
// RVA: 0x001DDDB0
// ADDRESS: 005dddb0
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGoods * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
