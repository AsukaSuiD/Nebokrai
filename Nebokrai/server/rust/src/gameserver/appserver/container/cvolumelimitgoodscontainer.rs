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
//! Constructor, volume reset, space/cell queries, add/remove и lifecycle
//! материализованы. Базовый expansion получает setup-policy явно и сохраняет
//! exact release→resize→restore-owner order. Player packet checks, listener
//! message assembly, codec, swap, clone и auction-scale mutation ниже остаются
//! RAW до замыкания соответствующих setup/player/message/goods owners.

use super::camountlimitgoodscontainer::{
    AmountLimitGoodsAdded, AmountLimitGoodsCleared, AmountLimitGoodsRelease, AmountLimitGoodsTaken,
    CAmountLimitGoodsContainer,
};
use super::cgoodscontainer::GoodsStackMergeOutcome;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_PARTICULAR_ATTRIBUTE;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::public::guid::CGuid;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VolumeExpandBlock {
    Disabled,
    ZeroAmount,
    ExceedsMaximum { current: u32, requested: u32 },
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

    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        let VolumeCell::Goods(ex_id) = *self.cells.get(position as usize)? else {
            return None;
        };
        self.base.find(ex_id)
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
        if !self.is_space_enough(position) {
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
// FUNCTION: CVolumeLimitGoodsContainer::Swap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:166
// RVA: 0x000DD390
// ADDRESS: 004dd390
// PROTOTYPE: int __thiscall Swap(ulong param_1, CGoods * param_2, CGoods * * param_3, void * param_4)
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
// FUNCTION: CVolumeLimitGoodsContainer::IsSpaceEnough
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodscontainer.cpp:140
// RVA: 0x000DE360
// ADDRESS: 004de360
// PROTOTYPE: int __thiscall IsSpaceEnough(vector<CGoods*,std::allocator<CGoods*>_> param_1)
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
