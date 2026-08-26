//! Реестр запуска и поисковая часть `CGoodsFactory` GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/goods/cgoodsfactory.cpp`. Селектор `0x00`
//! сначала освобождает три прежние карты, затем читает счётчик `u32` и
//! упорядоченные записи `goods_id + CGoodsBaseProperties`; ID и оба индекса
//! строковых байтов используют последнюю запись. Отсутствующий результат и
//! нулевой указатель имени возвращают ноль/`None`.
//!
//! Парный сериализатор WorldServer подтверждает формат обмена. `BTreeMap` и
//! значения во владении Rust заменяют дерево MSVC и сырые указатели без
//! изменения порядка. Одиночное создание предмета замкнуто вместе с
//! обязательной загрузкой свойств обычных предметов и боевых фей. Пошаговый
//! `UpgradeBFEquipment` меняет уровень экземпляра и восемь зависящих от роста
//! дополнений в исходном порядке каждого шага уровня. Пакетная перегрузка
//! CiQing сохраняет дробление расходуемых и бесполезных предметов по пределу
//! стопки и поштучное создание остальных типов; обычное улучшение ниже
//! сохраняется как RAW. Магазин NPC замыкает формулы ремонта и продажи с
//! коэффициентами настройки; целочисленное отношение долговечности при продаже
//! сохранено как наблюдаемая семантика x86.
//! `ReCreateBattleFairyAttributes` сохраняет странный повтор полного набора
//! бросков RNG по числу дополнений экземпляра; однопроходная оптимизация донора
//! не переносится, потому что меняла итоговое состояние игрового RNG.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::cgoods::{CGoods, GoodsAddonProperty, GoodsAddonPropertyValue};
use super::super::legacycodec::LegacyReader;
use super::cgoodsbaseproperties::{
    CGoodsBaseProperties, GAP_ARMOR_CORRECTION, GAP_ARMOR_UPGRADE, GAP_ATTACK_SPEED_CORRECTION,
    GAP_ATTACK_SPEED_UPGRADE, GAP_BF_ABRAVE_ADDON, GAP_BF_ABRAVE_GROW, GAP_BF_AGILITY_ADDON,
    GAP_BF_AGILITY_BASE, GAP_BF_AGILITY_GROW, GAP_BF_ATTACK_ADDON, GAP_BF_ATTACK_BASE,
    GAP_BF_ATTACK_GROW, GAP_BF_BATTLE_FAIRY, GAP_BF_BRAVE_BASE, GAP_BF_LIFE_ADDON,
    GAP_BF_LIFE_GROW, GAP_BF_MP_ADDON, GAP_BF_MP_GROW, GAP_BF_PULLULATERATE, GAP_BF_SPRITE_ADDON,
    GAP_BF_SPRITE_BASE, GAP_BF_SPRITE_GROW, GAP_BF_SPRITUALISE_ADDON, GAP_BF_SPRITUALISE_GROW,
    GAP_BF_SPRITUALISM_BASE, GAP_BF_STRENGH_ADDON, GAP_BF_STRENGH_BASE, GAP_BF_STRENGH_GROW,
    GAP_BF_WEAPON_LEVEL, GAP_BURDEN_UPPER_LIMIT_CORRECTION,
    GAP_BURDEN_UPPER_LIMIT_CORRECTION_UPGRADE, GAP_DAKONG_1, GAP_DAKONG_EXTERN_1,
    GAP_DAKONG_EXTERN_2, GAP_DAKONG_EXTERN_3, GAP_DODGE_CORRECTION, GAP_DODGE_UPGRADE,
    GAP_ELEMENT_ATTACK_CORRECTION, GAP_ELEMENT_ATTACK_UPGRADE, GAP_ELEMENT_RESISTANCE_CORRECTION,
    GAP_ELEMENT_RESISTANCE_CORRECTION_UPGRADE, GAP_FATAL_BLOW_RATE_CORRECTION,
    GAP_FATAL_BLOW_RATE_UPGRADE, GAP_GOODS_MAXIMUM_DURABILITY,
    GAP_GOODS_MAXIMUM_DURABILITY_UPGRADE, GAP_GOODS_STACKING_LIMIT, GAP_HIT_RATE_CORRECTION,
    GAP_HIT_RATE_UPGRADE, GAP_HP_UPPER_LIMIT_CORRECTION, GAP_HP_UPPER_LIMIT_CORRECTION_UPGRADE,
    GAP_ITEM_QUALITY, GAP_MAXIMUM_ATTACK_CORRECTION, GAP_MAXIMUM_ATTACK_UPGRADE,
    GAP_MINIMUM_ATTACK_CORRECTION, GAP_MINIMUM_ATTACK_UPGRADE, GAP_MP_UPPER_LIMIT_CORRECTION,
    GAP_MP_UPPER_LIMIT_CORRECTION_UPGRADE, GAP_ROLE_MINIMUM_AGILITY_LIMIT,
    GAP_ROLE_MINIMUM_AGILITY_LIMIT_UPGRADE, GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT,
    GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT_UPGRADE, GAP_ROLE_MINIMUM_LEVEL_LIMIT,
    GAP_ROLE_MINIMUM_LEVEL_LIMIT_UPGRADE, GAP_ROLE_MINIMUM_STRENGTH_LIMIT,
    GAP_ROLE_MINIMUM_STRENGTH_LIMIT_UPGRADE, GAP_ROLE_MINIMUM_WAKAN_LIMIT,
    GAP_ROLE_MINIMUM_WAKAN_LIMIT_UPGRADE, GAP_SKILL_REUSE_TIME_CORRECTION,
    GAP_SKILL_REUSE_TIME_CORRECTION_UPGRADE, GAP_STIFFEN_PROBABILITY_CORRECTION,
    GAP_STIFFEN_PROBABILITY_CORRECTION_UPGRADE, GAP_WEAPON_DAMAGE_LEVEL, GAP_WEAPON_DAMAGE_UPGRADE,
    GAP_WEAPON_LEVEL, GOODS_TYPE_CONSUMABLE, GOODS_TYPE_EQUIPMENT, GOODS_TYPE_USELESS,
    GoodsBasePropertiesDecodeError, ICON_TYPE_GROUND,
};
use crate::gameserver::appserver::session::cequipmentdakong::{
    EquipmentDaKongGemSnapshot, apply_embedded_gem_properties, deal_enchase_gems,
    equipment_da_kong_condition,
};
use crate::public::dakongxiangqian::CDaKongXiangQian;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsFactoryDecodeReport {
    pub(crate) declared_records: usize,
    pub(crate) unique_goods: usize,
    pub(crate) unique_original_names: usize,
    pub(crate) unique_names: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsFactoryDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        available: usize,
    },
    BaseProperties(GoodsBasePropertiesDecodeError),
}

