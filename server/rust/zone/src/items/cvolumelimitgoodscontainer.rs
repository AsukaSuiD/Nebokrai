//! Cell/storage core `CVolumeLimitGoodsContainer` исторического GameServer,
//! перенесённый в Zone `items/` — владельца типов контейнеров и операций над
//! ними.
//!
//! Тела перенесены буквально из прежнего
//! `src/gameserver/appserver/container/cvolumelimitgoodscontainer.rs` (волна Z-C1);
//! отличия — нормализация `pub(crate)`→`pub` на границе crate (включая
//! `pub(super)` depot-помощника wire-декодера) и швы переноса (не расхождения):
//! amount-limit ядро и stack-merge — Zone
//! `items/camountlimitgoodscontainer.rs`/`items/cgoodscontainer.rs`, `CGoods` —
//! Zone `items/cgoods.rs`, GAP-константы — Zone `content/goods.rs`, реестр
//! `CGoodsFactory` — Zone `content/goodsfactory.rs` (волна Z-G0b), `ShapeIdentity` —
//! Zone `regions/` (re-export `identity`), `CGuid` и wire-кодеки — Shared.
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
//! packet expansion достигнуты общим GameSave owner-ом. Auction-scale mutation
//! связана с `CPlayer::TellClientScale`; listener messages и clone ниже
//! остаются RAW до замыкания соответствующих owners.
//! Swap использует единый remove→Add→rollback-алгоритм; packet-owner подставляет
//! синхронный Add с GoodsAI/player-listener и для основной, и для обратной попытки.

use super::camountlimitgoodscontainer::{
    AmountLimitGoodsAdded, AmountLimitGoodsCleared, AmountLimitGoodsCodecError,
    AmountLimitGoodsRelease, AmountLimitGoodsTaken, CAmountLimitGoodsContainer,
};
use super::cgoods::CGoods;
use super::cgoodscontainer::GoodsStackMergeOutcome;
use crate::content::goods::{GAP_GOODS_AUCTION_SCALE, GAP_PARTICULAR_ATTRIBUTE};
use crate::content::goodsfactory::CGoodsFactory;
use crate::regions::ShapeIdentity;
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use nebokrai_shared::values::CGuid;
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
pub enum VolumeGoodsAddBlock {
    MissingGoods,
    InvalidCurrency,
    PositionUnavailable,
    MissingBaseProperties,
    AmountLimitReached,
    NoSpace,
}

#[must_use = "результат add определяет ownership и последующие listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VolumeGoodsAddOutcome {
    Added(AmountLimitGoodsAdded),
    Stack(GoodsStackMergeOutcome),
    Rejected(VolumeGoodsAddBlock),
}

#[must_use = "результат remove определяет ownership и последующие listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VolumeGoodsRemoveOutcome {
    Removed(AmountLimitGoodsTaken),
    RemovedButCellMissing(AmountLimitGoodsTaken),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeGoodsSwapRemoval {
    pub owner_type: i32,
    pub owner_id: i32,
    pub position: u32,
    pub identity: ShapeIdentity,
    pub amount: u32,
    pub listeners: Vec<super::ccontainer::ContainerListenerHandle>,
}

