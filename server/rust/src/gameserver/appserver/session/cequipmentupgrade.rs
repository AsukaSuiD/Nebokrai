//! Equipment-upgrade plug GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/session/cequipmentupgrade.cpp`. Plug хранит
//! пятислотовый shadow: equipment, обязательный base gem и до трёх
//! дополнительных gems. Живой `goodsmessage 0x8FC0F` выполняет validation,
//! оплату, общий MSVCRT RNG, ordinary-equipment mutation, расход shadow-goods,
//! player/equipment callbacks, `0xBF918` и World audit. `0x8FC10` завершает
//! session, очищает progress/shadow, отправляет `0xBF913` и освобождает
//! session/plug registry.
//!
//! MSVC listener/vtable plumbing заменён owned listener handle и concrete
//! terminal session state. Наблюдаемые эффекты исполняются живым `CGame` в
//! исходном порядке, а их результаты публикуются через `tracing`, не возвращаясь
//! диагностическим деревом. Runtime-границей остаются combat property recompute
//! и around effects снятого equipment.

use crate::gameserver::appserver::container::cequipmentupgradeshadowcontainer::{
    CEquipmentUpgradeShadowContainer, UpgradeEquipmentCell,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_GEM_PROBABILITY, GAP_GEM_UPGRADE_FAILED_RESULT, GAP_GEM_UPGRADE_SUCCEED_RESULT,
    GAP_GOODS_UPGRADE_PRICE,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::listener::cupgradepricelistener::UpgradePriceListener;
use crate::gameserver::appserver::shape::ShapeIdentity;
use nebokrai_shared::values::CGuid;

pub(crate) const EQUIPMENT_UPGRADE_SUCCESS_LOG_REASON: u8 = 1;
pub(crate) const EQUIPMENT_UPGRADE_FAILURE_LOG_REASON: u8 = 2;
pub(crate) const EQUIPMENT_UPGRADE_LOST_LOG_REASON: u8 = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentUpgradeGoodsSnapshot {
    pub(crate) identity: ShapeIdentity,
    pub(crate) base_index: u32,
    pub(crate) price: u32,
    pub(crate) name: Vec<u8>,
}

impl EquipmentUpgradeGoodsSnapshot {
    pub(crate) fn capture(goods: &CGoods) -> Self {
        Self {
            identity: goods.identity(),
            base_index: goods.base_properties_index(),
            price: goods.price(),
            name: goods.name().to_vec(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentUpgradeAuditLog {
    pub(crate) reason: u8,
    pub(crate) player_id: i32,
    pub(crate) equipment: EquipmentUpgradeGoodsSnapshot,
    pub(crate) gems: [Option<EquipmentUpgradeGoodsSnapshot>; 4],
    pub(crate) region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentUpgradeLostAuditLog {
    pub(crate) reason: u8,
    pub(crate) player_id: i32,
    pub(crate) pk_count: u16,
    pub(crate) money: u32,
    pub(crate) depot_money: u32,
    pub(crate) equipment: EquipmentUpgradeGoodsSnapshot,
    pub(crate) amount: u32,
    pub(crate) region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) client_ip: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CEquipmentUpgrade {
    upgrade_container: CEquipmentUpgradeShadowContainer,
    closed: bool,
}

impl Default for CEquipmentUpgrade {
    fn default() -> Self {
        Self::new()
    }
}

impl CEquipmentUpgrade {
    pub(crate) const fn new() -> Self {
        Self {
            upgrade_container: CEquipmentUpgradeShadowContainer::new(),
            closed: false,
        }
    }

    pub(crate) const fn upgrade_container(&self) -> &CEquipmentUpgradeShadowContainer {
        &self.upgrade_container
    }

    pub(crate) const fn upgrade_container_mut(&mut self) -> &mut CEquipmentUpgradeShadowContainer {
        &mut self.upgrade_container
    }

    pub(crate) fn close(&mut self) -> usize {
        if self.closed {
            return 0;
        }
        self.closed = true;
        self.upgrade_container.clear()
    }

    pub(crate) fn goods_id(&self, cell: UpgradeEquipmentCell) -> Option<CGuid> {
        self.upgrade_container.positions().get(&cell).copied()
    }

    pub(crate) fn upgrade_price<Resolve>(&self, mut resolve: Resolve) -> u32
    where
        Resolve: FnMut(CGuid) -> i32,
    {
        let mut listener = UpgradePriceListener::default();
        for goods_id in self.upgrade_container.positions().values() {
            listener.add_property_value(resolve(*goods_id));
        }
        listener.price()
    }

    pub(crate) fn probability<Resolve>(&self, mut resolve: Resolve) -> u32
    where
        Resolve: FnMut(CGuid, i32, u32) -> i32,
    {
        let total = [
            UpgradeEquipmentCell::BaseGem,
            UpgradeEquipmentCell::GemOne,
            UpgradeEquipmentCell::GemTwo,
            UpgradeEquipmentCell::GemThree,
            UpgradeEquipmentCell::Equipment,
        ]
        .into_iter()
        .filter_map(|cell| self.goods_id(cell))
        .fold(0i32, |total, goods_id| {
            total.wrapping_add(resolve(goods_id, GAP_GEM_PROBABILITY, 1))
        });
        total.clamp(0, 100) as u32
    }

    pub(crate) fn failed_result<Resolve>(&self, mut resolve: Resolve) -> u32
    where
        Resolve: FnMut(CGuid, i32, u32) -> i32,
    {
        [
            UpgradeEquipmentCell::BaseGem,
            UpgradeEquipmentCell::GemOne,
            UpgradeEquipmentCell::GemTwo,
            UpgradeEquipmentCell::GemThree,
        ]
        .into_iter()
        .filter_map(|cell| self.goods_id(cell))
        .map(|goods_id| resolve(goods_id, GAP_GEM_UPGRADE_FAILED_RESULT, 1) as u32)
        .filter(|&candidate| candidate != 0)
        .fold(4, u32::min)
    }

    pub(crate) fn succeed_result<Resolve, Random>(
        &self,
        mut resolve: Resolve,
        mut random: Random,
    ) -> u32
    where
        Resolve: FnMut(CGuid, i32, u32) -> i32,
        Random: FnMut(i32) -> i32,
    {
        let mut result = self
            .goods_id(UpgradeEquipmentCell::BaseGem)
            .map_or(0, |goods_id| {
                resolve(goods_id, GAP_GEM_UPGRADE_SUCCEED_RESULT, 1) as u32
            });
        for cell in [
            UpgradeEquipmentCell::GemOne,
            UpgradeEquipmentCell::GemTwo,
            UpgradeEquipmentCell::GemThree,
        ] {
            let Some(goods_id) = self.goods_id(cell) else {
                continue;
            };
            let candidate = resolve(goods_id, GAP_GEM_UPGRADE_SUCCEED_RESULT, 1) as u32;
            if result < candidate
                && random(100) <= resolve(goods_id, GAP_GEM_UPGRADE_SUCCEED_RESULT, 2)
            {
                result = candidate;
            }
        }
        result
    }

    pub(crate) fn price_property(goods: &CGoods, factory: &CGoodsFactory) -> i32 {
        goods.addon_property_value(factory, GAP_GOODS_UPGRADE_PRICE, 1)
    }
}