impl fmt::Display for GoodsFactoryDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                available,
            } => write!(
                formatter,
                "goods registry обрывается на {field} в {offset}: нужно 4, доступно {available}"
            ),
            Self::BaseProperties(error) => error.fmt(formatter),
        }
    }
}

impl Error for GoodsFactoryDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnexpectedEnd { .. } => None,
            Self::BaseProperties(error) => Some(error),
        }
    }
}

fn recreate_battle_fairy_property<Random>(
    goods: &mut CGoods,
    base: &CGoodsBaseProperties,
    property: i32,
    requested_minimum: i32,
    requested_maximum: i32,
    scaled_modifier_range: bool,
    random: &mut Random,
) where
    Random: FnMut(i32) -> i32,
{
    if !goods.query_attribute(property) {
        return;
    }
    let Some(value) = base.get_addon_property_values(property).first() else {
        return;
    };
    let (minimum, maximum, scale) = if scaled_modifier_range {
        let Some(modifier) = value.modifiers.first() else {
            return;
        };
        (
            (f64::from(modifier.lower_limit) * 0.0001_f64) as i32,
            (f64::from(modifier.upper_limit) * 0.0001_f64) as i32,
            10_000_i64,
        )
    } else {
        (requested_minimum, requested_maximum, 1_i64)
    };
    let width = i64::from(maximum) - i64::from(minimum) + 1;
    if width <= 0 || width > i64::from(i32::MAX) {
        return;
    }
    let rolled = minimum.wrapping_add(random(width as i32));
    let stored =
        i64::from(value.base_value).wrapping_add(i64::from(rolled).wrapping_mul(scale)) as i32;
    let _ = goods.set_addon_property_value_core(property, 1, 0);
    let _ = goods.set_addon_property_value_core(property, 1, stored);
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGoodsFactory {
    goods: BTreeMap<u32, CGoodsBaseProperties>,
    original_name_index: BTreeMap<Vec<u8>, u32>,
    name_index: BTreeMap<Vec<u8>, u32>,
}

impl CGoodsFactory {
    pub(crate) fn calculate_repair_price(&self, goods: &CGoods, repair_factor: f32) -> u32 {
        if !goods.can_repair(self) {
            return 0;
        }
        let current = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 2);
        let maximum = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 1);
        let quality = goods.addon_property_value(self, GAP_ITEM_QUALITY, 1);
        let factor = if 0 < quality {
            (quality.wrapping_mul(50).wrapping_add(150)) as f32 * 0.01 * repair_factor
        } else {
            repair_factor
        };
        let damage = ((maximum.wrapping_sub(current)) as f32 / maximum as f32).max(0.0);
        (goods.price() as f32 * damage * factor).round_ties_even() as u32
    }

    pub(crate) fn calculate_vend_price(
        &self,
        goods: &CGoods,
        base_price_rate: f32,
        trade_in_rate: f32,
    ) -> u32 {
        if goods.price() == 0 {
            return 0;
        }
        let mut price = goods.price() as f32;
        if self
            .query_goods_base_properties(goods.base_properties_index())
            .is_some_and(|properties| properties.goods_type() == GOODS_TYPE_EQUIPMENT)
        {
            let current = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 2);
            let maximum = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 1);
            if maximum != 0 {
                // Primary x86 делит integer durability до преобразования во float.
                let durability = current.min(maximum) / maximum;
                price = goods.price() as f32 * base_price_rate
                    + (1.0 - base_price_rate) * durability as f32 * goods.price() as f32;
            }
        }
        (trade_in_rate * price).round_ties_even() as u32
    }

    pub(crate) fn repair_equipment(&self, goods: &mut CGoods) -> bool {
        goods.repair_durability(self)
    }

    /// Exact `ReCreateBattleFairyAttributes`: исходный owner повторяет весь
    /// набор бросков для каждого instance-addon-а и тем самым оставляет в
    /// товаре последний RNG-pass. Это намеренно не свёрнуто в один бросок.
    pub(crate) fn recreate_battle_fairy_attributes<Random>(
        &self,
        goods: &mut CGoods,
        mode: i32,
        minimum: i32,
        maximum: i32,
        mut random: Random,
    ) -> bool
    where
        Random: FnMut(i32) -> i32,
    {
        if goods.addon_property_value(self, GAP_BF_BATTLE_FAIRY, 1) != 1 || !matches!(mode, 0 | 1) {
            return false;
        }
        let width = i64::from(maximum) - i64::from(minimum) + 1;
        if width <= 0 || width > i64::from(i32::MAX) {
            return false;
        }
        let Some(base) = self
            .query_goods_base_properties(goods.base_properties_index())
            .cloned()
        else {
            return false;
        };
        let pass_count = goods.addon_properties().len();
        for _ in 0..pass_count {
            recreate_battle_fairy_property(
                goods,
                &base,
                GAP_BF_PULLULATERATE,
                minimum,
                maximum,
                false,
                &mut random,
            );
            if mode == 0 {
                for property in [
                    GAP_BF_BRAVE_BASE,
                    GAP_BF_AGILITY_BASE,
                    GAP_BF_SPRITUALISM_BASE,
                    GAP_BF_STRENGH_BASE,
                    GAP_BF_SPRITE_BASE,
                    GAP_BF_ATTACK_BASE,
                ] {
                    recreate_battle_fairy_property(goods, &base, property, 0, 0, true, &mut random);
                }
            }
        }
        true
    }
    pub(crate) fn query_goods_max_stack_number(&self, goods_index: u32) -> u32 {
        let Some(properties) = self.query_goods_base_properties(goods_index) else {
            return 1;
        };
        if !matches!(
            properties.goods_type(),
            GOODS_TYPE_CONSUMABLE | GOODS_TYPE_USELESS
        ) {
            return 1;
        }
        properties
            .get_addon_property_values(GAP_GOODS_STACKING_LIMIT)
            .iter()
            .find(|value| value.id == 1)
            .map(|value| value.base_value)
            .filter(|value| *value > 0)
            .map_or(1, |value| value as u32)
    }

    /// Exact ordinary `UpgradeEquipment`: каждый level step обходит instance
    /// addon-ы в insertion order. Upgrade-range выбирает increment общим RNG,
    /// correction modifier clamp-ится к `0..=65535`, а отсутствие level value
    /// завершает уже применённый prefix отказом.
    pub(crate) fn upgrade_equipment<Random>(
        &self,
        goods: &mut CGoods,
        target_level: i32,
        mut random: Random,
    ) -> bool
    where
        Random: FnMut(i32) -> i32,
    {
        if !goods.can_upgraded(self) {
            return false;
        }
        let initial_level = goods.addon_property_value(self, GAP_WEAPON_LEVEL, 1);
        if initial_level < 0 {
            return false;
        }
        if initial_level == target_level {
            return true;
        }
        let increase = initial_level <= target_level;
        loop {
            let property_types = goods
                .addon_properties()
                .iter()
                .map(|property| property.property_type)
                .collect::<Vec<_>>();
            let mut level_stepped = false;
            for property_type in property_types {
                if property_type == GAP_WEAPON_LEVEL {
                    let Some(value) = goods
                        .addon_properties_mut()
                        .iter_mut()
                        .find(|property| property.property_type == GAP_WEAPON_LEVEL)
                        .and_then(|property| {
                            property.values.iter_mut().find(|value| value.id == 1)
                        })
                    else {
                        continue;
                    };
                    value.modifier = value.modifier.wrapping_add(if increase { 1 } else { -1 });
                    level_stepped = true;
                    continue;
                }
                let Some(target_property) = ordinary_upgrade_pair(property_type) else {
                    continue;
                };
                let minimum = goods.addon_property_value(self, property_type, 1);
                let maximum = goods.addon_property_value(self, property_type, 2);
                if minimum < 1 {
                    continue;
                }
                let delta = if maximum > 0 {
                    minimum.wrapping_add(random(maximum.wrapping_sub(minimum)))
                } else {
                    minimum
                };
                let Some(value) = goods
                    .addon_properties_mut()
                    .iter_mut()
                    .find(|property| property.property_type == target_property)
                    .and_then(|property| property.values.first_mut())
                else {
                    continue;
                };
                value.modifier = if increase {
                    value.modifier.wrapping_add(delta).min(0xffff)
                } else {
                    value.modifier.wrapping_sub(delta).max(0)
                };
            }
            if !level_stepped {
                return false;
            }
            if goods.addon_property_value(self, GAP_WEAPON_LEVEL, 1) == target_level {
                return true;
            }
        }
    }

    /// `ReCreateAddonProperties` повторно создаёт случайные дополнения,
    /// сохраняет уровень, долговечность и непрерывный диапазон DaKong, а затем
    /// заново накладывает свойства вставленных камней и внешних сочетаний.
    pub(crate) fn recreate_addon_properties<Random>(&self, goods: &mut CGoods, mut random: Random)
    where
        Random: FnMut(i32) -> i32,
    {
        let current_level = goods.addon_property_value(self, GAP_WEAPON_LEVEL, 1);
        let maximum_durability = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 1);
        let current_durability = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 2);
        let sockets: Vec<_> = goods
            .addon_properties()
            .iter()
            .filter(|property| {
                (GAP_DAKONG_1..=GAP_DAKONG_EXTERN_3).contains(&property.property_type)
            })
            .cloned()
            .collect();
        let base_index = goods.base_properties_index();
        let Some(recreated) = self.create_goods_core(
            base_index,
            |maximum| random(maximum),
            || CGuid::GUID_INVALID,
        ) else {
            return;
        };
        goods.addon_properties_mut().clear();
        goods
            .addon_properties_mut()
            .extend_from_slice(recreated.addon_properties());
        if current_level > 0 {
            let _ = self.upgrade_equipment(goods, current_level, |maximum| random(maximum));
        }
        let _ = goods.set_addon_property_value_first_core(
            GAP_GOODS_MAXIMUM_DURABILITY,
            1,
            maximum_durability,
        );
        let _ = goods.set_current_durability(current_durability);

        let expected = (GAP_DAKONG_EXTERN_3 - GAP_DAKONG_1 + 1) as usize;
        if sockets.len() == expected {
            for property in goods.addon_properties_mut() {
                if !(GAP_DAKONG_1..=GAP_DAKONG_EXTERN_3).contains(&property.property_type) {
                    continue;
                }
                if let Some(saved) = sockets.get((property.property_type - GAP_DAKONG_1) as usize) {
                    property.clone_from(saved);
                }
            }
        }
        if goods
            .addon_properties()
            .iter()
            .any(|property| property.property_type == GAP_DAKONG_1)
        {
            apply_embedded_gem_properties(goods, self, |maximum| random(maximum));
        }
    }

    /// Точная мутация `CGoodsFactory::DaKongModify`: снимает рассчитанные по
    /// вставленным камням свойства и сразу накладывает их заново по текущему
    /// реестру, не меняя сами семь слотов.
    pub(crate) fn da_kong_modify(&self, goods: &mut CGoods) {
        if !goods.query_attribute(GAP_DAKONG_1) {
            return;
        }
        let gems = std::array::from_fn(|index| {
            let gem_index = goods.addon_property_value(self, GAP_DAKONG_1 + index as i32, 2) as u32;
            EquipmentDaKongGemSnapshot::from_catalog(gem_index, self)
        });
        let _ = deal_enchase_gems(goods, &gems, self, false);
    }

    /// Точный проход `DaKongXiangQian(goods, true)`: сначала выбирает три
    /// внешних свойства по загруженной конфигурации, затем накладывает свойства
    /// уже записанных камней и рассчитанные внешние прибавки.
    pub(crate) fn da_kong_xiang_qian<Random>(
        &self,
        goods: &mut CGoods,
        setup: &CDaKongXiangQian,
        mut random: Random,
    ) where
        Random: FnMut(i32) -> i32,
    {
        let goods_index = goods.base_properties_index();
        let socket_count = goods.da_kong_count(self);
        for (group, minimum, property) in [
            (0, 3, GAP_DAKONG_EXTERN_1),
            (1, 6, GAP_DAKONG_EXTERN_2),
            (2, 7, GAP_DAKONG_EXTERN_3),
        ] {
            let allowed = setup.check_external_property(goods_index, group + 1)
                && if group == 2 {
                    socket_count == minimum
                } else {
                    socket_count >= minimum
                };
            let (property_type, value) = if allowed {
                setup
                    .make_sure_external_attribute(group, goods_index, &mut random)
                    .unwrap_or_default()
            } else {
                (0, 0)
            };
            let _ = goods.set_addon_property_value_core(property, 1, property_type);
            let _ = goods.set_addon_property_modifier_core(property, 2, value);
        }
        apply_embedded_gem_properties(goods, self, random);
    }

    /// Ограничивает особые свойства пределом из базы предмета, вкладом
    /// вставленных камней и зависящей от уровня прибавкой, как
    /// `CGoodsFactory::DaKongDeluxModify`.
    pub(crate) fn da_kong_delux_modify(&self, goods: &mut CGoods, setup: &CDaKongXiangQian) {
        let Some(base) = self
            .query_goods_base_properties(goods.base_properties_index())
            .cloned()
        else {
            return;
        };
        let property_types = goods
            .addon_properties()
            .iter()
            .map(|property| property.property_type)
            .collect::<Vec<_>>();
        for property_type in property_types {
            for modifier in setup
                .delux_modify()
                .iter()
                .filter(|modifier| modifier.property_type == property_type)
            {
                let current = goods.addon_property_value(self, property_type, 1);
                let stone_value = self.da_kong_property_value(goods, property_type);
                let level_value =
                    if modifier.add_type != -1 && base.has_addon_property(modifier.add_type) {
                        goods
                            .addon_property_value(self, modifier.add_type, 1)
                            .wrapping_mul(goods.addon_property_value(self, GAP_WEAPON_LEVEL, 1))
                    } else {
                        0
                    };
                let maximum = base
                    .query_addon_max_property_value(property_type, 1)
                    .wrapping_add(stone_value)
                    .wrapping_add(level_value);
                if maximum < current {
                    let _ = goods.set_addon_property_value_first_core(property_type, 1, maximum);
                }
            }
        }
    }

    fn da_kong_property_value(&self, goods: &CGoods, property_type: i32) -> i32 {
        let mut value = 0i32;
        let count = goods.da_kong_count(self) as usize;
        for index in 0..count {
            let socket = index + 1;
            let gem_index = goods.addon_property_value(self, GAP_DAKONG_1 + index as i32, 2) as u32;
            let Some(gem) = EquipmentDaKongGemSnapshot::from_catalog(gem_index, self) else {
                continue;
            };
            let Some(base) = self.query_goods_base_properties(gem_index) else {
                continue;
            };
            for addon in base
                .addon_properties()
                .iter()
                .filter(|addon| addon.property_type == property_type)
            {
                let mut addition = addon.values.first().map_or(0, |entry| entry.base_value);
                if socket < 7 && gem.color == 8 {
                    addition = 0;
                } else if socket == 7 && gem.color != 8 {
                    addition /= 2;
                } else if socket == 7
                    && gem.color == 8
                    && !equipment_da_kong_condition(gem, goods, self)
                {
                    addition = 0;
                }
                value = value.wrapping_add(addition);
            }
        }

        for (target, begin, end, minimum) in [
            (GAP_DAKONG_EXTERN_1, 1, 3, 3),
            (GAP_DAKONG_EXTERN_2, 4, 6, 6),
        ] {
            if count >= minimum
                && (begin..=end).all(|socket| {
                    let property = GAP_DAKONG_1 + socket as i32 - 1;
                    let socket_color = goods.addon_property_value(self, property, 1);
                    let gem_index = goods.addon_property_value(self, property, 2) as u32;
                    self.query_goods_base_properties(gem_index)
                        .and_then(|base| {
                            base.get_addon_property_values(
                                super::cgoodsbaseproperties::GAP_BAOSHI_COLOR,
                            )
                            .first()
                        })
                        .is_some_and(|gem_color| gem_color.base_value == socket_color)
                })
                && goods.addon_property_value(self, target, 1) == property_type
            {
                value = value.wrapping_add(goods.addon_property_value(self, target, 2));
            }
        }
        if count == 7 && goods.addon_property_value(self, GAP_DAKONG_EXTERN_3, 1) == property_type {
            let gem_index = goods.addon_property_value(self, GAP_DAKONG_1 + 6, 2) as u32;
            if EquipmentDaKongGemSnapshot::from_catalog(gem_index, self)
                .is_some_and(|gem| equipment_da_kong_condition(gem, goods, self))
            {
                value =
                    value.wrapping_add(goods.addon_property_value(self, GAP_DAKONG_EXTERN_3, 2));
            }
        }
        value
    }

    /// Переходит к целевому уровню по одному шагу. На каждом шаге зависящие от
    /// роста дополнения применяются в порядке хранения экземпляра, а уровень
    /// меняется через модификатор значения с ID `1`; отсутствие такого значения
    /// завершает уже применённую начальную часть прохода.
    pub(crate) fn upgrade_battle_fairy_equipment(
        &self,
        goods: &mut CGoods,
        target_level: i32,
    ) -> bool {
        if !goods.can_battle_fairy_equipment_upgrade(self) {
            return false;
        }
        let initial_level = goods.addon_property_value(self, GAP_BF_WEAPON_LEVEL, 1);
        if initial_level < 0 {
            return false;
        }
        if initial_level == target_level {
            return true;
        }
        let step = if initial_level <= target_level { 1 } else { -1 };
        loop {
            let property_types = goods
                .addon_properties()
                .iter()
                .map(|property| property.property_type)
                .collect::<Vec<_>>();
            let mut level_stepped = false;
            for (property_index, property_type) in property_types.into_iter().enumerate() {
                if property_type == GAP_BF_WEAPON_LEVEL {
                    if let Some(value) = goods.addon_properties_mut()[property_index]
                        .values
                        .iter_mut()
                        .find(|value| value.id == 1)
                    {
                        value.modifier = value.modifier.wrapping_add(step);
                        level_stepped = true;
                    }
                    continue;
                }
                let Some((addon, grow)) = battle_fairy_growth_pair(property_type) else {
                    continue;
                };
                let next = goods
                    .addon_property_value(self, addon, 1)
                    .wrapping_add(goods.addon_property_value(self, grow, 1).wrapping_mul(step));
                let _stored = goods.set_addon_property_value_core(addon, 1, next);
            }
            if !level_stepped {
                return false;
            }
            if goods.addon_property_value(self, GAP_BF_WEAPON_LEVEL, 1) == target_level {
                return true;
            }
        }
    }

    /// Достигнутый object/addon prefix `CreateGoods` RVA `0x000682E0`.
    /// Fairy/BattleFairy loaders остаются отдельной незамкнутой suffix-веткой.
    pub(crate) fn create_goods_core<Random, Guid>(
        &self,
        goods_index: u32,
        mut random: Random,
        mut create_guid: Guid,
    ) -> Option<CGoods>
    where
        Random: FnMut(i32) -> i32,
        Guid: FnMut() -> CGuid,
    {
        let properties = self.query_goods_base_properties(goods_index)?;
        let mut goods = CGoods::with_reached_constructor_defaults();
        goods.set_base_properties_index(goods_index);
        goods.set_name(properties.name());
        goods.set_description(properties.description());
        goods.set_price(properties.price());
        goods.set_graphics_id(properties.get_icon_id(ICON_TYPE_GROUND) as i32);
        goods.set_amount(1);

        for property in properties
            .addon_properties()
            .iter()
            .filter(|property| property.is_enabled == 1)
        {
            let roll = random(10_000) as u32;
            if roll >= property.occur_probability {
                continue;
            }
            let mut values = Vec::with_capacity(property.values.len());
            for value in &property.values {
                let mut modifier = 0;
                if value.is_modifier_enabled != 0 {
                    let modifier_roll = random(10_000) as u32;
                    let mut cumulative = 0u32;
                    if let Some(selected) = value.modifiers.iter().find(|candidate| {
                        let selected =
                            modifier_roll.wrapping_sub(cumulative) < candidate.probability;
                        cumulative = cumulative.wrapping_add(candidate.probability);
                        selected
                    }) {
                        let width = selected.upper_limit.wrapping_sub(selected.lower_limit);
                        modifier = random(width).wrapping_add(selected.lower_limit);
                    }
                }
                values.push(GoodsAddonPropertyValue {
                    id: value.id,
                    base_value: value.base_value,
                    modifier,
                });
            }
            goods.push_addon_property(GoodsAddonProperty {
                property_type: property.property_type,
                is_enabled: 1,
                is_implicit_attribute: property.is_implicit_attribute,
                values,
            });
        }
        goods.set_ex_id(create_guid());
        Some(goods)
    }

    /// Полный exact overload `CreateGoods(goods_index)`: оба loader-а идут
    /// после GUID и вызываются даже для headgear без профильных addon-ов.
    pub(crate) fn create_goods<Random, Guid, FairyThreshold, BattleFairyThreshold>(
        &self,
        goods_index: u32,
        random: Random,
        create_guid: Guid,
        fairy_threshold_for_level: FairyThreshold,
        battle_fairy_threshold_for_level: BattleFairyThreshold,
    ) -> Option<CGoods>
    where
        Random: FnMut(i32) -> i32,
        Guid: FnMut() -> CGuid,
        FairyThreshold: FnMut(u32, u32) -> u32,
        BattleFairyThreshold: FnMut(u32, u32) -> u32,
    {
        let mut goods = self.create_goods_core(goods_index, random, create_guid)?;
        goods
            .load_fairy_properties(self, fairy_threshold_for_level)
            .expect("catalog entry проверена create_goods_core");
        goods
            .load_battle_fairy_property(self, battle_fairy_threshold_for_level)
            .expect("catalog entry проверена create_goods_core");
        Some(goods)
    }

    /// Exact overload `CreateGoods(goods_index, amount, vector)`: stackable
    /// типы дробятся по limit с `id == 1`, остальные создаются поштучно.
    pub(crate) fn create_goods_batch<Random, Guid, FairyThreshold, BattleFairyThreshold>(
        &self,
        goods_index: u32,
        mut amount: u32,
        mut random: Random,
        mut create_guid: Guid,
        mut fairy_threshold_for_level: FairyThreshold,
        mut battle_fairy_threshold_for_level: BattleFairyThreshold,
    ) -> Vec<CGoods>
    where
        Random: FnMut(i32) -> i32,
        Guid: FnMut() -> CGuid,
        FairyThreshold: FnMut(u32, u32) -> u32,
        BattleFairyThreshold: FnMut(u32, u32) -> u32,
    {
        let Some(properties) = self.query_goods_base_properties(goods_index) else {
            return Vec::new();
        };
        let mut created = Vec::new();
        let stackable = matches!(
            properties.goods_type(),
            GOODS_TYPE_CONSUMABLE | GOODS_TYPE_USELESS
        );
        let maximum = if stackable {
            properties
                .get_addon_property_values(GAP_GOODS_STACKING_LIMIT)
                .iter()
                .find(|value| value.id == 1)
                .map(|value| value.base_value)
                .filter(|value| *value > 0)
                .map_or(1, |value| value as u32)
        } else {
            1
        };
        while amount != 0 {
            let created_goods = self.create_goods(
                goods_index,
                &mut random,
                &mut create_guid,
                &mut fairy_threshold_for_level,
                &mut battle_fairy_threshold_for_level,
            );
            if stackable {
                let Some(mut goods) = created_goods else {
                    break;
                };
                let stack = amount.min(maximum);
                goods.set_amount(stack);
                amount = amount.wrapping_sub(stack);
                created.push(goods);
            } else {
                if let Some(goods) = created_goods {
                    created.push(goods);
                }
                amount = amount.wrapping_sub(1);
            }
        }
        created
    }

    pub(crate) fn release(&mut self) {
        self.goods.clear();
        self.original_name_index.clear();
        self.name_index.clear();
    }

    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<GoodsFactoryDecodeReport, GoodsFactoryDecodeError> {
        self.release();
        let count = read_factory_u32(source, cursor, "goods count")?;
        for _ in 0..count {
            let goods_id = read_factory_u32(source, cursor, "goods id")?;
            let mut properties = CGoodsBaseProperties::default();
            properties
                .unserialize(source, cursor)
                .map_err(GoodsFactoryDecodeError::BaseProperties)?;
            self.original_name_index
                .insert(properties.original_name().to_vec(), goods_id);
            self.name_index.insert(properties.name().to_vec(), goods_id);
            self.goods.insert(goods_id, properties);
        }
        Ok(GoodsFactoryDecodeReport {
            declared_records: count as usize,
            unique_goods: self.goods.len(),
            unique_original_names: self.original_name_index.len(),
            unique_names: self.name_index.len(),
        })
    }

    pub(crate) const fn goods(&self) -> &BTreeMap<u32, CGoodsBaseProperties> {
        &self.goods
    }

    pub(crate) fn query_goods_base_properties(
        &self,
        goods_id: u32,
    ) -> Option<&CGoodsBaseProperties> {
        self.goods.get(&goods_id)
    }

    pub(crate) fn query_goods_original_name(&self, goods_id: u32) -> Option<&[u8]> {
        self.query_goods_base_properties(goods_id)
            .map(CGoodsBaseProperties::original_name)
    }

    pub(crate) fn query_goods_name(&self, goods_id: u32) -> Option<&[u8]> {
        self.query_goods_base_properties(goods_id)
            .map(CGoodsBaseProperties::name)
    }

    pub(crate) fn query_goods_id_by_original_name(&self, name: Option<&[u8]>) -> u32 {
        name.map(visible_c_string)
            .and_then(|name| self.original_name_index.get(name).copied())
            .unwrap_or(0)
    }

    pub(crate) fn query_goods_id_by_name(&self, name: Option<&[u8]>) -> u32 {
        name.map(visible_c_string)
            .and_then(|name| self.name_index.get(name).copied())
            .unwrap_or(0)
    }

    pub(crate) fn query_goods_base_properties_by_original_name(
        &self,
        name: Option<&[u8]>,
    ) -> Option<&CGoodsBaseProperties> {
        self.query_goods_base_properties(self.query_goods_id_by_original_name(name))
    }

    pub(crate) fn query_goods_base_properties_by_name(
        &self,
        name: Option<&[u8]>,
    ) -> Option<&CGoodsBaseProperties> {
        self.query_goods_base_properties(self.query_goods_id_by_name(name))
    }

    pub(crate) fn get_gold_coin_index(&self) -> u32 {
        self.query_goods_id_by_original_name(Some(b"MONEY"))
    }

    pub(crate) fn get_yuan_bao_index(&self) -> u32 {
        self.query_goods_id_by_original_name(Some(b"YUANBAO"))
    }

    pub(crate) fn get_ji_fen_index(&self) -> u32 {
        self.query_goods_id_by_original_name(Some(b"JIFEN"))
    }
}

