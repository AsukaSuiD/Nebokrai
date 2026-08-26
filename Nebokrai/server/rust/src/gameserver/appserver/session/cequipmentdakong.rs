//! Сессионное расширение DaKong/XiangQian исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
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

use crate::gameserver::appserver::container::cequipmentdakongcontainer::{
    CEquipmentDaKongContainer,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_BAOSHI_COLOR, GAP_DAKONG_1, GAP_DAKONG_EXTERN_1, GAP_DAKONG_EXTERN_2, GAP_DAKONG_EXTERN_3,
};
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::shape::ShapeIdentity;

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
pub(crate) struct EquipmentDaKongGoodsSnapshot {
    pub(crate) identity: ShapeIdentity,
    pub(crate) base_index: u32,
    pub(crate) price: u32,
    pub(crate) name: Vec<u8>,
    pub(crate) description: Vec<u8>,
    pub(crate) socket_and_external_values: Vec<(i32, u32, i32)>,
}

impl EquipmentDaKongGoodsSnapshot {
    pub(crate) fn capture(goods: &CGoods, factory: &CGoodsFactory) -> Self {
        let mut values = Vec::new();
        for property in GAP_DAKONG_1..=GAP_DAKONG_1 + 6 {
            for value_id in 1..=2 {
                values.push((
                    property,
                    value_id,
                    goods.addon_property_value(factory, property, value_id),
                ));
            }
        }
        for property in GAP_DAKONG_EXTERN_1..=GAP_DAKONG_EXTERN_1 + 2 {
            for value_id in 1..=2 {
                values.push((
                    property,
                    value_id,
                    goods.addon_property_value(factory, property, value_id),
                ));
            }
        }
        Self {
            identity: goods.identity(),
            base_index: goods.base_properties_index(),
            price: goods.price(),
            name: goods.name().to_vec(),
            description: goods.description().to_vec(),
            socket_and_external_values: values,
        }
    }
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