#[must_use = "swap outcome сохраняет displaced ownership и rollback partial effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VolumeGoodsSwapOutcome {
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
        garbage_collected: ShapeIdentity,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VolumeExpandBlock {
    Disabled,
    ZeroAmount,
    ExceedsMaximum { current: u32, requested: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum VolumeGoodsCodecError {
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
pub enum VolumeExpandOutcome {
    Expanded {
        size: u32,
        released: AmountLimitGoodsRelease,
    },
    Rejected(VolumeExpandBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CVolumeLimitGoodsContainer {
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
    pub fn new() -> Self {
        Self {
            base: CAmountLimitGoodsContainer::new(),
            size: 0,
            cells: Vec::new(),
        }
    }

    pub const fn base(&self) -> &CAmountLimitGoodsContainer {
        &self.base
    }

    pub const fn base_mut(&mut self) -> &mut CAmountLimitGoodsContainer {
        &mut self.base
    }

    pub const fn size(&self) -> u32 {
        self.size
    }

    pub fn set_container_volume(&mut self, size: u32) -> AmountLimitGoodsRelease {
        let released = self.release();
        self.size = size;
        self.cells.resize(size as usize, VolumeCell::Available);
        self.base.set_goods_amount_limit(size);
        released
    }

    pub fn set_container_dimensions(
        &mut self,
        width: u32,
        height: u32,
    ) -> AmountLimitGoodsRelease {
        self.set_container_volume(width.wrapping_mul(height))
    }

    pub fn check_space(&self, requested: u32) -> bool {
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

    pub fn space(&self) -> u32 {
        self.cells.iter().fold(0u32, |available, cell| {
            available.wrapping_add(u32::from(*cell == VolumeCell::Available))
        })
    }

    pub fn find_empty_space_for_goods(&self) -> Option<u32> {
        self.cells
            .iter()
            .position(|cell| *cell == VolumeCell::Available)
            .map(|position| position as u32)
    }

    pub fn is_space_enough(&self, position: u32) -> bool {
        position < self.size && self.cells.get(position as usize) == Some(&VolumeCell::Available)
    }

    /// Exact vector-overload `IsSpaceEnough`: проверяет весь batch на временной
    /// копии packet owner-а, поэтому учитывает существующие и уже размещённые
    /// в этом batch stack-и, inactive cells и duplicate GUID до mutation.
    pub fn is_space_enough_for_goods(
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

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        let VolumeCell::Goods(ex_id) = *self.cells.get(position as usize)? else {
            return None;
        };
        self.base.find(ex_id)
    }

    pub fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        let VolumeCell::Goods(ex_id) = *self.cells.get(position as usize)? else {
            return None;
        };
        self.base.find_mut(ex_id)
    }

    pub fn query_goods_position(&self, ex_id: CGuid) -> Option<u32> {
        self.cells
            .iter()
            .position(|cell| *cell == VolumeCell::Goods(ex_id))
            .map(|position| position as u32)
    }

    pub fn goods_amount(&self, factory: &CGoodsFactory) -> u32 {
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
    pub fn get_scale_goods(&mut self, factory: &CGoodsFactory) -> Vec<CGuid> {
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

    pub fn is_full(&self, factory: &CGoodsFactory) -> bool {
        self.base.is_full(factory) || !self.cells.contains(&VolumeCell::Available)
    }

    pub fn find_position_for_goods(
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

    pub fn add_goods(
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

    pub fn add_goods_at(
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
    pub fn add_goods_at_available_or_inactive(
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

    pub fn remove_goods(&mut self, ex_id: CGuid) -> Option<VolumeGoodsRemoveOutcome> {
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

    pub fn take_goods<Create>(
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
    pub fn swap_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> Option<VolumeGoodsSwapOutcome> {
        Self::swap_goods_with_owner(
            self,
            position,
            incoming,
            owner_progress_allows,
            |container| container,
            |container, position, incoming, owner_progress_allows| {
                container.add_goods_at(position, incoming, factory, owner_progress_allows)
            },
        )
    }

    pub fn swap_goods_with_owner<Owner>(
        owner: &mut Owner,
        position: u32,
        incoming: &mut Option<CGoods>,
        owner_progress_allows: bool,
        container: impl Fn(&mut Owner) -> &mut Self,
        mut add_at: impl FnMut(&mut Owner, u32, &mut Option<CGoods>, bool) -> VolumeGoodsAddOutcome,
    ) -> Option<VolumeGoodsSwapOutcome> {
        let incoming_id = incoming.as_ref()?.identity().ex_id;
        if container(owner).base.find(incoming_id).is_some() {
            return None;
        }
        let displaced_id = container(owner).get_goods(position)?.identity().ex_id;
        let VolumeGoodsRemoveOutcome::Removed(removed) = container(owner).remove_goods(displaced_id)?
        else {
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
        let added = add_at(owner, position, incoming, owner_progress_allows);
        if let VolumeGoodsAddOutcome::Added(added) = added {
            return Some(VolumeGoodsSwapOutcome::Swapped {
                outgoing: outgoing.take().expect("displaced goods сохранён"),
                removed,
                added,
            });
        }

        let rejected = added;
        let rollback = add_at(owner, position, &mut outgoing, true);
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

    pub fn clean_cell(&mut self) {
        for cell in &mut self.cells {
            if *cell == VolumeCell::Inactive {
                *cell = VolumeCell::Available;
            }
        }
    }

    pub fn have_cell(&mut self, mut count: i32) {
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

    pub fn set_all_inactive(&mut self) {
        for cell in self.cells.iter_mut().skip(EXPANSION_BASE_CELL) {
            if *cell == VolumeCell::Available {
                *cell = VolumeCell::Inactive;
            }
        }
    }

    pub fn expansion_available_space(&self) -> u32 {
        self.cells
            .iter()
            .skip(EXPANSION_BASE_CELL)
            .fold(0u32, |available, cell| {
                available.wrapping_add(u32::from(*cell == VolumeCell::Available))
            })
    }

    /// `SetExpantePosInvalid`: после базового `SetAllInactive` equipment
    /// expansion открывает prefix дополнительных packet-ячеек.
    pub fn apply_player_expansion_limit(&mut self, expanded: u32) {
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

    pub fn is_cell_inactive(&self, position: u32) -> bool {
        self.cells.get(position as usize) == Some(&VolumeCell::Inactive)
    }

    pub fn set_cell_inactive(&mut self, position: u32) -> bool {
        let Some(cell) = self.cells.get_mut(position as usize) else {
            return false;
        };
        *cell = VolumeCell::Inactive;
        true
    }

    pub fn expand(
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

    pub fn activated_but_unused_count(&self, pack_add_enabled: bool) -> u32 {
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
    pub fn can_swap(
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
    pub fn compute_cell(&self, cells: &mut Vec<u32>) -> i32 {
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

    pub fn clear_goods(&mut self) -> AmountLimitGoodsCleared {
        let mut cleared = self.base.clear_goods();
        for removed in &mut cleared.removed {
            removed.position = self.query_goods_position(removed.goods.identity().ex_id);
        }
        self.cells.clear();
        self.cells.resize(self.size as usize, VolumeCell::Available);
        cleared
    }

    pub fn release(&mut self) -> AmountLimitGoodsRelease {
        let released = self.base.release();
        self.size = 0;
        self.cells.clear();
        released
    }

    /// Volume wire хранит только реально размещённые factory goods: count,
    /// DWORD position и полный `CGoods` в amount-container insertion order.
    pub fn serialize(&self, destination: &mut Vec<u8>, factory: &CGoodsFactory) -> bool {
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
    pub fn unserialize<OrdinaryThreshold, BattleThreshold>(
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

pub fn read_volume_wire_u32(
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
