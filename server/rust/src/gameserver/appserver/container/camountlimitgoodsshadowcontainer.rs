//! Count-limit слой goods shadow container исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/camountlimitgoodsshadowcontainer.cpp`.
//! Maximum начинается с нуля, поэтому новый container full до явного setup.
//! Успешная запись metadata сразу формирует `AddShadow` report; фактический GUID
//! placement заменяет legacy повторное чтение потенциально уже уничтоженного
//! incoming pointer-а после stack merge.
//!
//! `BTreeMap` и resolver mechanics принадлежат base owner-у. Clone target и
//! packet assembly ниже остаются RAW до их concrete caller-ов.

use super::ccontainer::PreviousContainer;
use super::cgoodsshadowcontainer::{
    CGoodsShadowContainer, PlacedShadowGoods, ShadowPresenceReport, ShadowRecordBlock,
    ShadowRecorded,
};

#[must_use = "успешный add содержит metadata и обязательный AddShadow report"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AmountShadowAdded {
    pub(crate) recorded: ShadowRecorded,
    pub(crate) presence: ShadowPresenceReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CAmountLimitGoodsShadowContainer {
    base: CGoodsShadowContainer,
    max_goods_amount: u32,
}

impl Default for CAmountLimitGoodsShadowContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CAmountLimitGoodsShadowContainer {
    pub(crate) const fn new() -> Self {
        Self {
            base: CGoodsShadowContainer::new(),
            max_goods_amount: 0,
        }
    }

    pub(crate) const fn base(&self) -> &CGoodsShadowContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CGoodsShadowContainer {
        &mut self.base
    }

    pub(crate) const fn set_goods_amount_limit(&mut self, limit: u32) {
        self.max_goods_amount = limit;
    }

    pub(crate) const fn goods_amount_limit(&self) -> u32 {
        self.max_goods_amount
    }

    pub(crate) fn is_full(&self) -> bool {
        self.max_goods_amount <= self.base.goods_amount()
    }

    pub(crate) fn record_placed_goods(
        &mut self,
        previous: PreviousContainer,
        placed: PlacedShadowGoods,
    ) -> Result<AmountShadowAdded, ShadowRecordBlock> {
        let recorded = self
            .base
            .record_placed_goods(previous, placed, !self.is_full())?;
        let presence = self
            .base
            .add_shadow_report(recorded.record.goods_id)
            .expect("записанный amount shadow обязан существовать");
        Ok(AmountShadowAdded { recorded, presence })
    }

    pub(crate) fn clear(&mut self) -> usize {
        self.base.clear()
    }

    pub(crate) fn release(&mut self) -> usize {
        let count = self.base.release();
        self.max_goods_amount = 0;
        count
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp

// ============================================================================
// FUNCTION: CAmountLimitGoodsShadowContainer::CAmountLimitGoodsShadowContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp:16
// RVA: 0x00104280
// ADDRESS: 00504280
// PROTOTYPE: undefined __thiscall CAmountLimitGoodsShadowContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsShadowContainer::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp:70
// RVA: 0x001042A0
// ADDRESS: 005042a0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsShadowContainer::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp:78
// RVA: 0x001042B0
// ADDRESS: 005042b0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsShadowContainer::SetGoodsAmountLimit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp:101
// RVA: 0x001042D0
// ADDRESS: 005042d0
// PROTOTYPE: void __thiscall SetGoodsAmountLimit(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsShadowContainer::~CAmountLimitGoodsShadowContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp:29
// RVA: 0x001042E0
// ADDRESS: 005042e0
// PROTOTYPE: void __thiscall ~CAmountLimitGoodsShadowContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsShadowContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp:36
// RVA: 0x00104340
// ADDRESS: 00504340
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsShadowContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp:53
// RVA: 0x00104390
// ADDRESS: 00504390
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsShadowContainer::IsFull
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp:87
// RVA: 0x001043E0
// ADDRESS: 005043e0
// PROTOTYPE: int __thiscall IsFull(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsShadowContainer::Clone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\camountlimitgoodsshadowcontainer.cpp:117
// RVA: 0x00104540
// ADDRESS: 00504540
// PROTOTYPE: int __thiscall Clone(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
