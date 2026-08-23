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
use crate::public::guid::CGuid;

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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:170
// RVA: 0x001DDDA0
// ADDRESS: 005ddda0
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::IsFull
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:225
// RVA: 0x001DDDD0
// ADDRESS: 005dddd0
// PROTOTYPE: int __thiscall IsFull(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::QueryGoodsPosition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:262
// RVA: 0x001DDE20
// ADDRESS: 005dde20
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGUID * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:278
// RVA: 0x001DDE80
// ADDRESS: 005dde80
// PROTOTYPE: CGoods * __thiscall GetGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::IsSpaceEnough
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:344
// RVA: 0x001DDEF0
// ADDRESS: 005ddef0
// PROTOTYPE: int __thiscall IsSpaceEnough(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:48
// RVA: 0x001DDF40
// ADDRESS: 005ddf40
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::FindPositionForGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:313
// RVA: 0x001DE180
// ADDRESS: 005de180
// PROTOTYPE: int __thiscall FindPositionForGoods(CGoods * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:30
// RVA: 0x001DE250
// ADDRESS: 005de250
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:194
// RVA: 0x001DE2C0
// ADDRESS: 005de2c0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::~CVolumeLimitGoodsShadowContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:24
// RVA: 0x001DE2E0
// ADDRESS: 005de2e0
// PROTOTYPE: void __thiscall ~CVolumeLimitGoodsShadowContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::RemoveShadow
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:128
// RVA: 0x001DE360
// ADDRESS: 005de360
// PROTOTYPE: int __thiscall RemoveShadow(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::CVolumeLimitGoodsShadowContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:19
// RVA: 0x001DE520
// ADDRESS: 005de520
// PROTOTYPE: undefined __thiscall CVolumeLimitGoodsShadowContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:184
// RVA: 0x001DE570
// ADDRESS: 005de570
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::Clone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:204
// RVA: 0x001DE5C0
// ADDRESS: 005de5c0
// PROTOTYPE: int __thiscall Clone(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVolumeLimitGoodsShadowContainer::SetContainerVolume
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cvolumelimitgoodsshadowcontainer.cpp:295
// RVA: 0x001DE610
// ADDRESS: 005de610
// PROTOTYPE: void __thiscall SetContainerVolume(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
