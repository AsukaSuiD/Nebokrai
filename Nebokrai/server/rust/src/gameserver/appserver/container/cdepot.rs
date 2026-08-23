//! Lock/position/expansion core `CDepot` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cdepot.cpp`. Depot оборачивает
//! `CVolumeLimitGoodsContainer`, стартует locked и разрешает storage mutation
//! только после внешне подтверждённого player-password check. Базовые позиции
//! `0..95` обычные; extension-группы начинаются в `96 + 13n`, а anchor goods
//! активирует остальные двенадцать позиций группы.
//!
//! `Vec`/`IndexMap` базы остаются библиотечным storage-слоем. Здесь сохранены
//! exact position selection, inactive-anchor и expansion partial effects.
//! Extension-item add/remove callbacks, player notifications и codec restore
//! ниже остаются RAW до замыкания goods/player/message owners.

use super::camountlimitgoodscontainer::{AmountLimitGoodsCleared, AmountLimitGoodsRelease};
use super::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeExpandOutcome, VolumeGoodsAddBlock, VolumeGoodsAddOutcome,
    VolumeGoodsRemoveOutcome,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_GOODS_PACKAGE_EXTENTION, GAP_PARTICULAR_ATTRIBUTE,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::public::guid::CGuid;

const DEPOT_BASE_CELLS: u32 = 96;
const DEPOT_EXTENSION_WIDTH: u32 = 13;
const DEPOT_EXTENSION_END: u32 = 161;

#[must_use = "expansion может изменить storage даже при false legacy return"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DepotExpandOutcome {
    pub(crate) base: VolumeExpandOutcome,
    pub(crate) size: u32,
    pub(crate) initialized_anchors: u32,
    pub(crate) legacy_success: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DepotGoodsAddBlock {
    Locked,
    MissingGoods,
    InvalidCurrency {
        base_properties_index: u32,
        position: u32,
    },
    NoSpace,
    PositionUnavailable {
        position: u32,
    },
    ExtensionSlotOccupied {
        position: u32,
    },
    ExtensionGroupInactive {
        position: u32,
    },
    ExtensionItemRequired {
        position: u32,
    },
    InvalidExtensionKind {
        position: u32,
        kind: i32,
    },
    MissingBaseProperties {
        base_properties_index: u32,
    },
    AmountLimitReached,
}

