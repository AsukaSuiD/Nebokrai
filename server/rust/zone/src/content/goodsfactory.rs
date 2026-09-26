//! Реестр запуска и поисковая часть `CGoodsFactory` исторического GameServer:
//! создание/улучшение экземпляров, цены магазина NPC и DaKong-алгоритм камней
//! исходного владельца. Исходный владелец `appserver/goods/cgoodsfactory.cpp`;
//! сверка по точной паре `gameserver.exe` + `GameServer.pdb`.
//!
//! Селектор `0x00` сначала освобождает три прежние карты, затем читает счётчик
//! `u32` и упорядоченные записи `goods_id + CGoodsBaseProperties`; ID и оба
//! индекса строковых байтов используют последнюю запись. Парный сериализатор
//! WorldServer подтверждает формат обмена. Invariant-ы: пошаговый
//! `UpgradeBFEquipment` меняет уровень и восемь зависящих от роста дополнений в
//! исходном порядке шага; `EquipmentWaste` вычитает fray из итоговой прочности,
//! но записывает результат в `base_value`; `ReCreateBattleFairyAttributes`
//! повторяет полный набор бросков RNG по числу дополнений экземпляра —
//! однопроходная оптимизация донора не переносится, потому что меняла итоговое
//! состояние игрового RNG. RNG поступает generic-замыканиями вызывающей стороны
//! (`with_legacy_random_stream` остаётся в `CGame`); таблицы `CDaKongXiangQian`
//! — Shared resources. Обычное улучшение ещё требует реконструкции.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#предметы-и-контейнеры

use std::collections::BTreeMap;
use thiserror::Error;