const fn ordinary_upgrade_pair(property: i32) -> Option<i32> {
    Some(match property {
        GAP_MINIMUM_ATTACK_UPGRADE => GAP_MINIMUM_ATTACK_CORRECTION,
        GAP_MAXIMUM_ATTACK_UPGRADE => GAP_MAXIMUM_ATTACK_CORRECTION,
        GAP_ELEMENT_ATTACK_UPGRADE => GAP_ELEMENT_ATTACK_CORRECTION,
        GAP_ARMOR_UPGRADE => GAP_ARMOR_CORRECTION,
        GAP_ATTACK_SPEED_UPGRADE => GAP_ATTACK_SPEED_CORRECTION,
        GAP_HIT_RATE_UPGRADE => GAP_HIT_RATE_CORRECTION,
        GAP_FATAL_BLOW_RATE_UPGRADE => GAP_FATAL_BLOW_RATE_CORRECTION,
        GAP_DODGE_UPGRADE => GAP_DODGE_CORRECTION,
        GAP_ROLE_MINIMUM_LEVEL_LIMIT_UPGRADE => GAP_ROLE_MINIMUM_LEVEL_LIMIT,
        GAP_ROLE_MINIMUM_STRENGTH_LIMIT_UPGRADE => GAP_ROLE_MINIMUM_STRENGTH_LIMIT,
        GAP_ROLE_MINIMUM_AGILITY_LIMIT_UPGRADE => GAP_ROLE_MINIMUM_AGILITY_LIMIT,
        GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT_UPGRADE => GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT,
        GAP_ROLE_MINIMUM_WAKAN_LIMIT_UPGRADE => GAP_ROLE_MINIMUM_WAKAN_LIMIT,
        GAP_GOODS_MAXIMUM_DURABILITY_UPGRADE => GAP_GOODS_MAXIMUM_DURABILITY,
        GAP_HP_UPPER_LIMIT_CORRECTION_UPGRADE => GAP_HP_UPPER_LIMIT_CORRECTION,
        GAP_MP_UPPER_LIMIT_CORRECTION_UPGRADE => GAP_MP_UPPER_LIMIT_CORRECTION,
        GAP_SKILL_REUSE_TIME_CORRECTION_UPGRADE => GAP_SKILL_REUSE_TIME_CORRECTION,
        GAP_STIFFEN_PROBABILITY_CORRECTION_UPGRADE => GAP_STIFFEN_PROBABILITY_CORRECTION,
        GAP_BURDEN_UPPER_LIMIT_CORRECTION_UPGRADE => GAP_BURDEN_UPPER_LIMIT_CORRECTION,
        GAP_ELEMENT_RESISTANCE_CORRECTION_UPGRADE => GAP_ELEMENT_RESISTANCE_CORRECTION,
        GAP_WEAPON_DAMAGE_UPGRADE => GAP_WEAPON_DAMAGE_LEVEL,
        _ => return None,
    })
}

