//! Сессионное расширение DaKong/XiangQian GameServer в Zone `sessions/`.
//! Прежняя форма: `src/gameserver/appserver/session/cequipmentdakong.rs`. Типы
//! подключаются из zone-владельцев: восьмислотовый shadow-контейнер — Zone
//! `items/cequipmentdakongcontainer.rs`, DaKong-семейство фабрики — Zone
//! `content/goodsfactory.rs` (реэкспортируется здесь для потребителей старого
//! пакета), `CGuid` — Shared. Исходный owner
//! `appserver/session/cequipmentdakong.cpp`; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb`.
//!
//! Игровая логика и окончательное закрытие выполняются через канонические
//! `CGame`, игрока, фабрику предметов и общий RNG MSVCRT; сценарный вызов `9351`
//! использует тот же алгоритм внешних свойств. Результаты уже выполненных
//! отправок публикуются через `tracing`, не накапливаясь в отчётах.
//! Quirk сохранён: половинное свойство седьмого слота держит x87-усечение к
//! нулю, включая отрицательные значения снятия камня.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#сессии-игрока

use crate::items::cequipmentdakongcontainer::CEquipmentDaKongContainer;
use crate::regions::ShapeIdentity;
use nebokrai_shared::values::CGuid;

pub use crate::content::goodsfactory::{
    EquipmentDaKongEnchaseEvent, EquipmentDaKongGemSnapshot, EquipmentDaKongGoodsSnapshot,
    deal_enchase_gems, deal_with_da_kong_external_attributes, deal_with_da_kong_seven,
    equipment_da_kong_condition,
};

pub const DA_KONG_USE_SINKER_INDEX: u32 = 0x120f_daa7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentDaKongOperation {
    DaKong { color_index: i32 },
    EnchaseGem { parameter: i32 },
    ChangeRoleColor { socket: i32 },
    QueryResult,
    DestroyGem { socket: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentDaKongAuditLog {
    pub player_id: i32,
    pub reason: u8,
    pub cost_base_index: u32,
    pub cost_price: u32,
    pub cost_name: Vec<u8>,
    pub equipment: EquipmentDaKongGoodsSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentDaKongClientUpdate {
    pub player_id: i32,
    pub goods: ShapeIdentity,
    pub old_client_payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EquipmentDaKongAroundEffect {
    pub effect_id: i32,
    pub region_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentDaKongScriptModifyKind {
    ReapplyGemProperties,
    ClampDeluxProperties,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CEquipmentDaKong {
    upgrade_container: CEquipmentDaKongContainer,
}

impl CEquipmentDaKong {
    pub const fn new() -> Self {
        Self {
            upgrade_container: CEquipmentDaKongContainer::new(),
        }
    }

    pub const fn upgrade_container(&self) -> &CEquipmentDaKongContainer {
        &self.upgrade_container
    }

    pub const fn upgrade_container_mut(&mut self) -> &mut CEquipmentDaKongContainer {
        &mut self.upgrade_container
    }

    pub const fn last_equipment_id(&self) -> CGuid {
        self.upgrade_container.last_goods()
    }
}
