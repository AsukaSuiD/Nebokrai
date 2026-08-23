//! Позиционный storage-prefix `CBattleFairyContainer` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cbattlefairycontainer.cpp`.
//! Материализованы 17 фиксированных ячеек и exact positional add-фильтры по
//! goods type/addon marker. Gear-слоты публикуют ранний `BFPropertyAdd(+1)`
//! effect до base Add, поэтому отказ storage не отменяет этот effect.
//!
//! Автоматический overload читает неинициализированный `m_eBFEquipPlace` у
//! catalog owner-а. Rust выражает этот UB как typed block, а не выбирает
//! логичную ячейку из позднего C++-донора. Остальные combine/upgrade/summon и
//! player-integrated remove методы ниже пока остаются RAW.

use super::camountlimitgoodscontainer::{AmountLimitGoodsCleared, AmountLimitGoodsRelease};
use super::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeGoodsAddOutcome};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_BF_BATTLE_FAIRY, GAP_BF_BFEQUIPEMENT, GAP_BF_CLOTH, GAP_BF_FETCH_BODY, GAP_BF_FETCH_STONE,
    GAP_BF_GEM, GAP_BF_GLOVE, GAP_BF_HUXINJING, GAP_BF_JEWELLERY, GAP_BF_MATERIAL, GAP_BF_PIFENG,
    GAP_BF_WEAPON, GAP_BF_XIEZI, GAP_BF_YAODAI, GAP_GEM_TYPE, GOODS_TYPE_CONSUMABLE,
    GOODS_TYPE_EQUIPMENT, GOODS_TYPE_USELESS,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCell {
    Weapon = 0,
    Body = 1,
    Huxinjing = 2,
    Jewelry = 3,
    Glove = 4,
    Pifeng = 5,
    Yaodai = 6,
    Xiezi = 7,
    Material = 8,
    FetchStone = 9,
    FetchBody = 10,
    Battle = 11,
    Equipment = 12,
    GemBase = 13,
    GemOne = 14,
    GemTwo = 15,
    GemThree = 16,
}

impl BattleFairyCell {
    pub(crate) const fn from_position(position: u32) -> Option<Self> {
        Some(match position {
            0 => Self::Weapon,
            1 => Self::Body,
            2 => Self::Huxinjing,
            3 => Self::Jewelry,
            4 => Self::Glove,
            5 => Self::Pifeng,
            6 => Self::Yaodai,
            7 => Self::Xiezi,
            8 => Self::Material,
            9 => Self::FetchStone,
            10 => Self::FetchBody,
            11 => Self::Battle,
            12 => Self::Equipment,
            13 => Self::GemBase,
            14 => Self::GemOne,
            15 => Self::GemTwo,
            16 => Self::GemThree,
            _ => return None,
        })
    }

