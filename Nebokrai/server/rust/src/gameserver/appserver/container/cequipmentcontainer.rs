//! Позиционное owning-ядро `CEquipmentContainer` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cequipmentcontainer.cpp`. Семнадцать
//! колонок `0..=16`, их deterministic enum-order, lookup-ы, подсчёты, weight,
//! `Clear` и mode-зависимый `Release` материализованы по RVA
//! `0x000ED180..0x000EDB30` и `0x000EEBF0`. `BTreeMap` заменяет legacy
//! `std::map`, а owned `CGoods` — сырые указатели без изменения порядка.
//!
//! Exact EXE сравнивает число непустых колонок именно с 17. Вопреки позднему
//! архивному донору, `Clear` и `Release` не обнуляют `m_nExpantPkgNum`;
//! `Release` очищает external listeners через `CGoodsContainer::Release`, а
//! внутренний self-listener представлен прямым derived callback-ом и потому
//! не хранится как самоссылка. `Clear` возвращает ordered reports вместе с
//! владением товарами, а `Release` в test-mode возвращает detached товары;
//! это безопасная замена legacy pointer lifetime/`GarbageCollect`.
//!
//! Player policy, timed-add prefix, package-extension callbacks, swap,
//! fairy/battle-fairy и codec ниже остаются RAW до materialization связанных
//! owners; достигнутое storage-ядро не выдаётся за весь контейнер.

use std::collections::BTreeMap;

use super::ccontainer::ContainerListenerHandle;
use super::cgoodscontainer::{CGoodsContainer, GoodsContainerMode};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::public::guid::CGuid;

pub(crate) const EQUIPMENT_COLUMN_LIMIT: u32 = 17;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EquipmentColumn {
    Head = 0,
    Body = 1,
    Hand = 2,
    Glove = 3,
    Boot = 4,
    Jewelry = 5,
    OrnamentsOne = 6,
    OrnamentsTwo = 7,
    Medal = 8,
    Posterior = 9,
    Headgear = 10,
    Talisman = 11,
    Frock = 12,
    Wing = 13,
    Manteau = 14,
    Fairy = 15,
    LingBao = 16,
}

impl EquipmentColumn {
    pub(crate) const fn from_position(position: u32) -> Option<Self> {
        Some(match position {
            0 => Self::Head,
            1 => Self::Body,
            2 => Self::Hand,
            3 => Self::Glove,
            4 => Self::Boot,
            5 => Self::Jewelry,
            6 => Self::OrnamentsOne,
            7 => Self::OrnamentsTwo,
            8 => Self::Medal,
            9 => Self::Posterior,
            10 => Self::Headgear,
            11 => Self::Talisman,
            12 => Self::Frock,
            13 => Self::Wing,
            14 => Self::Manteau,
            15 => Self::Fairy,
            16 => Self::LingBao,
            _ => return None,
        })
    }

    pub(crate) const fn position(self) -> u32 {
        self as u32
    }
}