const fn battle_fairy_growth_pair(property_type: i32) -> Option<(i32, i32)> {
    Some(match property_type {
        GAP_BF_LIFE_ADDON => (GAP_BF_LIFE_ADDON, GAP_BF_LIFE_GROW),
        GAP_BF_MP_ADDON => (GAP_BF_MP_ADDON, GAP_BF_MP_GROW),
        GAP_BF_ATTACK_ADDON => (GAP_BF_ATTACK_ADDON, GAP_BF_ATTACK_GROW),
        GAP_BF_SPRITE_ADDON => (GAP_BF_SPRITE_ADDON, GAP_BF_SPRITE_GROW),
        GAP_BF_ABRAVE_ADDON => (GAP_BF_ABRAVE_ADDON, GAP_BF_ABRAVE_GROW),
        GAP_BF_AGILITY_ADDON => (GAP_BF_AGILITY_ADDON, GAP_BF_AGILITY_GROW),
        GAP_BF_SPRITUALISE_ADDON => (GAP_BF_SPRITUALISE_ADDON, GAP_BF_SPRITUALISE_GROW),
        GAP_BF_STRENGH_ADDON => (GAP_BF_STRENGH_ADDON, GAP_BF_STRENGH_GROW),
        _ => return None,
    })
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn read_factory_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, GoodsFactoryDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        GoodsFactoryDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            available: block.available,
        }
    })?;
    let value = reader.read_u32().map_err(|block| GoodsFactoryDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        available: block.available,
    })?;
    *cursor = reader.position();
    Ok(value)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp

// ============================================================================
// FUNCTION: CGoodsFactory::GarbageCollect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:481
// RVA: 0x00063A20
// ADDRESS: 00463a20
// PROTOTYPE: int __cdecl GarbageCollect(CGoods * * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::AddExterndProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1133
// RVA: 0x00063AA0
// ADDRESS: 00463aa0
// PROTOTYPE: int __cdecl AddExterndProperty(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::EquipmentWaste
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:623
// RVA: 0x00063E50
// ADDRESS: 00463e50
// PROTOTYPE: int __cdecl EquipmentWaste(CGoods * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::UnserializeGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:443
// RVA: 0x000640A0
// ADDRESS: 004640a0
// PROTOTYPE: CGoods * __cdecl UnserializeGoods(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `Upgrade` и `UpgradeEquipment` материализованы выше;
// полностью замещённые RAW-тела удалены.

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsMaxStackNumber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:896
// RVA: 0x00066F70
// ADDRESS: 00466f70
// PROTOTYPE: ulong __cdecl QueryGoodsMaxStackNumber(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsBasePropertiesValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:919
// RVA: 0x000670D0
// ADDRESS: 004670d0
// PROTOTYPE: long __cdecl QueryGoodsBasePropertiesValue(ulong param_1, GOODS_ADDON_PROPERTIES param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CreateAddonProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:240
// RVA: 0x00068010
// ADDRESS: 00468010
// PROTOTYPE: void __cdecl CreateAddonProperties(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CreateGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:312
// RVA: 0x000682E0
// ADDRESS: 004682e0
// PROTOTYPE: CGoods * __cdecl CreateGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DealWithExternAttr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1066
// RVA: 0x00068580
// ADDRESS: 00468580
// PROTOTYPE: int __cdecl DealWithExternAttr(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CutPreData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1362
// RVA: 0x00068700
// ADDRESS: 00468700
// PROTOTYPE: int __cdecl CutPreData(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryPropertyValue7
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1458
// RVA: 0x00068AB0
// ADDRESS: 00468ab0
// PROTOTYPE: long __cdecl QueryPropertyValue7(CGoods * param_1, GOODS_ADDON_PROPERTIES param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DaKongDeluxModify
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1628
// RVA: 0x00068ED0
// ADDRESS: 00468ed0
// PROTOTYPE: int __cdecl DaKongDeluxModify(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DealEnchaseGem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:974
// RVA: 0x000690E0
// ADDRESS: 004690e0
// PROTOTYPE: int __cdecl DealEnchaseGem(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DaKongModify
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1348
// RVA: 0x00069470
// ADDRESS: 00469470
// PROTOTYPE: int __cdecl DaKongModify(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DaKongXiangQian
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:968
// RVA: 0x000694A0
// ADDRESS: 004694a0
// PROTOTYPE: int __cdecl DaKongXiangQian(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::ReCreateAddonProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:167
// RVA: 0x000694C0
// ADDRESS: 004694c0
// PROTOTYPE: void __cdecl ReCreateAddonProperties(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