#[must_use = "результат depot add определяет ownership и message/listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DepotGoodsAddOutcome {
    Volume(VolumeGoodsAddOutcome),
    Rejected(DepotGoodsAddBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CDepot {
    base: CVolumeLimitGoodsContainer,
    locked: bool,
}

impl Default for CDepot {
    fn default() -> Self {
        Self::new()
    }
}

impl CDepot {
    pub(crate) fn new() -> Self {
        Self {
            base: CVolumeLimitGoodsContainer::new(),
            locked: true,
        }
    }

    pub(crate) const fn base(&self) -> &CVolumeLimitGoodsContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.base
    }

    pub(crate) const fn is_locked(&self) -> bool {
        self.locked
    }

    pub(crate) fn lock(&mut self) -> bool {
        self.locked = true;
        true
    }

    /// Player/game owner выполняет owner lookup и byte-exact password compare;
    /// неуспех оставляет текущее состояние без изменений.
    pub(crate) fn unlock_if_authenticated(&mut self, authenticated: bool) -> bool {
        if !authenticated {
            return false;
        }
        self.locked = false;
        true
    }

    pub(crate) fn find(&self, ex_id: CGuid) -> Option<&CGoods> {
        if self.locked {
            return None;
        }
        self.base.base().find(ex_id)
    }

    pub(crate) fn remove_goods(&mut self, ex_id: CGuid) -> Option<VolumeGoodsRemoveOutcome> {
        if self.locked {
            return None;
        }
        self.base.remove_goods(ex_id)
    }

    pub(crate) fn add_goods(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> DepotGoodsAddOutcome {
        if self.locked {
            return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::Locked);
        }
        let Some(goods) = incoming.as_ref() else {
            return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::MissingGoods);
        };
        let Some(position) = self.find_position_for_goods(goods, factory) else {
            return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::NoSpace);
        };
        self.add_goods_at(position, incoming, factory, owner_progress_allows)
    }

    pub(crate) fn add_goods_at(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> DepotGoodsAddOutcome {
        if self.locked {
            return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::Locked);
        }
        let Some(goods) = incoming.as_ref() else {
            return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::MissingGoods);
        };
        let base_properties_index = goods.base_properties_index();
        if base_properties_index == factory.get_gold_coin_index()
            || base_properties_index == factory.get_yuan_bao_index()
        {
            return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::InvalidCurrency {
                base_properties_index,
                position,
            });
        }

        if self.base.get_goods(position).is_some() {
            if Self::is_extension_item_position(position) {
                return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::ExtensionSlotOccupied {
                    position,
                });
            }
            if DEPOT_BASE_CELLS <= position && !self.is_activated(position) {
                return DepotGoodsAddOutcome::Rejected(
                    DepotGoodsAddBlock::ExtensionGroupInactive { position },
                );
            }
            return Self::from_volume_add(
                self.base
                    .add_goods_at(position, incoming, factory, owner_progress_allows),
                position,
                base_properties_index,
            );
        }

        if !self.is_space_enough(position) {
            return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::PositionUnavailable {
                position,
            });
        }
        if Self::is_extension_item_position(position) {
            if !goods.query_attribute(GAP_GOODS_PACKAGE_EXTENTION) {
                return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::ExtensionItemRequired {
                    position,
                });
            }
            let kind = goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 1);
            if kind != 1 {
                return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::InvalidExtensionKind {
                    position,
                    kind,
                });
            }
            return Self::from_volume_add(
                self.base.add_goods_at_available_or_inactive(
                    position,
                    incoming,
                    factory,
                    owner_progress_allows,
                ),
                position,
                base_properties_index,
            );
        }

        if DEPOT_BASE_CELLS <= position
            && !self.is_activated(position)
            && !self.inactive_group_allows_add(position)
        {
            return DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::ExtensionGroupInactive {
                position,
            });
        }
        Self::from_volume_add(
            self.base
                .add_goods_at(position, incoming, factory, owner_progress_allows),
            position,
            base_properties_index,
        )
    }

    fn from_volume_add(
        outcome: VolumeGoodsAddOutcome,
        position: u32,
        base_properties_index: u32,
    ) -> DepotGoodsAddOutcome {
        let VolumeGoodsAddOutcome::Rejected(block) = outcome else {
            return DepotGoodsAddOutcome::Volume(outcome);
        };
        match block {
            VolumeGoodsAddBlock::MissingGoods => {
                DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::MissingGoods)
            }
            VolumeGoodsAddBlock::InvalidCurrency => {
                DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::InvalidCurrency {
                    base_properties_index,
                    position,
                })
            }
            VolumeGoodsAddBlock::PositionUnavailable => {
                DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::PositionUnavailable { position })
            }
            VolumeGoodsAddBlock::MissingBaseProperties => {
                DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::MissingBaseProperties {
                    base_properties_index,
                })
            }
            VolumeGoodsAddBlock::AmountLimitReached => {
                DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::AmountLimitReached)
            }
            VolumeGoodsAddBlock::NoSpace => {
                DepotGoodsAddOutcome::Rejected(DepotGoodsAddBlock::NoSpace)
            }
        }
    }

    pub(crate) fn clear_goods(&mut self) -> AmountLimitGoodsCleared {
        self.locked = true;
        self.base.clear_goods()
    }

    pub(crate) fn release(&mut self) -> AmountLimitGoodsRelease {
        self.locked = true;
        self.base.release()
    }

    pub(crate) fn is_extension_item_position(position: u32) -> bool {
        if position < DEPOT_BASE_CELLS {
            return false;
        }
        let mut anchor = DEPOT_BASE_CELLS;
        while anchor < DEPOT_EXTENSION_END {
            if position == anchor {
                return true;
            }
            anchor = anchor.wrapping_add(DEPOT_EXTENSION_WIDTH);
        }
        false
    }

    pub(crate) fn is_activated(&self, position: u32) -> bool {
        if position < DEPOT_BASE_CELLS || self.base.size() < position {
            return false;
        }
        let group = position.wrapping_sub(DEPOT_BASE_CELLS) / DEPOT_EXTENSION_WIDTH;
        let anchor = group
            .wrapping_mul(DEPOT_EXTENSION_WIDTH)
            .wrapping_add(DEPOT_BASE_CELLS);
        self.base.get_goods(anchor).is_some()
    }

    pub(crate) fn inactive_group_allows_add(&self, position: u32) -> bool {
        if position < DEPOT_BASE_CELLS || self.base.size() < position {
            return false;
        }
        let group = position.wrapping_sub(DEPOT_BASE_CELLS) / DEPOT_EXTENSION_WIDTH;
        let anchor = group
            .wrapping_mul(DEPOT_EXTENSION_WIDTH)
            .wrapping_add(DEPOT_BASE_CELLS);
        self.base.is_cell_inactive(anchor)
    }

    pub(crate) fn is_space_enough(&self, position: u32) -> bool {
        if self.base.is_space_enough(position) {
            return true;
        }
        position < self.base.size()
            && Self::is_extension_item_position(position)
            && self.base.is_cell_inactive(position)
    }

    pub(crate) fn find_empty_space_for_goods(&self) -> Option<u32> {
        for position in 0..self.base.size() {
            if !self.base.is_space_enough(position) {
                continue;
            }
            if position < DEPOT_BASE_CELLS {
                return Some(position);
            }
            if Self::is_extension_item_position(position) {
                continue;
            }
            if self.is_activated(position) {
                return Some(position);
            }
        }
        None
    }

    pub(crate) fn find_position_for_goods(
        &self,
        incoming: &CGoods,
        factory: &CGoodsFactory,
    ) -> Option<u32> {
        let maximum = incoming.max_stack_number(factory);
        if 1 < maximum {
            for stored in self.base.base().traversing_goods() {
                if stored.base_properties_index() != incoming.base_properties_index()
                    || maximum < incoming.amount().wrapping_add(stored.amount())
                    || incoming.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1)
                        != stored.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1)
                {
                    continue;
                }
                let Some(position) = self.base.query_goods_position(stored.identity().ex_id) else {
                    continue;
                };
                if position < DEPOT_BASE_CELLS {
                    return Some(position);
                }
                if Self::is_extension_item_position(position) {
                    return None;
                }
                return self.is_activated(position).then_some(position);
            }
        }
        self.find_empty_space_for_goods()
    }

    pub(crate) fn expand(&mut self, requested: u32, expansion_enabled: bool) -> DepotExpandOutcome {
        let base = self.base.expand(requested, expansion_enabled);
        let size = self.base.size();
        let mut anchor = DEPOT_BASE_CELLS;
        let mut initialized_anchors = 0u32;
        while anchor < DEPOT_EXTENSION_END {
            if size <= anchor {
                return DepotExpandOutcome {
                    base,
                    size,
                    initialized_anchors,
                    legacy_success: false,
                };
            }
            let initialized = self.base.set_cell_inactive(anchor);
            debug_assert!(initialized, "expanded depot anchor обязан существовать");
            initialized_anchors = initialized_anchors.wrapping_add(u32::from(initialized));
            anchor = anchor.wrapping_add(DEPOT_EXTENSION_WIDTH);
        }
        DepotExpandOutcome {
            base,
            size,
            initialized_anchors,
            legacy_success: true,
        }
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.h

// ============================================================================
// FUNCTION: CDepot::CDepot
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:21
// RVA: 0x000E1650
// ADDRESS: 004e1650
// PROTOTYPE: undefined __thiscall CDepot(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:41
// RVA: 0x000E1670
// ADDRESS: 004e1670
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:50
// RVA: 0x000E1690
// ADDRESS: 004e1690
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:59
// RVA: 0x000E16A0
// ADDRESS: 004e16a0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Lock
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:201
// RVA: 0x000E16B0
// ADDRESS: 004e16b0
// PROTOTYPE: int __thiscall Lock(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:231
// RVA: 0x000E16C0
// ADDRESS: 004e16c0
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:243
// RVA: 0x000E16D0
// ADDRESS: 004e16d0
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::IsDepotLocked
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:296
// RVA: 0x000E16E0
// ADDRESS: 004e16e0
// PROTOTYPE: int __thiscall IsDepotLocked(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::IsActived
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:315
// RVA: 0x000E16F0
// ADDRESS: 004e16f0
// PROTOTYPE: int __thiscall IsActived(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::IsExtentionItemPos
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.h:61
// RVA: 0x000E1750
// ADDRESS: 004e1750
// PROTOTYPE: int __thiscall IsExtentionItemPos(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::~CDepot
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:34
// RVA: 0x000E1780
// ADDRESS: 004e1780
// PROTOTYPE: void __thiscall ~CDepot(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:254
// RVA: 0x000E17E0
// ADDRESS: 004e17e0
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:300
// RVA: 0x000E18A0
// ADDRESS: 004e18a0
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::NoActiveButAddItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:332
// RVA: 0x000E1910
// ADDRESS: 004e1910
// PROTOTYPE: int __thiscall NoActiveButAddItem(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Expant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:351
// RVA: 0x000E1980
// ADDRESS: 004e1980
// PROTOTYPE: int __thiscall Expant(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::FindEmptySpaceForGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:377
// RVA: 0x000E19E0
// ADDRESS: 004e19e0
// PROTOTYPE: int __thiscall FindEmptySpaceForGoods(ulong * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::FindPositionForGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:400
// RVA: 0x000E1A70
// ADDRESS: 004e1a70
// PROTOTYPE: int __thiscall FindPositionForGoods(CGoods * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::IsSpaceEnough
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:431
// RVA: 0x000E1B70
// ADDRESS: 004e1b70
// PROTOTYPE: int __thiscall IsSpaceEnough(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Unlock
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:210
// RVA: 0x000E1C10
// ADDRESS: 004e1c10
// PROTOTYPE: int __thiscall Unlock(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDepot::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cdepot.cpp:72
// RVA: 0x000E1CC0
// ADDRESS: 004e1cc0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
