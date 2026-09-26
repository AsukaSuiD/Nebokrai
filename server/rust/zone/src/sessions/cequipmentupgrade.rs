//! Equipment-upgrade plug GameServer в Zone `sessions/`: пятислотовый shadow и
//! сессионный lifecycle upgrade-сессии. Прежняя форма:
//! `src/gameserver/appserver/session/cequipmentupgrade.rs`. Типы подключаются из
//! zone-владельцев: shadow-контейнер — Zone `items/cequipmentupgradeshadowcontainer.rs`,
//! `CGoods` — Zone `items/cgoods.rs`, GAP-константы — Zone `content/goods.rs`,
//! реестр — Zone `content/goodsfactory.rs`, `UpgradePriceListener` — Zone
//! `items/cupgradepricelistener.rs`, `CGuid` — Shared. Исходный owner
//! `appserver/session/cequipmentupgrade.cpp`; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb`.
//!
//! Живой `goodsmessage 0x8FC0F` старого пакета выполняет validation, оплату,
//! общий MSVCRT RNG, mutation и публикации; `0x8FC10` завершает session и
//! освобождает registry. Gameplay-исполнение не переносится: здесь — типы
//! plug-а и их операции над shadow-состоянием. Наблюдаемые эффекты исполняются
//! живым `CGame` в исходном порядке и публикуются через `tracing`;
//! runtime-границей остаются combat property recompute и around effects
//! снятого equipment.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#сессии-игрока

use crate::content::goods::{
    GAP_GEM_PROBABILITY, GAP_GEM_UPGRADE_FAILED_RESULT, GAP_GEM_UPGRADE_SUCCEED_RESULT,
    GAP_GOODS_UPGRADE_PRICE,
};
use crate::content::goodsfactory::CGoodsFactory;
use crate::items::cequipmentupgradeshadowcontainer::{
    CEquipmentUpgradeShadowContainer, UpgradeEquipmentCell,
};
use crate::items::cgoods::CGoods;
use crate::items::cupgradepricelistener::UpgradePriceListener;
use crate::regions::ShapeIdentity;
use nebokrai_shared::values::CGuid;

pub const EQUIPMENT_UPGRADE_SUCCESS_LOG_REASON: u8 = 1;
pub const EQUIPMENT_UPGRADE_FAILURE_LOG_REASON: u8 = 2;
pub const EQUIPMENT_UPGRADE_LOST_LOG_REASON: u8 = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentUpgradeGoodsSnapshot {
    pub identity: ShapeIdentity,
    pub base_index: u32,
    pub price: u32,
    pub name: Vec<u8>,
}

impl EquipmentUpgradeGoodsSnapshot {
    pub fn capture(goods: &CGoods) -> Self {
        Self {
            identity: goods.identity(),
            base_index: goods.base_properties_index(),
            price: goods.price(),
            name: goods.name().to_vec(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentUpgradeAuditLog {
    pub reason: u8,
    pub player_id: i32,
    pub equipment: EquipmentUpgradeGoodsSnapshot,
    pub gems: [Option<EquipmentUpgradeGoodsSnapshot>; 4],
    pub region_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentUpgradeLostAuditLog {
    pub reason: u8,
    pub player_id: i32,
    pub pk_count: u16,
    pub money: u32,
    pub depot_money: u32,
    pub equipment: EquipmentUpgradeGoodsSnapshot,
    pub amount: u32,
    pub region_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub client_ip: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CEquipmentUpgrade {
    upgrade_container: CEquipmentUpgradeShadowContainer,
    closed: bool,
}

impl Default for CEquipmentUpgrade {
    fn default() -> Self {
        Self::new()
    }
}

impl CEquipmentUpgrade {
    pub const fn new() -> Self {
        Self {
            upgrade_container: CEquipmentUpgradeShadowContainer::new(),
            closed: false,
        }
    }

    pub const fn upgrade_container(&self) -> &CEquipmentUpgradeShadowContainer {
        &self.upgrade_container
    }

    pub const fn upgrade_container_mut(&mut self) -> &mut CEquipmentUpgradeShadowContainer {
        &mut self.upgrade_container
    }

    pub fn close(&mut self) -> usize {
        if self.closed {
            return 0;
        }
        self.closed = true;
        self.upgrade_container.clear()
    }

    pub fn goods_id(&self, cell: UpgradeEquipmentCell) -> Option<CGuid> {
        self.upgrade_container.positions().get(&cell).copied()
    }

    pub fn upgrade_price<Resolve>(&self, mut resolve: Resolve) -> u32
    where
        Resolve: FnMut(CGuid) -> i32,
    {
        let mut listener = UpgradePriceListener::default();
        for goods_id in self.upgrade_container.positions().values() {
            listener.add_property_value(resolve(*goods_id));
        }
        listener.price()
    }

    pub fn probability<Resolve>(&self, mut resolve: Resolve) -> u32
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

    pub fn failed_result<Resolve>(&self, mut resolve: Resolve) -> u32
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

    pub fn succeed_result<Resolve, Random>(&self, mut resolve: Resolve, mut random: Random) -> u32
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

    pub fn price_property(goods: &CGoods, factory: &CGoodsFactory) -> i32 {
        goods.addon_property_value(factory, GAP_GOODS_UPGRADE_PRICE, 1)
    }
}
