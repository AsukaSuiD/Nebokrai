//! Storage-prefix `CFairyContainer` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cfairycontainer.cpp`. Контейнер
//! владеет ordinary-fairy товарами через `CVolumeLimitGoodsContainer`:
//! auto-add сначала выбирает ячейку base-owner-а, positional add сохраняет
//! отдельное правило `0..12` для headgear с fairy property и исключение
//! `13/FZ0885`, а remove блокирует фею с ненулевым hatch timer.
//!
//! `CVolumeLimitGoodsContainer` и owned `CGoods` заменяют vtable dispatch/raw
//! pointers, не меняя lock/stack/listener semantics base-owner-а. State change,
//! hatch/exp traversal, codec-tail и syncretize ниже пока остаются RAW.

use super::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddBlock, VolumeGoodsAddOutcome,
    VolumeGoodsRemoveOutcome,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    EQUIP_PLACE_HEADGEAR, GAP_BF_BATTLE_FAIRY,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::public::guid::CGuid;

const FAIRY_SPECIAL_POSITION: u32 = 13;
const FAIRY_SPECIAL_ORIGINAL_NAME: &[u8] = b"FZ0885";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FairyContainerAddBlock {
    MissingGoods,
    BattleFairy,
    MissingBaseProperties { index: u32 },
    InvalidFairyPosition { position: u32 },
    InvalidFairyGoods { position: u32 },
}

#[must_use = "outcome определяет ownership incoming и base listener effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairyContainerAddOutcome {
    Base(VolumeGoodsAddOutcome),
    Rejected(FairyContainerAddBlock),
}

#[must_use = "remove может быть заблокирован живым hatch timer"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FairyContainerRemoveOutcome {
    Missing,
    Hatching {
        position: Option<u32>,
        goods_id: CGuid,
    },
    Removed(VolumeGoodsRemoveOutcome),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFairyContainer {
    base: CVolumeLimitGoodsContainer,
}

impl Default for CFairyContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CFairyContainer {
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

    pub(crate) fn add(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> FairyContainerAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return FairyContainerAddOutcome::Rejected(FairyContainerAddBlock::MissingGoods);
        };
        let Some(position) = self.base.find_position_for_goods(goods, factory) else {
            return FairyContainerAddOutcome::Base(VolumeGoodsAddOutcome::Rejected(
                VolumeGoodsAddBlock::NoSpace,
            ));
        };
        self.add_at(position, incoming, factory, owner_progress_allows)
    }

    pub(crate) fn add_at(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> FairyContainerAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return FairyContainerAddOutcome::Rejected(FairyContainerAddBlock::MissingGoods);
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 1 {
            return FairyContainerAddOutcome::Rejected(FairyContainerAddBlock::BattleFairy);
        }
        let index = goods.base_properties_index();
        let Some(properties) = factory.query_goods_base_properties(index) else {
            return FairyContainerAddOutcome::Rejected(
                FairyContainerAddBlock::MissingBaseProperties { index },
            );
        };

        let accepted = if position < FAIRY_SPECIAL_POSITION {
            properties.equip_place() == EQUIP_PLACE_HEADGEAR && goods.fairy_properties().is_some()
        } else if position == FAIRY_SPECIAL_POSITION {
            properties.original_name() == FAIRY_SPECIAL_ORIGINAL_NAME
        } else {
            return FairyContainerAddOutcome::Rejected(
                FairyContainerAddBlock::InvalidFairyPosition { position },
            );
        };
        if !accepted {
            return FairyContainerAddOutcome::Rejected(FairyContainerAddBlock::InvalidFairyGoods {
                position,
            });
        }

        FairyContainerAddOutcome::Base(self.base.add_goods_at(
            position,
            incoming,
            factory,
            owner_progress_allows,
        ))
    }

    pub(crate) fn remove(&mut self, goods_id: CGuid) -> FairyContainerRemoveOutcome {
        let position = self.base.query_goods_position(goods_id);
        let Some(goods) = self.base.base().find(goods_id) else {
            return FairyContainerRemoveOutcome::Missing;
        };
        if goods
            .fairy_properties()
            .is_some_and(|fairy| fairy.hatch_start_time != 0)
        {
            return FairyContainerRemoveOutcome::Hatching { position, goods_id };
        }
        self.base
            .remove_goods(goods_id)
            .map(FairyContainerRemoveOutcome::Removed)
            .unwrap_or(FairyContainerRemoveOutcome::Missing)
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp

// ============================================================================
// FUNCTION: CFairyContainer::CFairyContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:22
// RVA: 0x000DB650
// ADDRESS: 004db650
// PROTOTYPE: undefined __thiscall CFairyContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:41
// RVA: 0x000DB680
// ADDRESS: 004db680
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::~CFairyContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:26
// RVA: 0x000DB6A0
// ADDRESS: 004db6a0
// PROTOTYPE: void __thiscall ~CFairyContainer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:88
// RVA: 0x000DB700
// ADDRESS: 004db700
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::FairyChangeState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:443
// RVA: 0x000DB750
// ADDRESS: 004db750
// PROTOTYPE: CGoods * __thiscall FairyChangeState(CGoods * param_1, EFairyState param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:518
// RVA: 0x000DBB60
// ADDRESS: 004dbb60
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:538
// RVA: 0x000DBBC0
// ADDRESS: 004dbbc0
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::CheckHatcher
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:151
// RVA: 0x000DBC50
// ADDRESS: 004dbc50
// PROTOTYPE: void __thiscall CheckHatcher(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:46
// RVA: 0x000DBDE0
// ADDRESS: 004dbde0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::FairyExpUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:106
// RVA: 0x000DBF00
// ADDRESS: 004dbf00
// PROTOTYPE: void __thiscall FairyExpUp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFairyContainer::FairySyncretize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cfairycontainer.cpp:186
// RVA: 0x000DC190
// ADDRESS: 004dc190
// PROTOTYPE: ESyncreticResult __thiscall FairySyncretize(ESyncreticProperty param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