    pub(crate) const fn position(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPropertyAddEffect {
    pub(crate) cell: BattleFairyCell,
    pub(crate) delta: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyContainerAddBlock {
    MissingGoods,
    MissingBaseProperties { index: u32 },
    AutomaticAddRequiresEquipment { goods_type: i32 },
    UninitializedBattleFairyEquipPlace,
    InvalidPosition { position: u32 },
    GoodsRejected { cell: BattleFairyCell },
}

#[must_use = "outcome сохраняет ранний BFPropertyAdd effect и ownership incoming"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyContainerAddOutcome {
    Stored {
        base: VolumeGoodsAddOutcome,
        property_effect: Option<BattleFairyPropertyAddEffect>,
    },
    Rejected(BattleFairyContainerAddBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CBattleFairyContainer {
    base: CVolumeLimitGoodsContainer,
}

impl Default for CBattleFairyContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CBattleFairyContainer {
    pub(crate) fn new() -> Self {
        Self {
            base: CVolumeLimitGoodsContainer::new(),
        }
    }

    pub(crate) const fn base(&self) -> &CVolumeLimitGoodsContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.base
    }

    pub(crate) fn clear(&mut self) -> AmountLimitGoodsCleared {
        self.base.clear_goods()
    }

    pub(crate) fn release(&mut self) -> AmountLimitGoodsRelease {
        self.base.release()
    }

    pub(crate) fn add(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> BattleFairyContainerAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::MissingGoods,
            );
        };
        let index = goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(index) else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::MissingBaseProperties { index },
            );
        };
        if properties.goods_type() != GOODS_TYPE_EQUIPMENT {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::AutomaticAddRequiresEquipment {
                    goods_type: properties.goods_type(),
                },
            );
        }
        let Some(raw_place) = properties.battle_fairy_equip_place() else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::UninitializedBattleFairyEquipPlace,
            );
        };
        let Some(cell) = BattleFairyCell::from_position(raw_place as u32) else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::InvalidPosition {
                    position: raw_place as u32,
                },
            );
        };
        self.add_at(cell, incoming, factory, owner_progress_allows)
    }

    pub(crate) fn add_at(
        &mut self,
        cell: BattleFairyCell,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> BattleFairyContainerAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::MissingGoods,
            );
        };
        let index = goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(index) else {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::MissingBaseProperties { index },
            );
        };
        let value =
            |property_type, value_id| goods.addon_property_value(factory, property_type, value_id);
        let (allowed, applies_property) = match properties.goods_type() {
            GOODS_TYPE_EQUIPMENT => match cell {
                BattleFairyCell::Battle => (value(GAP_BF_BATTLE_FAIRY, 1) == 1, false),
                BattleFairyCell::Weapon => (value(GAP_BF_WEAPON, 1) == 1, true),
                BattleFairyCell::Huxinjing => (value(GAP_BF_HUXINJING, 1) == 1, true),
                BattleFairyCell::Body => (value(GAP_BF_CLOTH, 1) == 1, true),
                BattleFairyCell::Jewelry => (value(GAP_BF_JEWELLERY, 1) == 1, true),
                BattleFairyCell::Pifeng => (value(GAP_BF_PIFENG, 1) == 1, true),
                BattleFairyCell::Yaodai => (value(GAP_BF_YAODAI, 1) == 1, true),
                BattleFairyCell::Xiezi => (value(GAP_BF_XIEZI, 1) == 1, true),
                BattleFairyCell::Glove => (value(GAP_BF_GLOVE, 1) == 1, true),
                BattleFairyCell::Equipment => (value(GAP_BF_BFEQUIPEMENT, 2) == 1, false),
                _ => (false, false),
            },
            GOODS_TYPE_USELESS => match cell {
                BattleFairyCell::Material => (value(GAP_BF_MATERIAL, 1) == 1, false),
                BattleFairyCell::FetchBody => (value(GAP_BF_FETCH_BODY, 1) == 1, false),
                BattleFairyCell::FetchStone => (value(GAP_BF_FETCH_STONE, 1) == 1, false),
                _ => (false, false),
            },
            GOODS_TYPE_CONSUMABLE if value(GAP_BF_GEM, 1) == 1 => match cell {
                BattleFairyCell::GemBase => (value(GAP_GEM_TYPE, 1) == 1, false),
                BattleFairyCell::GemOne | BattleFairyCell::GemTwo | BattleFairyCell::GemThree => {
                    (value(GAP_GEM_TYPE, 1) == 2, false)
                }
                _ => (false, false),
            },
            _ => (false, false),
        };
        if !allowed {
            return BattleFairyContainerAddOutcome::Rejected(
                BattleFairyContainerAddBlock::GoodsRejected { cell },
            );
        }
        let property_effect =
            applies_property.then_some(BattleFairyPropertyAddEffect { cell, delta: 1 });
        BattleFairyContainerAddOutcome::Stored {
            base: self
                .base
                .add_goods_at(cell.position(), incoming, factory, owner_progress_allows),
            property_effect,
        }
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp

