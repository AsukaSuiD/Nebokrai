//! Equipment-compose plug GameServer, перенесённый в Zone `sessions/`.
//!
//! Тело перенесено буквально из прежнего
//! `src/gameserver/appserver/session/cequipmentcompose.rs` (волна Z-C4);
//! отличия — нормализация `pub(crate)`→`pub` на границе crate и швы переноса
//! (не расхождения): трёхслотовый shadow-контейнер — Zone
//! `items/cequipmentcomposeshadowcontainer.rs`, `CGoods` — Zone
//! `items/cgoods.rs`, GAP-константы — Zone `content/goods.rs`, реестр
//! `CGoodsFactory` — Zone `content/goodsfactory.rs` (волна Z-G0b),
//! `ShapeIdentity` — Zone `regions/identity.rs`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/session/cequipmentcompose.cpp`. Process-owned
//! plug хранит трёхслотовый shadow container; gameplay `Compose` вызывается
//! живым `goodsmessage 0x8FC24` старого пакета через проверенные session/plug
//! identity. Insert/end listener lifecycle связан с packet/equipment и
//! terminal session-stage. Addon transfer, universal `UpgradeEquipment`,
//! validation, ordering и ownership результата принадлежат `CGame`. Notice,
//! source/stone container wire, packet result и gated World audit также
//! исполняются живым `CGame`; announcement script проходит через живой
//! `CScript::RunFunction` dispatcher с player/region context и его runtime
//! side effects. Результаты уже выполненных отправок публикуются через
//! `tracing`, а не возвращаются диагностическим отчётом.

use crate::content::goods::{
    GAP_ANIMA_BIND, GAP_DAKONG_1, GAP_DAKONG_EXTERN_1, GAP_DAKONG_EXTERN_2, GAP_DAKONG_EXTERN_3,
    GAP_EQUIP_ACTIVE, GAP_ITEM_QUALITY, GAP_PARTICULAR_ATTRIBUTE, GAP_WEAPON_LEVEL,
};
use crate::content::goodsfactory::CGoodsFactory;
use crate::items::cequipmentcomposeshadowcontainer::CEquipmentComposeShadowContainer;
use crate::items::cgoods::CGoods;
use crate::regions::ShapeIdentity;

pub const COMPOSE_STONE_GOODS_INDEX: u32 = 0x120f_db24;
pub const COMPOSE_CONSUME_REASON: u8 = 0x7f;
pub const COMPOSE_CREATE_REASON: u8 = 0x7e;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentComposeSourceSnapshot {
    pub identity: ShapeIdentity,
    pub base_index: u32,
    pub weapon_level: i32,
    pub anima_bind: i32,
    pub quality: i32,
    pub price: u32,
    pub amount: u32,
    pub name: Vec<u8>,
    pub transferred_addons: Vec<(i32, u32, i32)>,
}

impl EquipmentComposeSourceSnapshot {
    pub fn capture(goods: &CGoods, factory: &CGoodsFactory) -> Self {
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
pub struct EquipmentComposeAuditLog {
    pub player_id: i32,
    pub reason: u8,
    pub goods: ShapeIdentity,
    pub base_index: u32,
    pub price: u32,
    pub name: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CEquipmentCompose {
    compose_container: CEquipmentComposeShadowContainer,
}

impl CEquipmentCompose {
    pub const fn new() -> Self {
        Self {
            compose_container: CEquipmentComposeShadowContainer::new(),
        }
    }

    pub const fn compose_container(&self) -> &CEquipmentComposeShadowContainer {
        &self.compose_container
    }

    pub const fn compose_container_mut(&mut self) -> &mut CEquipmentComposeShadowContainer {
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
// COMPONENT_VARIANT_END: GameServer
