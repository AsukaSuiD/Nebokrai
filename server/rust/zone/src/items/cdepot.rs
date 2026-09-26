//! Lock/position/expansion core `CDepot` исторического GameServer: locked
//! склад поверх `CVolumeLimitGoodsContainer` с базовыми позициями `0..95` и
//! extension-anchor группами по 13 ячеек.
//!
//! Player-владелец публикует этот контейнер под extend-id
//! [`PlayerContainerKind::Depot`][crate::items::playercontainers::PlayerContainerKind]
//! единого каталога (дизайн D4); константы `96/13/161` ниже — внутренняя
//! разметка позиций склада (база 96 ячеек, шаг extension-группы 13, конец
//! расширения `96 + 13·5 = 161`), а не wire-номера.
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
//! Достигнутый `0x90301` move/stack caller сохраняет запрет извлечения anchor,
//! kind-1 activation, GoodsAI/listener effects и rollback. Hand-owned
//! `OT_SWITCH_OBJECT` проходит через depot guard и общий volume swap. Persisted
//! restore очищает исходные 96 ячеек, при `bToAdd` расширяет их на 65,
//! размечает extension-anchor и загружает goods через depot-specific Add;
//! lock возвращается при любом результате. Extension-remove listener после
//! базовой message-разметки лишь повторно запрашивает позицию и не добавляет
//! observable mutation.

use super::camountlimitgoodscontainer::{
    AmountLimitGoodsCleared, AmountLimitGoodsCodecError, AmountLimitGoodsRelease,
};
use super::cgoods::CGoods;
use super::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeExpandOutcome, VolumeGoodsAddBlock, VolumeGoodsAddOutcome,
    VolumeGoodsCodecError, VolumeGoodsRemoveOutcome, VolumeGoodsSwapOutcome, read_volume_wire_u32,
};
use crate::content::goods::{GAP_GOODS_PACKAGE_EXTENTION, GAP_PARTICULAR_ATTRIBUTE};
use crate::content::goodsfactory::CGoodsFactory;
use nebokrai_shared::values::CGuid;

const DEPOT_BASE_CELLS: u32 = 96;
const DEPOT_EXTENSION_WIDTH: u32 = 13;
const DEPOT_EXTENSION_END: u32 = 161;

