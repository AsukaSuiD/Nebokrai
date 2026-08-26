//! Equipment-compose plug GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/session/cequipmentcompose.cpp`. Process-owned
//! plug хранит трёхслотовый shadow container; gameplay `Compose` вызывается
//! живым `goodsmessage 0x8FC24` через проверенные session/plug identity.
//! Insert/end listener lifecycle связан с packet/equipment и terminal
//! session-stage. Addon transfer, universal `UpgradeEquipment`, validation,
//! ordering и ownership результата принадлежат `CGame`. Notice, source/stone
//! container wire, packet result и gated World audit также исполняются живым
//! `CGame`; announcement script проходит через живой `CScript::RunFunction`
//! dispatcher с player/region context и его runtime side effects. Результаты
//! уже выполненных отправок публикуются через `tracing`, а не возвращаются
//! диагностическим отчётом.

use crate::gameserver::appserver::container::cequipmentcomposeshadowcontainer::{
    CEquipmentComposeShadowContainer,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_ANIMA_BIND, GAP_DAKONG_1, GAP_DAKONG_EXTERN_1, GAP_DAKONG_EXTERN_2, GAP_DAKONG_EXTERN_3,
    GAP_EQUIP_ACTIVE, GAP_ITEM_QUALITY, GAP_PARTICULAR_ATTRIBUTE, GAP_WEAPON_LEVEL,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::shape::ShapeIdentity;

pub(crate) const COMPOSE_STONE_GOODS_INDEX: u32 = 0x120f_db24;
pub(crate) const COMPOSE_CONSUME_REASON: u8 = 0x7f;
pub(crate) const COMPOSE_CREATE_REASON: u8 = 0x7e;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentComposeSourceSnapshot {
    pub(crate) identity: ShapeIdentity,
    pub(crate) base_index: u32,
    pub(crate) weapon_level: i32,
    pub(crate) anima_bind: i32,
    pub(crate) quality: i32,
    pub(crate) price: u32,
    pub(crate) amount: u32,
    pub(crate) name: Vec<u8>,
    pub(crate) transferred_addons: Vec<(i32, u32, i32)>,
}

impl EquipmentComposeSourceSnapshot {
    pub(crate) fn capture(goods: &CGoods, factory: &CGoodsFactory) -> Self {
        let mut transferred_addons = Vec::new();
        for property in GAP_DAKONG_1..=GAP_DAKONG_1 + 6 {
            transferred_addons.push((
                property,
                1,
                goods.addon_property_value(factory, property, 1),
            ));
        }
        for property in [
            GAP_DAKONG_EXTERN_1,
            GAP_DAKONG_EXTERN_2,
            GAP_DAKONG_EXTERN_3,
            GAP_ANIMA_BIND,
            GAP_EQUIP_ACTIVE,
        ] {
            for value_id in 1..=2 {
                transferred_addons.push((
                    property,
                    value_id,
                    goods.addon_property_value(factory, property, value_id),
                ));
            }
        }
        for property in [GAP_ITEM_QUALITY, GAP_PARTICULAR_ATTRIBUTE] {
            transferred_addons.push((
                property,
                1,
                goods.addon_property_value(factory, property, 1),
            ));
        }
        Self {
            identity: goods.identity(),
            base_index: goods.base_properties_index(),
            weapon_level: goods.addon_property_value(factory, GAP_WEAPON_LEVEL, 1),
            anima_bind: goods.addon_property_value(factory, GAP_ANIMA_BIND, 1),
            quality: goods.addon_property_value(factory, GAP_ITEM_QUALITY, 1),
            price: goods.price(),
            amount: goods.amount(),
            name: goods.name().to_vec(),
            transferred_addons,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentComposeAuditLog {
    pub(crate) player_id: i32,
    pub(crate) reason: u8,
    pub(crate) goods: ShapeIdentity,
    pub(crate) base_index: u32,
    pub(crate) price: u32,
    pub(crate) name: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CEquipmentCompose {
    compose_container: CEquipmentComposeShadowContainer,
}

impl CEquipmentCompose {
    pub(crate) const fn new() -> Self {
        Self {
            compose_container: CEquipmentComposeShadowContainer::new(),
        }
    }

    pub(crate) const fn compose_container(&self) -> &CEquipmentComposeShadowContainer {
        &self.compose_container
    }

    pub(crate) const fn compose_container_mut(&mut self) -> &mut CEquipmentComposeShadowContainer {
        &mut self.compose_container
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cequipmentcompose.cpp

// VERIFIED: `OnSessionEnded` только проверяет наличие player/region и возвращает
// bool, который `CSession::End` не использует; отдельной mutation/publication нет.
// ============================================================================
// FUNCTION: CEquipmentCompose::GetContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cequipmentcompose.cpp:129
// RVA: 0x0010B300
// ADDRESS: 0050b300
// PROTOTYPE: CContainer * __thiscall GetContainer(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `OnPlugInserted` owner/extend setup и packet/equipment listener attach
// выполняются equipment-session factory/caller-ом; покрытое RAW-тело удалено.
// ============================================================================
// FUNCTION: CEquipmentCompose::CEquipmentCompose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cequipmentcompose.cpp:18
// RVA: 0x001B4C40
// ADDRESS: 005b4c40
// PROTOTYPE: undefined __thiscall CEquipmentCompose(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: terminal session-stage снимает packet/equipment listener перед
// registry GC; покрытое RAW-тело `OnPlugEnded` удалено.
// ============================================================================
// FUNCTION: CEquipmentCompose::~CEquipmentCompose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cequipmentcompose.cpp:22
// RVA: 0x001B4D10
// ADDRESS: 005b4d10
// PROTOTYPE: void __thiscall ~CEquipmentCompose(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentCompose::IsPlugAvailable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cequipmentcompose.cpp:29
// RVA: 0x001B4D80
// ADDRESS: 005b4d80
// PROTOTYPE: int __thiscall IsPlugAvailable(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentCompose::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cequipmentcompose.cpp:561
// RVA: 0x001B4DD0
// ADDRESS: 005b4dd0
// PROTOTYPE: int __thiscall DeleteGoods(COMPOSING_EQUIPMENT_PLACE_CELL param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentCompose::WriteLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cequipmentcompose.cpp:592
// RVA: 0x001B4F20
// ADDRESS: 005b4f20
// PROTOTYPE: int __thiscall WriteLog(CMessage * param_1, CPlayer * param_2, CGoods * param_3, uchar param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEquipmentCompose::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\session\cequipmentcompose.cpp:121
// RVA: 0x001B5E80
// ADDRESS: 005b5e80
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