#[must_use = "report сохраняет порядок remove-callback-ов и владение товаром"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentClearedGoods {
    pub(crate) column: EquipmentColumn,
    pub(crate) goods: CGoods,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "в test-mode detached товары должны получить нового владельца"]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EquipmentReleaseReport {
    pub(crate) garbage_collected: Vec<ShapeIdentity>,
    pub(crate) detached: Vec<(EquipmentColumn, CGoods)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CEquipmentContainer {
    base: CGoodsContainer,
    equipment: BTreeMap<EquipmentColumn, CGoods>,
    expanded_package_num: u32,
}

impl Default for CEquipmentContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CEquipmentContainer {
    pub(crate) const fn new() -> Self {
        Self {
            base: CGoodsContainer::new(),
            equipment: BTreeMap::new(),
            expanded_package_num: 0,
        }
    }

    pub(crate) const fn base(&self) -> &CGoodsContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CGoodsContainer {
        &mut self.base
    }

    pub(crate) const fn expanded_package_num(&self) -> u32 {
        self.expanded_package_num
    }

    pub(crate) fn occupied_count(&self) -> u32 {
        self.equipment.len() as u32
    }

    /// Exact `IsFull` использует equality, а не `>=`.
    pub(crate) fn is_full(&self) -> bool {
        self.occupied_count() == EQUIPMENT_COLUMN_LIMIT
    }

    pub(crate) fn is_slot_empty(&self, column: EquipmentColumn) -> bool {
        !self.equipment.contains_key(&column)
    }

    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        self.equipment
            .get(&EquipmentColumn::from_position(position)?)
    }

    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.equipment
            .get_mut(&EquipmentColumn::from_position(position)?)
    }

    pub(crate) fn find(&self, goods_id: CGuid) -> Option<&CGoods> {
        self.equipment
            .values()
            .find(|goods| goods.identity().ex_id == goods_id)
    }

    pub(crate) fn query_goods_position_by_id(&self, goods_id: CGuid) -> Option<EquipmentColumn> {
        self.equipment
            .iter()
            .find_map(|(column, goods)| (goods.identity().ex_id == goods_id).then_some(*column))
    }

    /// Typed replacement исходного pointer-identity overload-а.
    pub(crate) fn query_goods_position(&self, goods: &CGoods) -> Option<EquipmentColumn> {
        self.equipment
            .iter()
            .find_map(|(column, stored)| std::ptr::eq(stored, goods).then_some(*column))
    }

    pub(crate) fn get_first_goods(&self, base_properties_index: u32) -> Option<&CGoods> {
        self.equipment
            .values()
            .find(|goods| goods.base_properties_index() == base_properties_index)
    }

    pub(crate) fn get_goods_by_base_properties(&self, base_properties_index: u32) -> Vec<&CGoods> {
        self.equipment
            .values()
            .filter(|goods| goods.base_properties_index() == base_properties_index)
            .collect()
    }

    pub(crate) fn is_goods_existed(&self, base_properties_index: u32) -> bool {
        self.get_first_goods(base_properties_index).is_some()
    }

    pub(crate) fn traversing_goods(&self) -> Vec<(EquipmentColumn, &CGoods)> {
        self.equipment
            .iter()
            .map(|(column, goods)| (*column, goods))
            .collect()
    }

    pub(crate) fn contents_weight(&self, factory: &CGoodsFactory) -> u32 {
        self.equipment
            .values()
            .fold(0, |total, goods| total.wrapping_add(goods.weight(factory)))
    }

    /// Exact owner считает только товары с живой catalog-записью.
    pub(crate) fn goods_amount(&self, factory: &CGoodsFactory) -> u32 {
        self.equipment
            .values()
            .filter(|goods| {
                factory
                    .query_goods_base_properties(goods.base_properties_index())
                    .is_some()
            })
            .count() as u32
    }

    /// Internal self-callback должен быть применён dispatcher-ом перед
    /// перечисленными external listeners для каждого report-а.
    pub(crate) fn clear(&mut self) -> Vec<EquipmentClearedGoods> {
        let listeners = self.base.base().listeners().to_vec();
        std::mem::take(&mut self.equipment)
            .into_iter()
            .map(|(column, goods)| EquipmentClearedGoods {
                column,
                goods,
                listeners: listeners.clone(),
            })
            .collect()
    }

    pub(crate) fn release(&mut self) -> EquipmentReleaseReport {
        let mode = self.base.container_mode();
        let equipment = std::mem::take(&mut self.equipment);
        let mut report = EquipmentReleaseReport::default();
        match mode {
            GoodsContainerMode::Normal => {
                report.garbage_collected = equipment
                    .into_values()
                    .map(|goods| goods.identity())
                    .collect();
            }
            GoodsContainerMode::Test => {
                report.detached = equipment.into_iter().collect();
            }
        }
        self.base.release();
        report
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp

// ============================================================================
// FUNCTION: CEquipmentContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:46
// RVA: 0x000ED180
// ADDRESS: 004ed180
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:127
// RVA: 0x000ED480
// ADDRESS: 004ed480
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:153
// RVA: 0x000ED5A0
// ADDRESS: 004ed5a0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::TraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:171
// RVA: 0x000ED680
// ADDRESS: 004ed680
// PROTOTYPE: void __thiscall TraversingContainer(CContainerListener * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetContentsWeight
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:191
// RVA: 0x000ED700
// ADDRESS: 004ed700
// PROTOTYPE: ulong __thiscall GetContentsWeight(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::IsGoodsExisted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:208
// RVA: 0x000ED770
// ADDRESS: 004ed770
// PROTOTYPE: int __thiscall IsGoodsExisted(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:396
// RVA: 0x000ED800
// ADDRESS: 004ed800
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:699
// RVA: 0x000ED890
// ADDRESS: 004ed890
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::IsFull
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:714
// RVA: 0x000ED900
// ADDRESS: 004ed900
// PROTOTYPE: int __thiscall IsFull(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::QueryGoodsPosition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:736
// RVA: 0x000ED970
// ADDRESS: 004ed970
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGoods * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::QueryGoodsPosition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:760
// RVA: 0x000ED9D0
// ADDRESS: 004ed9d0
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGUID * param_1, ulong * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetTheFirstGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:781
// RVA: 0x000EDA60
// ADDRESS: 004eda60
// PROTOTYPE: CGoods * __thiscall GetTheFirstGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:799
// RVA: 0x000EDAF0
// ADDRESS: 004edaf0
// PROTOTYPE: CGoods * __thiscall GetGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetGoodsAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:963
// RVA: 0x000EDB30
// ADDRESS: 004edb30
// PROTOTYPE: ulong __thiscall GetGoodsAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:1003
// RVA: 0x000EDBB0
// ADDRESS: 004edbb0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::~CEquipmentContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:39
// RVA: 0x000EE410
// ADDRESS: 004ee410
// PROTOTYPE: void __thiscall ~CEquipmentContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::FairyExpUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:1075
// RVA: 0x000EE4A0
// ADDRESS: 004ee4a0
// PROTOTYPE: EExpUpResult __thiscall FairyExpUp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::BFLevelUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:1149
// RVA: 0x000EE860
// ADDRESS: 004ee860
// PROTOTYPE: eExpUpResult __thiscall BFLevelUp(int param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::CEquipmentContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:25
// RVA: 0x000EEBF0
// ADDRESS: 004eebf0
// PROTOTYPE: undefined __thiscall CEquipmentContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Swap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:227
// RVA: 0x000EECB0
// ADDRESS: 004eecb0
// PROTOTYPE: int __thiscall Swap(EQUIPMENT_COLUMN param_1, CGoods * param_2, CGoods * * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:418
// RVA: 0x000EEF90
// ADDRESS: 004eef90
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:492
// RVA: 0x000EF2F0
// ADDRESS: 004ef2f0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:813
// RVA: 0x000EF5B0
// ADDRESS: 004ef5b0
// PROTOTYPE: void __thiscall GetGoods(ulong param_1, vector<CGoods*,std::allocator<CGoods*>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::OnObjectAdded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:829
// RVA: 0x000EF6C0
// ADDRESS: 004ef6c0
// PROTOTYPE: int __thiscall OnObjectAdded(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:894
// RVA: 0x000EF9A0
// ADDRESS: 004ef9a0
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentContainer::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cequipmentcontainer.cpp:1024
// RVA: 0x000EFC30
// ADDRESS: 004efc30
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