#[must_use = "expansion может изменить storage даже при false legacy return"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepotExpandOutcome {
    pub base: VolumeExpandOutcome,
    pub size: u32,
    pub initialized_anchors: u32,
    pub legacy_success: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DepotGoodsAddBlock {
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
pub enum DepotGoodsAddOutcome {
    Volume(VolumeGoodsAddOutcome),
    Rejected(DepotGoodsAddBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CDepot {
    base: CVolumeLimitGoodsContainer,
    locked: bool,
}

impl Default for CDepot {
    fn default() -> Self {
        Self::new()
    }
}

impl CDepot {
    pub fn new() -> Self {
        Self {
            base: CVolumeLimitGoodsContainer::new(),
            locked: true,
        }
    }

    pub const fn base(&self) -> &CVolumeLimitGoodsContainer {
        &self.base
    }

    pub const fn base_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.base
    }

    pub const fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn lock(&mut self) -> bool {
        self.locked = true;
        true
    }

    /// Player/game owner выполняет owner lookup и byte-exact password compare;
    /// неуспех оставляет текущее состояние без изменений.
    pub fn unlock_if_authenticated(&mut self, authenticated: bool) -> bool {
        if !authenticated {
            return false;
        }
        self.locked = false;
        true
    }

    pub fn find(&self, ex_id: CGuid) -> Option<&CGoods> {
        if self.locked {
            return None;
        }
        self.base.base().find(ex_id)
    }

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        if self.locked {
            return None;
        }
        self.base.get_goods(position)
    }

    pub fn goods_amount(&self, factory: &CGoodsFactory) -> u32 {
        self.base.goods_amount(factory)
    }

    pub fn snapshot_goods(&self) -> impl Iterator<Item = (u32, &CGoods)> {
        self.base.base().traversing_goods().filter_map(|goods| {
            self.base
                .query_goods_position(goods.identity().ex_id)
                .map(|position| (position, goods))
        })
    }

    pub fn remove_goods(&mut self, ex_id: CGuid) -> Option<VolumeGoodsRemoveOutcome> {
        if self.locked {
            return None;
        }
        self.base.remove_goods(ex_id)
    }

    /// `CC2SContainerObjectMove::GetGoods` запрещает клиентское извлечение
    /// extension anchor до virtual `Remove(position, amount)`. Обычные слоты
    /// сохраняют общий full/partial ownership pass volume-container-а.
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
        if self.locked || Self::is_extension_item_position(position) {
            return None;
        }
        self.base
            .take_goods(position, requested_amount, factory, create_goods)
    }

    pub fn add_goods(
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

    pub fn add_goods_at(
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

    /// `SwapGoods` caller уже проверил source hand. Depot дополнительно
    /// запрещает lock, extension anchor и неактивную extension-группу.
    pub fn swap_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> Option<VolumeGoodsSwapOutcome> {
        if self.locked
            || Self::is_extension_item_position(position)
            || position >= DEPOT_BASE_CELLS && !self.is_activated(position)
        {
            return None;
        }
        self.base
            .swap_goods(position, incoming, factory, owner_progress_allows)
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

    pub fn clear_goods(&mut self) -> AmountLimitGoodsCleared {
        self.locked = true;
        self.base.clear_goods()
    }

    pub fn release(&mut self) -> AmountLimitGoodsRelease {
        self.locked = true;
        self.base.release()
    }

    pub fn is_extension_item_position(position: u32) -> bool {
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

    pub fn is_activated(&self, position: u32) -> bool {
        if position < DEPOT_BASE_CELLS || self.base.size() < position {
            return false;
        }
        let group = position.wrapping_sub(DEPOT_BASE_CELLS) / DEPOT_EXTENSION_WIDTH;
        let anchor = group
            .wrapping_mul(DEPOT_EXTENSION_WIDTH)
            .wrapping_add(DEPOT_BASE_CELLS);
        self.base.get_goods(anchor).is_some()
    }

    pub fn inactive_group_allows_add(&self, position: u32) -> bool {
        if position < DEPOT_BASE_CELLS || self.base.size() < position {
            return false;
        }
        let group = position.wrapping_sub(DEPOT_BASE_CELLS) / DEPOT_EXTENSION_WIDTH;
        let anchor = group
            .wrapping_mul(DEPOT_EXTENSION_WIDTH)
            .wrapping_add(DEPOT_BASE_CELLS);
        self.base.is_cell_inactive(anchor)
    }

    pub fn is_space_enough(&self, position: u32) -> bool {
        if self.base.is_space_enough(position) {
            return true;
        }
        position < self.base.size()
            && Self::is_extension_item_position(position)
            && self.base.is_cell_inactive(position)
    }

    pub fn find_empty_space_for_goods(&self) -> Option<u32> {
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

    pub fn find_position_for_goods(
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

    pub fn expand(&mut self, requested: u32, expansion_enabled: bool) -> DepotExpandOutcome {
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

    pub fn serialize(&self, destination: &mut Vec<u8>, factory: &CGoodsFactory) -> bool {
        self.base.serialize(destination, factory)
    }

    pub fn unserialize<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        factory: &CGoodsFactory,
        expansion_enabled: bool,
        ordinary_threshold: OrdinaryThreshold,
        battle_threshold: BattleThreshold,
    ) -> Result<(), VolumeGoodsCodecError>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        let mut ordinary_threshold = ordinary_threshold;
        let mut battle_threshold = battle_threshold;
        self.locked = true;
        let _cleared = self.base.clear_goods();
        if expansion_enabled {
            let _legacy_ignored = self.expand(0x41, true);
        }
        self.locked = false;
        let result = (|| {
            let count = read_volume_wire_u32(source, cursor, "depot goods count")?;
            for _ in 0..count {
                let position =
                    read_volume_wire_u32(source, cursor, "depot goods position")?;
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
                let _legacy_ignored =
                    self.add_goods_at(position, &mut incoming, factory, true);
            }
            Ok(())
        })();
        self.locked = true;
        result
    }
}
