//! Сессионное расширение DaKong/XiangQian исторического GameServer.
//!
//! Точная пара `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/session/cequipmentdakong.cpp`. Расширение во
//! владении Rust хранит восьмислотовую теневую копию и достигается из кодов
//! предметов `0x8FC1E..0x8FC23`; игровая логика и окончательное закрытие
//! выполняются через канонические `CGame`, игрока, фабрику предметов и общий
//! RNG MSVCRT. Жизненный цикл слушателя и сессии связан через фабрику сессий
//! экипировки и `MainLoop`; сценарный вызов `9351` использует тот же алгоритм
//! внешних свойств, обязательную причину `4`, расход, эффект области `11` и
//! обновление предмета. Уведомления, расход пакета, `0xBF918`, `0xBF50A` и
//! World `0x60212` исполняются `CGame`; сценарии объявлений проходят через
//! живой диспетчер `CScript::RunFunction` с временным возвратом игрока в
//! каноническую карту игры в точной позиции вызова. Результаты уже выполненных
//! отправок публикуются через `tracing`, не накапливаясь в отчётах.
//! Половинное свойство седьмого слота сохраняет x87-усечение к нулю, включая
//! отрицательные значения снятия камня.
//!
//! Волна Z-G0b: точное DaKong-семейство исходного `cgoodsfactory.cpp`
//! (`DealEnchaseGem`, ветви `DealWithExternAttr`, условие седьмого слота,
//! снимки камня/предмета и типы событий) возвращено к владельцу в Zone
//! `content/goodsfactory.rs` вместе с фабрикой; здесь остаются сессионные
//! типы и реэкспорт для потребителей старого пакета.

use crate::gameserver::appserver::container::cequipmentdakongcontainer::CEquipmentDaKongContainer;
use crate::gameserver::appserver::shape::ShapeIdentity;

pub(crate) use nebokrai_zone::content::goodsfactory::{
    EquipmentDaKongEnchaseEvent, EquipmentDaKongGemSnapshot, EquipmentDaKongGoodsSnapshot,
    deal_enchase_gems, deal_with_da_kong_external_attributes, deal_with_da_kong_seven,
    equipment_da_kong_condition,
};

pub(crate) const DA_KONG_USE_SINKER_INDEX: u32 = 0x120f_daa7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentDaKongOperation {
    DaKong { color_index: i32 },
    EnchaseGem { parameter: i32 },
    ChangeRoleColor { socket: i32 },
    QueryResult,
    DestroyGem { socket: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentDaKongAuditLog {
    pub(crate) player_id: i32,
    pub(crate) reason: u8,
    pub(crate) cost_base_index: u32,
    pub(crate) cost_price: u32,
    pub(crate) cost_name: Vec<u8>,
    pub(crate) equipment: EquipmentDaKongGoodsSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentDaKongClientUpdate {
    pub(crate) player_id: i32,
    pub(crate) goods: ShapeIdentity,
    pub(crate) old_client_payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentDaKongAroundEffect {
    pub(crate) effect_id: i32,
    pub(crate) region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentDaKongScriptModifyKind {
    ReapplyGemProperties,
    ClampDeluxProperties,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CEquipmentDaKong {
    upgrade_container: CEquipmentDaKongContainer,
}

impl CEquipmentDaKong {
    pub(crate) const fn new() -> Self {
        Self {
            upgrade_container: CEquipmentDaKongContainer::new(),
        }
    }

    pub(crate) const fn upgrade_container(&self) -> &CEquipmentDaKongContainer {
        &self.upgrade_container
    }

    pub(crate) const fn upgrade_container_mut(&mut self) -> &mut CEquipmentDaKongContainer {
        &mut self.upgrade_container
    }

    pub(crate) const fn last_equipment_id(&self) -> nebokrai_shared::values::CGuid {
        self.upgrade_container.last_goods()
    }
}