// ============================================================================
// FUNCTION: CBattleFairyContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:68
// RVA: 0x000DB670
// ADDRESS: 004db670
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:334
// RVA: 0x000DB690
// ADDRESS: 004db690
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:74
// RVA: 0x000FD460
// ADDRESS: 004fd460
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:339
// RVA: 0x000FD470
// ADDRESS: 004fd470
// PROTOTYPE: CBaseObject * __thiscall Remove(ulong param_1, ulong param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1815
// RVA: 0x000FD4A0
// ADDRESS: 004fd4a0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1821
// RVA: 0x000FD4B0
// ADDRESS: 004fd4b0
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:81
// RVA: 0x000FD510
// ADDRESS: 004fd510
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::GetSuccessResult
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1148
// RVA: 0x000FD840
// ADDRESS: 004fd840
// PROTOTYPE: ulong __thiscall GetSuccessResult(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::GetFailResult
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1216
// RVA: 0x000FD9A0
// ADDRESS: 004fd9a0
// PROTOTYPE: ulong __thiscall GetFailResult(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::GetProbability
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1269
// RVA: 0x000FDAB0
// ADDRESS: 004fdab0
// PROTOTYPE: ulong __thiscall GetProbability(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::GetUpgradePrice
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1316
// RVA: 0x000FDBF0
// ADDRESS: 004fdbf0
// PROTOTYPE: ulong __thiscall GetUpgradePrice(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::CheckBattleFairyCombine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:495
// RVA: 0x000FE780
// ADDRESS: 004fe780
// PROTOTYPE: eCombineResult __thiscall CheckBattleFairyCombine(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:773
// RVA: 0x000FECB0
// ADDRESS: 004fecb0
// PROTOTYPE: bool __thiscall DeleteGoods(eBattleFairy_Place_Cell param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::ResetPotential
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:800
// RVA: 0x000FEE20
// ADDRESS: 004fee20
// PROTOTYPE: void __thiscall ResetPotential(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::AllocatePotential
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:924
// RVA: 0x000FF480
// ADDRESS: 004ff480
// PROTOTYPE: void __thiscall AllocatePotential(int param_1, GOODS_ADDON_PROPERTIES param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::IsValidateUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1327
// RVA: 0x00100030
// ADDRESS: 00500030
// PROTOTYPE: bool __thiscall IsValidateUpgrade(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Upgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1394
// RVA: 0x001002E0
// ADDRESS: 005002e0
// PROTOTYPE: bool __thiscall Upgrade(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::ResetSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:1851
// RVA: 0x00101530
// ADDRESS: 00501530
// PROTOTYPE: void __thiscall ResetSkill(int param_1, int param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::SummonBF
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:2096
// RVA: 0x00101CB0
// ADDRESS: 00501cb0
// PROTOTYPE: void __thiscall SummonBF(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::BFPropertyAdd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:2286
// RVA: 0x001020E0
// ADDRESS: 005020e0
// PROTOTYPE: void __thiscall BFPropertyAdd(eBattleFairy_Place_Cell param_1, CGoods * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::LoadBFDefualtProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:2475
// RVA: 0x00102BC0
// ADDRESS: 00502bc0
// PROTOTYPE: void __thiscall LoadBFDefualtProperty(int param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:157
// RVA: 0x00103080
// ADDRESS: 00503080
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:290
// RVA: 0x00103310
// ADDRESS: 00503310
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::BatllteFairyCombine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:577
// RVA: 0x001034C0
// ADDRESS: 005034c0
// PROTOTYPE: bool __thiscall BatllteFairyCombine(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::CBattleFairyContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:46
// RVA: 0x00103FC0
// ADDRESS: 00503fc0
// PROTOTYPE: undefined __thiscall CBattleFairyContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::~CBattleFairyContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cbattlefairycontainer.cpp:61
// RVA: 0x00104190
// ADDRESS: 00504190
// PROTOTYPE: void __thiscall ~CBattleFairyContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