use super::goods::{
    CGoodsBaseProperties, GAP_ARMOR_CORRECTION, GAP_ARMOR_UPGRADE, GAP_ATTACK_SPEED_CORRECTION,
    GAP_ATTACK_SPEED_UPGRADE, GAP_BAOSHI_COLOR, GAP_BF_ABRAVE_ADDON, GAP_BF_ABRAVE_GROW,
    GAP_BF_AGILITY_ADDON, GAP_BF_AGILITY_BASE, GAP_BF_AGILITY_GROW, GAP_BF_ATTACK_ADDON,
    GAP_BF_ATTACK_BASE, GAP_BF_ATTACK_GROW, GAP_BF_BATTLE_FAIRY, GAP_BF_BRAVE_BASE,
    GAP_BF_LIFE_ADDON, GAP_BF_LIFE_GROW, GAP_BF_MP_ADDON, GAP_BF_MP_GROW, GAP_BF_PULLULATERATE,
    GAP_BF_SPRITE_ADDON, GAP_BF_SPRITE_BASE, GAP_BF_SPRITE_GROW, GAP_BF_SPRITUALISE_ADDON,
    GAP_BF_SPRITUALISE_GROW, GAP_BF_SPRITUALISM_BASE, GAP_BF_STRENGH_ADDON, GAP_BF_STRENGH_BASE,
    GAP_BF_STRENGH_GROW, GAP_BF_WEAPON_LEVEL, GAP_BURDEN_UPPER_LIMIT_CORRECTION,
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
    GoodsBaseAddonPropertyValue, GoodsBasePropertiesDecodeError, ICON_TYPE_GROUND,
};
use crate::items::cgoods::{
    CGoods, GoodsAddonProperty, GoodsAddonPropertyValue, GoodsBasePropertiesLookup,
};
use crate::regions::ShapeIdentity;
use nebokrai_shared::protocol::LegacyReader;
use nebokrai_shared::resources::CDaKongXiangQian;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GoodsFactoryDecodeError {
    #[error("goods registry обрывается на {field} в {offset}: нужно 4, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        available: usize,
    },
    #[error(transparent)]
    BaseProperties(#[from] GoodsBasePropertiesDecodeError),
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
pub struct CGoodsFactory {
    goods: BTreeMap<u32, CGoodsBaseProperties>,
    original_name_index: BTreeMap<Vec<u8>, u32>,
    name_index: BTreeMap<Vec<u8>, u32>,
}

impl CGoodsFactory {
    pub fn calculate_repair_price(&self, goods: &CGoods, repair_factor: f32) -> u32 {
        if !goods.can_repair(self) {
            return 0;
        }
        let current = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 2);
        let maximum = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 1);
        let quality = goods.addon_property_value(self, GAP_ITEM_QUALITY, 1);
        let factor = if 0 < quality {
            (f64::from(quality.wrapping_mul(50).wrapping_add(150))
                * 0.01_f64
                * f64::from(repair_factor)) as f32
        } else {
            repair_factor
        };
        let damage = ((f64::from(maximum.wrapping_sub(current)) / f64::from(maximum)) as f32)
            .max(0.0);
        (f64::from(goods.price()) * f64::from(damage) * f64::from(factor)).trunc()
            as i32 as u32
    }

    pub fn calculate_vend_price(
        &self,
        goods: &CGoods,
        base_price_rate: f32,
        trade_in_rate: f32,
    ) -> u32 {
        if goods.price() == 0 {
            return 0;
        }
        let stored_price = goods.price() as f32;
        let mut price = f64::from(stored_price);
        if self
            .query_goods_base_properties(goods.base_properties_index())
            .is_some_and(|properties| properties.goods_type() == GOODS_TYPE_EQUIPMENT)
        {
            let current = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 2);
            let maximum = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 1);
            if maximum != 0 {
                // Primary x86 делит integer durability до преобразования во float.
                let durability = current.min(maximum) / maximum;
                let durability_component = (f64::from(goods.price())
                    * f64::from(durability)
                    * (1.0 - f64::from(base_price_rate)))
                    as f32;
                price = f64::from(goods.price()) * f64::from(base_price_rate)
                    + f64::from(durability_component);
            }
        }
        (f64::from(trade_in_rate) * price).trunc() as i32 as u32
    }

    pub fn repair_equipment(&self, goods: &mut CGoods) -> bool {
        goods.repair_durability(self)
    }

    /// Exact scalar `EquipmentWaste`: проверяет итоговый maximum, вычитает
    /// fray из текущего итогового durability, но записывает результат именно
    /// в instance `base_value`, сохраняя отдельный modifier.
    pub fn equipment_waste(&self, goods: &mut CGoods, fray: i32) -> bool {
        if goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 1) <= 0 {
            return false;
        }
        let current = goods.addon_property_value(self, GAP_GOODS_MAXIMUM_DURABILITY, 2);
        let _ = goods.set_addon_property_base_value_first_core(
            GAP_GOODS_MAXIMUM_DURABILITY,
            2,
            current.wrapping_sub(fray),
        );
        true
    }

    /// Exact `ReCreateBattleFairyAttributes`: исходный owner повторяет весь
    /// набор бросков для каждого instance-addon-а и тем самым оставляет в
    /// товаре последний RNG-pass. Это намеренно не свёрнуто в один бросок.
    pub fn recreate_battle_fairy_attributes<Random>(
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
    pub fn query_goods_max_stack_number(&self, goods_index: u32) -> u32 {
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
    pub fn upgrade_equipment<Random>(
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
    pub fn recreate_addon_properties<Random>(&self, goods: &mut CGoods, mut random: Random)
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
        let Some(recreated) = self.create_goods_template(base_index, |maximum| random(maximum)) else {
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
    pub fn da_kong_modify(&self, goods: &mut CGoods) {
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
    pub fn da_kong_xiang_qian<Random>(
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
    pub fn da_kong_delux_modify(&self, goods: &mut CGoods, setup: &CDaKongXiangQian) {
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
                                GAP_BAOSHI_COLOR,
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
    pub fn upgrade_battle_fairy_equipment(
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

    /// Создание экземпляра по `CreateGoods` RVA `0x000682E0` до загрузки фей.
    /// GUID запрашивается после бросков свойств; отказ не выдаёт экземпляр.
    pub fn create_goods_core<Random, Guid>(
        &self,
        goods_index: u32,
        random: Random,
        mut create_guid: Guid,
    ) -> Option<CGoods>
    where
        Random: FnMut(i32) -> i32,
        Guid: FnMut() -> Option<CGuid>,
    {
        let mut goods = self.create_goods_template(goods_index, random)?;
        goods.set_ex_id(create_guid().filter(|guid| !guid.is_invalid())?);
        Some(goods)
    }

    /// Временные свойства без идентичности для пересчёта и проверки камней.
    /// Шаблон нельзя публиковать или помещать в инвентарь как новый предмет.
    pub fn create_goods_template<Random>(
        &self,
        goods_index: u32,
        mut random: Random,
    ) -> Option<CGoods>
    where
        Random: FnMut(i32) -> i32,
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
        Some(goods)
    }

    /// `CreateGoods(goods_index)`: оба loader-а идут
    /// после GUID и вызываются даже для headgear без профильных addon-ов.
    pub fn create_goods<Random, Guid, FairyThreshold, BattleFairyThreshold>(
        &self,
        goods_index: u32,
        random: Random,
        create_guid: Guid,
        fairy_threshold_for_level: FairyThreshold,
        battle_fairy_threshold_for_level: BattleFairyThreshold,
    ) -> Option<CGoods>
    where
        Random: FnMut(i32) -> i32,
        Guid: FnMut() -> Option<CGuid>,
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

    /// `CreateGoods(goods_index, amount, vector)`: stackable типы дробятся
    /// по limit с `id == 1`, остальные создаются поштучно. Отказ GUID отменяет
    /// подготовленный набор целиком: вызывающий не должен оплатить недостачу.
    pub fn create_goods_batch<Random, Guid, FairyThreshold, BattleFairyThreshold>(
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
        Guid: FnMut() -> Option<CGuid>,
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
            let Some(mut goods) = self.create_goods(
                goods_index,
                &mut random,
                &mut create_guid,
                &mut fairy_threshold_for_level,
                &mut battle_fairy_threshold_for_level,
            ) else {
                // Набор ещё не передан владельцу инвентаря. Броски игрового
                // RNG уже выполнены и не откатываются при отказе GUID.
                return Vec::new();
            };
            if stackable {
                let stack = amount.min(maximum);
                goods.set_amount(stack);
                amount = amount.wrapping_sub(stack);
                created.push(goods);
            } else {
                created.push(goods);
                amount = amount.wrapping_sub(1);
            }
        }
        created
    }

    pub fn release(&mut self) {
        self.goods.clear();
        self.original_name_index.clear();
        self.name_index.clear();
    }

    pub fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), GoodsFactoryDecodeError> {
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
        tracing::trace!(declared_records = count, unique_goods = self.goods.len(), unique_original_names = self.original_name_index.len(), unique_names = self.name_index.len(), "реестр предметов декодирован");
        Ok(())
    }

    pub const fn goods(&self) -> &BTreeMap<u32, CGoodsBaseProperties> {
        &self.goods
    }

    pub fn query_goods_base_properties(
        &self,
        goods_id: u32,
    ) -> Option<&CGoodsBaseProperties> {
        self.goods.get(&goods_id)
    }

    pub fn query_goods_original_name(&self, goods_id: u32) -> Option<&[u8]> {
        self.query_goods_base_properties(goods_id)
            .map(CGoodsBaseProperties::original_name)
    }

    pub fn query_goods_name(&self, goods_id: u32) -> Option<&[u8]> {
        self.query_goods_base_properties(goods_id)
            .map(CGoodsBaseProperties::name)
    }

    pub fn query_goods_id_by_original_name(&self, name: Option<&[u8]>) -> u32 {
        name.map(visible_c_string)
            .and_then(|name| self.original_name_index.get(name).copied())
            .unwrap_or(0)
    }

    pub fn query_goods_id_by_name(&self, name: Option<&[u8]>) -> u32 {
        name.map(visible_c_string)
            .and_then(|name| self.name_index.get(name).copied())
            .unwrap_or(0)
    }

    pub fn query_goods_base_properties_by_original_name(
        &self,
        name: Option<&[u8]>,
    ) -> Option<&CGoodsBaseProperties> {
        self.query_goods_base_properties(self.query_goods_id_by_original_name(name))
    }

    pub fn query_goods_base_properties_by_name(
        &self,
        name: Option<&[u8]>,
    ) -> Option<&CGoodsBaseProperties> {
        self.query_goods_base_properties(self.query_goods_id_by_name(name))
    }

    pub fn get_gold_coin_index(&self) -> u32 {
        self.query_goods_id_by_original_name(Some(b"MONEY"))
    }

    pub fn get_yuan_bao_index(&self) -> u32 {
        self.query_goods_id_by_original_name(Some(b"YUANBAO"))
    }

    pub fn get_ji_fen_index(&self) -> u32 {
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

/// Реализация lookup-шва `items/cgoods.rs` владельцем реестра; тело делегирует
/// inherent-поиску, имя операции сохраняет исходный `QueryGoodsBaseProperties`.
impl GoodsBasePropertiesLookup for CGoodsFactory {
    fn query_goods_base_properties(&self, goods_id: u32) -> Option<&CGoodsBaseProperties> {
        self.query_goods_base_properties(goods_id)
    }
}

// ============================================================================
// Точное DaKong-семейство исходного `cgoodsfactory.cpp`: снимки камня и
// предмета, типы событий вставки, условие седьмого слота, ветви
// `DealWithExternAttr` и пере/наложение свойств камней `DealEnchaseGem`.
// Реконструировано ранее в сессионном `cequipmentdakong.rs` и возвращено к
// владельцу фабрикой; сессионный файл держит реэкспорт для старого пакета.
// ============================================================================

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentDaKongGoodsSnapshot {
    pub identity: ShapeIdentity,
    pub base_index: u32,
    pub price: u32,
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub socket_and_external_values: Vec<(i32, u32, i32)>,
}

impl EquipmentDaKongGoodsSnapshot {
    pub fn capture(goods: &CGoods, factory: &CGoodsFactory) -> Self {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EquipmentDaKongGemSnapshot {
    pub identity: Option<ShapeIdentity>,
    pub base_index: u32,
    pub color: i32,
    pub condition: i32,
    pub temporary: bool,
}

impl EquipmentDaKongGemSnapshot {
    pub fn capture(goods: &CGoods, factory: &CGoodsFactory) -> Self {
        Self {
            identity: Some(goods.identity()),
            base_index: goods.base_properties_index(),
            color: goods.addon_property_value(factory, GAP_BAOSHI_COLOR, 1),
            condition: goods.addon_property_value(factory, GAP_BAOSHI_COLOR, 2),
            temporary: false,
        }
    }

    pub fn from_catalog(base_index: u32, factory: &CGoodsFactory) -> Option<Self> {
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
pub enum EquipmentDaKongEnchaseEvent {
    Notification(&'static str),
    GemApplied {
        gem: EquipmentDaKongGemSnapshot,
        equipment: EquipmentDaKongGoodsSnapshot,
        audit: bool,
    },
    Script(&'static [u8]),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EquipmentDaKongEnchaseEffects {
    pub events: Vec<EquipmentDaKongEnchaseEvent>,
}

pub fn equipment_da_kong_condition(
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
    values: &[GoodsBaseAddonPropertyValue],
) -> i32 {
    values
        .iter()
        .find(|value| value.id == 1)
        .or_else(|| values.first())
        .map_or(0, |value| value.base_value)
}

fn half_slot_seven_addition(value: i32) -> i32 {
    (f64::from(value) * 0.5).trunc() as i64 as i32
}

fn same_color_socket(equipment: &CGoods, factory: &CGoodsFactory, slot: i32) -> bool {
    let property = GAP_DAKONG_1 + slot - 1;
    let socket_color = equipment.addon_property_value(factory, property, 1);
    let gem_index = equipment.addon_property_value(factory, property, 2) as u32;
    base_gem_color(factory, gem_index) == socket_color
}

pub fn deal_with_da_kong_external_attributes(
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
pub fn apply_embedded_gem_properties<Random>(
    equipment: &mut CGoods,
    factory: &CGoodsFactory,
    mut random: Random,
) where
    Random: FnMut(i32) -> i32,
{
    let mut add_types = std::collections::BTreeSet::new();
    CDaKongXiangQian::get_add_type(&mut add_types);

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
                .create_goods_template(gem_index, |maximum| random(maximum))
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

pub fn deal_with_da_kong_seven(
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
    CDaKongXiangQian::get_add_type(&mut add_types);
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

pub fn deal_enchase_gems(
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
    CDaKongXiangQian::get_add_type(&mut add_types);
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
