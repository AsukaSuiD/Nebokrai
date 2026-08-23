//! Восьмислотовая DaKong shadow-сессия GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cequipmentdakongcontainer.cpp`.
//! Slot `0` принимает goods с instance-addon `GAP_DAKONG_1`, слоты
//! `1..7` — goods с `GAP_BAOSHI_COLOR`. Auto-add equipment всегда выбирает
//! `0`; auto-add gem ищет первый неразрешившийся слот не выше
//! `goods.QueryDaKongCount()`.
//!
//! Две map имеют разный lifecycle. Positional projection связывает
//! слот с shadow; consumption ledger запоминает gems для позднейшего
//! `DeleteGoods`. `Clear` сбрасывает projection/last GUID и восстанавливает
//! limit `8`, но сохраняет ledger. `Release` сбрасывает base и
//! projection, но сохраняет ledger/last GUID и limit `0`.
//!
//! Exact source move остаётся caller-boundary. Живой placed GUID заменяет
//! legacy dangling incoming GUID после stack merge. Typed effects фиксируют
//! overwrite и то, что при отказе base add ledger/last не откатываются.
//! Нижний pseudocode сохранён как первичная документация.

use std::collections::BTreeMap;

use super::camountlimitgoodsshadowcontainer::{
    AmountShadowAdded, CAmountLimitGoodsShadowContainer,
};
use super::ccontainer::PreviousContainer;
use super::cgoodsshadowcontainer::{
    GoodsShadow, PlacedShadowGoods, ShadowRecordBlock, ShadowSourceChangeOutcome,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{GAP_BAOSHI_COLOR, GAP_DAKONG_1};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::public::guid::CGuid;

const GOODS_LIMIT: u32 = 8;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DaKongCell {
    Equipment = 0,
    GemOne = 1,
    GemTwo = 2,
    GemThree = 3,
    GemFour = 4,
    GemFive = 5,
    GemSix = 6,
    GemSeven = 7,
}

impl DaKongCell {
    const GEMS: [Self; 7] = [
        Self::GemOne,
        Self::GemTwo,
        Self::GemThree,
        Self::GemFour,
        Self::GemFive,
        Self::GemSix,
        Self::GemSeven,
    ];

    pub(crate) const fn position(self) -> u32 {
        self as u32
    }

    pub(crate) const fn from_position(position: u32) -> Option<Self> {
        match position {
            0 => Some(Self::Equipment),
            1 => Some(Self::GemOne),
            2 => Some(Self::GemTwo),
            3 => Some(Self::GemThree),
            4 => Some(Self::GemFour),
            5 => Some(Self::GemFive),
            6 => Some(Self::GemSix),
            7 => Some(Self::GemSeven),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DaKongAddBlock {
    UnsupportedGoods,
    NoResolvableGemCell,
    InvalidPosition { position: u32 },
    InvalidEquipment,
    InvalidGem,
    Shadow(ShadowRecordBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DaKongAddEffects {
    pub(crate) cell: DaKongCell,
    pub(crate) displaced_position: Option<CGuid>,
    pub(crate) displaced_ledger: Option<CGuid>,
    pub(crate) previous_last_goods: Option<CGuid>,
}

#[must_use = "outcome фиксирует persistent ledger/last частичные эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DaKongAddOutcome {
    Added {
        effects: DaKongAddEffects,
        shadow: AmountShadowAdded,
    },
    Rejected {
        block: DaKongAddBlock,
        effects: Option<DaKongAddEffects>,
    },
}

#[must_use = "callback меняет shadow, ledger и positional projection"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DaKongSourceRemoved {
    pub(crate) invalidated_ledger_position: Option<u32>,
    pub(crate) shadow: ShadowSourceChangeOutcome,
    pub(crate) erased_cell: Option<DaKongCell>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CEquipmentDaKongContainer {
    base: CAmountLimitGoodsShadowContainer,
    positions: BTreeMap<DaKongCell, CGuid>,
    equipment_goods: BTreeMap<u32, CGuid>,
    last_goods: CGuid,
}

impl Default for CEquipmentDaKongContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CEquipmentDaKongContainer {
    pub(crate) const fn new() -> Self {
        let mut base = CAmountLimitGoodsShadowContainer::new();
        base.set_goods_amount_limit(GOODS_LIMIT);
        Self {
            base,
            positions: BTreeMap::new(),
            equipment_goods: BTreeMap::new(),
            last_goods: CGuid::GUID_INVALID,
        }
    }

    pub(crate) const fn base(&self) -> &CAmountLimitGoodsShadowContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CAmountLimitGoodsShadowContainer {
        &mut self.base
    }

    pub(crate) fn positions(&self) -> &BTreeMap<DaKongCell, CGuid> {
        &self.positions
    }

    pub(crate) fn equipment_goods(&self) -> &BTreeMap<u32, CGuid> {
        &self.equipment_goods
    }

    pub(crate) const fn last_goods(&self) -> CGuid {
        self.last_goods
    }

    pub(crate) const fn set_last_goods(&mut self, goods_id: CGuid) {
        self.last_goods = goods_id;
    }

    pub(crate) fn select_cell<'a, Resolve>(
        &self,
        goods: &CGoods,
        factory: &CGoodsFactory,
        mut resolve: Resolve,
    ) -> Result<DaKongCell, DaKongAddBlock>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        if goods.query_attribute(GAP_DAKONG_1) {
            return Ok(DaKongCell::Equipment);
        }
        if !goods.query_attribute(GAP_BAOSHI_COLOR) {
            return Err(DaKongAddBlock::UnsupportedGoods);
        }
        let maximum = goods.da_kong_count(factory).min(7);
        DaKongCell::GEMS
            .into_iter()
            .take(maximum as usize)
            .find(|cell| self.get_goods(*cell, &mut resolve).is_none())
            .ok_or(DaKongAddBlock::NoResolvableGemCell)
    }

    pub(crate) fn record_placed_goods(
        &mut self,
        cell: DaKongCell,
        goods: &CGoods,
        previous: PreviousContainer,
        placed_identity: CGuid,
        placed_position: u32,
    ) -> DaKongAddOutcome {
        if let Err(block) = self.validate_cell_goods(cell, goods) {
            return DaKongAddOutcome::Rejected {
                block,
                effects: None,
            };
        }

        let displaced_position = self.positions.insert(cell, placed_identity);
        let mut effects = DaKongAddEffects {
            cell,
            displaced_position,
            displaced_ledger: None,
            previous_last_goods: None,
        };
        match cell {
            DaKongCell::Equipment => {
                effects.previous_last_goods = Some(self.last_goods);
                self.last_goods = placed_identity;
            }
            _ => {
                effects.displaced_ledger = self
                    .equipment_goods
                    .insert(cell.position(), placed_identity);
            }
        }

        let placed = PlacedShadowGoods {
            identity: placed_identity,
            position: placed_position,
            base_properties_index: goods.base_properties_index(),
            amount: goods.amount(),
        };
        match self.base.record_placed_goods(previous, placed) {
            Ok(shadow) => DaKongAddOutcome::Added { effects, shadow },
            Err(block) => {
                // Exact rollback удаляет только positional entry. Ledger и
                // last GUID уже изменены и намеренно остаются изменёнными.
                self.positions.remove(&cell);
                DaKongAddOutcome::Rejected {
                    block: DaKongAddBlock::Shadow(block),
                    effects: Some(effects),
                }
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
    ) -> DaKongAddOutcome {
        let Some(cell) = DaKongCell::from_position(position) else {
            return DaKongAddOutcome::Rejected {
                block: DaKongAddBlock::InvalidPosition { position },
                effects: None,
            };
        };
        self.record_placed_goods(cell, goods, previous, placed_identity, placed_position)
    }

    pub(crate) fn get_goods<'a, Resolve>(
        &self,
        cell: DaKongCell,
        resolve: Resolve,
    ) -> Option<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.base.base().find(*self.positions.get(&cell)?, resolve)
    }

    /// Exact plural overload останавливается после первого match.
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

    pub(crate) fn on_source_removed(
        &mut self,
        goods_id: Option<CGuid>,
        amount: u32,
    ) -> DaKongSourceRemoved {
        let invalidated_ledger_position = goods_id.and_then(|goods_id| {
            self.equipment_goods
                .iter_mut()
                .find_map(|(position, candidate)| {
                    if *candidate == goods_id {
                        *candidate = CGuid::GUID_INVALID;
                        Some(*position)
                    } else {
                        None
                    }
                })
        });
        let goods_id = goods_id.unwrap_or(CGuid::GUID_INVALID);
        let shadow = self.base.base_mut().on_source_removed(goods_id, amount);
        let erased_cell = self.cell_for_goods(goods_id).inspect(|cell| {
            self.positions.remove(cell);
        });
        DaKongSourceRemoved {
            invalidated_ledger_position,
            shadow,
            erased_cell,
        }
    }

    pub(crate) fn clear(&mut self) -> usize {
        let count = self.base.clear();
        self.positions.clear();
        self.last_goods = CGuid::GUID_INVALID;
        self.base.set_goods_amount_limit(GOODS_LIMIT);
        count
    }

    pub(crate) fn release(&mut self) -> usize {
        let count = self.base.release();
        self.positions.clear();
        count
    }

    fn validate_cell_goods(&self, cell: DaKongCell, goods: &CGoods) -> Result<(), DaKongAddBlock> {
        match cell {
            DaKongCell::Equipment => goods
                .query_attribute(GAP_DAKONG_1)
                .then_some(())
                .ok_or(DaKongAddBlock::InvalidEquipment),
            _ => goods
                .query_attribute(GAP_BAOSHI_COLOR)
                .then_some(())
                .ok_or(DaKongAddBlock::InvalidGem),
        }
    }

    fn cell_for_goods(&self, goods_id: CGuid) -> Option<DaKongCell> {
        self.positions
            .iter()
            .find_map(|(cell, candidate)| (*candidate == goods_id).then_some(*cell))
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentdakongcontainer.cpp

// ============================================================================
// FUNCTION: CEquipmentDaKongContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentdakongcontainer.cpp:43
// RVA: 0x001DE670
// ADDRESS: 005de670
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentDaKongContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentdakongcontainer.cpp:169
// RVA: 0x001DE9B0
// ADDRESS: 005de9b0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentDaKongContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentdakongcontainer.cpp:180
// RVA: 0x001DEA10
// ADDRESS: 005dea10
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentDaKongContainer::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentdakongcontainer.cpp:315
// RVA: 0x001DEFB0
// ADDRESS: 005defb0
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentDaKongContainer::~CEquipmentDaKongContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentdakongcontainer.cpp:36
// RVA: 0x001DF610
// ADDRESS: 005df610
// PROTOTYPE: void __thiscall ~CEquipmentDaKongContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentDaKongContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentdakongcontainer.cpp:81
// RVA: 0x001DF700
// ADDRESS: 005df700
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentDaKongContainer::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentdakongcontainer.cpp:394
// RVA: 0x001DF890
// ADDRESS: 005df890
// PROTOTYPE: void __thiscall GetGoods(ulong param_1, vector<CGoods*,std::allocator<CGoods*>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentDaKongContainer::CEquipmentDaKongContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentdakongcontainer.cpp:22
// RVA: 0x001DF990
// ADDRESS: 005df990
// PROTOTYPE: undefined __thiscall CEquipmentDaKongContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