    pub(crate) const fn last_equipment_id(&self) -> crate::public::guid::CGuid {
        self.upgrade_container.last_goods()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EquipmentDaKongGemSnapshot {
    pub(crate) identity: Option<ShapeIdentity>,
    pub(crate) base_index: u32,
    pub(crate) color: i32,
    pub(crate) condition: i32,
    pub(crate) temporary: bool,
}

impl EquipmentDaKongGemSnapshot {
    pub(crate) fn capture(goods: &CGoods, factory: &CGoodsFactory) -> Self {
        Self {
            identity: Some(goods.identity()),
            base_index: goods.base_properties_index(),
            color: goods.addon_property_value(factory, GAP_BAOSHI_COLOR, 1),
            condition: goods.addon_property_value(factory, GAP_BAOSHI_COLOR, 2),
            temporary: false,
        }
    }

    pub(crate) fn from_catalog(base_index: u32, factory: &CGoodsFactory) -> Option<Self> {
        let properties = factory.query_goods_base_properties(base_index)?;
        let value = |id| {
            properties
                .get_addon_property_values(GAP_BAOSHI_COLOR)
                .iter()
                .find(|value| value.id == id)
                .map_or(0, |value| value.base_value)
        };
        Some(Self {
            identity: None,
            base_index,
            color: value(1),
            condition: value(2),
            temporary: true,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentDaKongEnchaseEvent {
    Notification(&'static str),
    GemApplied {
        gem: EquipmentDaKongGemSnapshot,
        equipment: EquipmentDaKongGoodsSnapshot,
        audit: bool,
    },
    Script(&'static [u8]),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EquipmentDaKongEnchaseEffects {
    pub(crate) events: Vec<EquipmentDaKongEnchaseEvent>,
}

pub(crate) fn equipment_da_kong_condition(
    gem: EquipmentDaKongGemSnapshot,
    equipment: &CGoods,
    factory: &CGoodsFactory,
) -> bool {
    let count = |color| equipment.query_enchanse_color(factory, color);
    let pair = |first, second, third| count(first) == 2 && count(second) == 2 && count(third) == 2;
    match gem.condition {
        1 => count(2) > 2,
        2 => count(3) > 2,
        3 => count(4) > 2,
        4 => count(6) > 2,
        5 => count(7) > 2,
        6 => count(5) > 2,
        7 => (2..=7).all(|color| count(color) == 1),
        8 => pair(2, 3, 4),
        9 => pair(2, 3, 6),
        10 => pair(2, 3, 5),
        11 => pair(2, 3, 7),
        12 => pair(2, 4, 6),
        13 => pair(2, 4, 5),
        14 => pair(2, 4, 7),
        15 => pair(2, 5, 6),
        16 => pair(2, 6, 7),
        17 => pair(2, 5, 7),
        18 => pair(3, 4, 6),
        19 => pair(3, 4, 5),
        20 => pair(3, 4, 7),
        21 => pair(4, 5, 6),
        22 => pair(4, 6, 7),
        23 => pair(5, 6, 7),
        _ => false,
    }
}

fn base_gem_color(factory: &CGoodsFactory, base_index: u32) -> i32 {
    factory
        .query_goods_base_properties(base_index)
        .and_then(|properties| {
            properties
                .get_addon_property_values(GAP_BAOSHI_COLOR)
                .iter()
                .find(|value| value.id == 1)
        })
        .map_or(0, |value| value.base_value)
}

fn first_base_value(
    values: &[crate::gameserver::appserver::goods::cgoodsbaseproperties::GoodsBaseAddonPropertyValue],
) -> i32 {
    values
        .iter()
        .find(|value| value.id == 1)
        .or_else(|| values.first())
        .map_or(0, |value| value.base_value)
}

fn half_slot_seven_addition(value: i32) -> i32 {
    (f64::from(value) * 0.5).round_ties_even() as i32
}

fn same_color_socket(equipment: &CGoods, factory: &CGoodsFactory, slot: i32) -> bool {
    let property = GAP_DAKONG_1 + slot - 1;
    let socket_color = equipment.addon_property_value(factory, property, 1);
    let gem_index = equipment.addon_property_value(factory, property, 2) as u32;
    base_gem_color(factory, gem_index) == socket_color
}

pub(crate) fn deal_with_da_kong_external_attributes(
    equipment: &mut CGoods,
    factory: &CGoodsFactory,
    slot_seven: Option<EquipmentDaKongGemSnapshot>,
    positive_pass: bool,
) {
    for (property, begin, end, minimum) in [
        (GAP_DAKONG_EXTERN_1, 1, 3, 3),
        (GAP_DAKONG_EXTERN_2, 4, 6, 6),
    ] {
        if equipment.da_kong_count(factory) < minimum
            || !(begin..=end).all(|slot| same_color_socket(equipment, factory, slot))
        {
            continue;
        }
        let target = equipment.addon_property_value(factory, property, 1);
        let mut value = equipment.addon_property_value(factory, property, 2);
        if positive_pass {
            value = value.wrapping_neg();
        }
        let _ = equipment.cut_addon_property_value(factory, target, 1, value, 0);
    }
    if equipment.da_kong_count(factory) == 7
        && same_color_socket(equipment, factory, 7)
        && slot_seven.is_some_and(|gem| equipment_da_kong_condition(gem, equipment, factory))
    {
        let target = equipment.addon_property_value(factory, GAP_DAKONG_EXTERN_3, 1);
        let mut value = equipment.addon_property_value(factory, GAP_DAKONG_EXTERN_3, 2);
        if positive_pass {
            value = value.wrapping_neg();
        }
        let _ = equipment.cut_addon_property_value(factory, target, 1, value, 0);
    }
}

/// Накладывает свойства уже записанных в предмет камней без расхода,
/// уведомлений и изменения самих слотов. Это точная ветка
/// `CGoodsFactory::DealEnchaseGem(goods, false)`, которую вызывает
/// `ReCreateAddonProperties` после восстановления диапазона DaKong.
pub(crate) fn apply_embedded_gem_properties<Random>(
    equipment: &mut CGoods,
    factory: &CGoodsFactory,
    mut random: Random,
) where
    Random: FnMut(i32) -> i32,
{
    let mut add_types = std::collections::BTreeSet::new();
    crate::public::dakongxiangqian::CDaKongXiangQian::get_add_type(&mut add_types);

    let count = equipment.da_kong_count(factory);
    for socket in 0..count {
        let socket_property = GAP_DAKONG_1 + socket as i32;
        let socket_color = equipment.addon_property_value(factory, socket_property, 1);
        let gem_index = equipment.addon_property_value(factory, socket_property, 2) as u32;
        let Some(gem) = EquipmentDaKongGemSnapshot::from_catalog(gem_index, factory) else {
            continue;
        };
        let Some(base) = factory.query_goods_base_properties(gem_index) else {
            continue;
        };
        let deluxe_condition_met = socket_color != 7
            || gem.color != 8
            || factory
                .create_goods_core(
                    gem_index,
                    |maximum| random(maximum),
                    || crate::public::guid::CGuid::GUID_INVALID,
                )
                .is_some_and(|created| {
                    equipment_da_kong_condition(
                        EquipmentDaKongGemSnapshot::capture(&created, factory),
                        equipment,
                        factory,
                    )
                });
        for addon in base.addon_properties() {
            if !add_types.contains(&addon.property_type) {
                continue;
            }
            let mut value = first_base_value(&addon.values);
            if socket_color < 7 && gem.color == 8 {
                value = 0;
            } else if socket_color == 7 && gem.color != 8 {
                value /= 2;
            } else if !deluxe_condition_met {
                value = 0;
            }
            let _ = equipment.cut_addon_property_value(
                factory,
                addon.property_type,
                1,
                value.wrapping_neg(),
                gem_index,
            );
        }
    }

    let seventh_index = equipment.addon_property_value(factory, GAP_DAKONG_1 + 6, 2) as u32;
    let seventh = EquipmentDaKongGemSnapshot::from_catalog(seventh_index, factory);
    deal_with_da_kong_external_attributes(equipment, factory, seventh, true);
}

pub(crate) fn deal_with_da_kong_seven(
    equipment: &mut CGoods,
    factory: &CGoodsFactory,
    positive_pass: bool,
) {
    if equipment.da_kong_count(factory) != 7 {
        return;
    }
    let gem_index = equipment.addon_property_value(factory, GAP_DAKONG_1 + 6, 2) as u32;
    let Some(gem) = EquipmentDaKongGemSnapshot::from_catalog(gem_index, factory) else {
        return;
    };
    if !equipment_da_kong_condition(gem, equipment, factory) {
        return;
    }
    let mut add_types = std::collections::BTreeSet::new();
    crate::public::dakongxiangqian::CDaKongXiangQian::get_add_type(&mut add_types);
    let Some(base) = factory.query_goods_base_properties(gem_index) else {
        return;
    };
    for addon in base.addon_properties() {
        if !add_types.contains(&addon.property_type) {
            continue;
        }
        let mut value = first_base_value(&addon.values);
        if positive_pass {
            value = value.wrapping_neg();
        }
        let _ =
            equipment.cut_addon_property_value(factory, addon.property_type, 1, value, gem_index);
    }
}

pub(crate) fn deal_enchase_gems(
    equipment: &mut CGoods,
    gems: &[Option<EquipmentDaKongGemSnapshot>; 7],
    factory: &CGoodsFactory,
    apply: bool,
) -> EquipmentDaKongEnchaseEffects {
    let mut effects = EquipmentDaKongEnchaseEffects::default();
    let old_seven_index = equipment.addon_property_value(factory, GAP_DAKONG_1 + 6, 2) as u32;
    let old_seven =
        gems[6].or_else(|| EquipmentDaKongGemSnapshot::from_catalog(old_seven_index, factory));
    deal_with_da_kong_external_attributes(equipment, factory, old_seven, false);

    let count = equipment.da_kong_count(factory) as usize;
    let mut add_types = std::collections::BTreeSet::new();
    crate::public::dakongxiangqian::CDaKongXiangQian::get_add_type(&mut add_types);
    for slot in (1..=count).rev() {
        let property = GAP_DAKONG_1 + slot as i32 - 1;
        let old_index = equipment.addon_property_value(factory, property, 2) as u32;
        let Some(base) = factory.query_goods_base_properties(old_index) else {
            continue;
        };
        let color = base_gem_color(factory, old_index);
        for addon in base.addon_properties() {
            if !add_types.contains(&addon.property_type) {
                continue;
            }
            let mut value = first_base_value(&addon.values);
            if slot < 7 && color == 8 {
                value = 0;
            } else if slot == 7 && color != 8 {
                value /= 2;
            } else if slot == 7
                && color == 8
                && !old_seven
                    .is_some_and(|gem| equipment_da_kong_condition(gem, equipment, factory))
            {
                value = 0;
            }
            let _ = equipment.cut_addon_property_value(
                factory,
                addon.property_type,
                1,
                value,
                old_index,
            );
        }
    }

    for slot in 1..=count {
        let Some(gem) = gems[slot - 1].or_else(|| {
            (slot == 7)
                .then(|| EquipmentDaKongGemSnapshot::from_catalog(old_seven_index, factory))
                .flatten()
        }) else {
            continue;
        };
        let property = GAP_DAKONG_1 + slot as i32 - 1;
        let _ = equipment.set_addon_property_modifier_core(property, 2, gem.base_index as i32);
        let Some(base) = factory.query_goods_base_properties(gem.base_index) else {
            continue;
        };
        let deluxe_seven =
            slot == 7 && gem.color == 8 && equipment_da_kong_condition(gem, equipment, factory);
        if (2..8).contains(&gem.color) {
            for addon in base.addon_properties() {
                if !add_types.contains(&addon.property_type) {
                    continue;
                }
                let mut value = first_base_value(&addon.values).wrapping_neg();
                if slot == 7 {
                    value = half_slot_seven_addition(value);
                    if apply && !gem.temporary {
                        effects
                            .events
                            .push(EquipmentDaKongEnchaseEvent::Notification("GS1168"));
                    }
                }
                let _ = equipment.cut_addon_property_value(
                    factory,
                    addon.property_type,
                    1,
                    value,
                    gem.base_index,
                );
                if apply && !gem.temporary {
                    effects
                        .events
                        .push(EquipmentDaKongEnchaseEvent::GemApplied {
                            gem,
                            equipment: EquipmentDaKongGoodsSnapshot::capture(equipment, factory),
                            audit: slot < 7,
                        });
                    if slot == 6 {
                        effects.events.push(EquipmentDaKongEnchaseEvent::Script(
                            b"scripts/goods/jewel06_gonggao.script",
                        ));
                    }
                }
            }
        } else if gem.color == 8 {
            if slot != 7 {
                if slot <= 6 && apply && !gem.temporary {
                    effects
                        .events
                        .push(EquipmentDaKongEnchaseEvent::Notification("GS1169"));
                }
            } else if deluxe_seven {
                for addon in base.addon_properties() {
                    if !add_types.contains(&addon.property_type) {
                        continue;
                    }
                    let _ = equipment.cut_addon_property_value(
                        factory,
                        addon.property_type,
                        1,
                        first_base_value(&addon.values).wrapping_neg(),
                        gem.base_index,
                    );
                    if apply && !gem.temporary {
                        effects
                            .events
                            .push(EquipmentDaKongEnchaseEvent::GemApplied {
                                gem,
                                equipment: EquipmentDaKongGoodsSnapshot::capture(
                                    equipment, factory,
                                ),
                                audit: true,
                            });
                    }
                }
                if apply && !gem.temporary {
                    effects.events.push(EquipmentDaKongEnchaseEvent::Script(
                        b"scripts/goods/jewel07_gonggao.script",
                    ));
                }
            }
        }
    }
    let new_seven_index = equipment.addon_property_value(factory, GAP_DAKONG_1 + 6, 2) as u32;
    let new_seven =
        gems[6].or_else(|| EquipmentDaKongGemSnapshot::from_catalog(new_seven_index, factory));
    deal_with_da_kong_external_attributes(equipment, factory, new_seven, true);
    effects
}
