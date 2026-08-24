//! Startup registry и lookup-часть `CGoodsFactory` GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/goods/cgoodsfactory.cpp`. Selector `0x00`
//! сначала освобождает три прежних map, затем читает `u32 count` и ordered
//! records `goods_id + CGoodsBaseProperties`; ID и оба byte-string index-а
//! используют last-write-wins. Lookup miss и null name возвращают ноль/`None`.
//!
//! Парный WorldServer serializer подтверждает wire. `BTreeMap` и owned values
//! заменяют MSVC tree/raw pointers без изменения порядка. Одиночное создание
//! предмета замкнуто вместе с обязательной загрузкой ordinary/battle-fairy
//! свойств. Пошаговый `UpgradeBFEquipment` меняет instance level и восемь
//! growth-зависимых addon-ов в исходном порядке каждого level step; прочая
//! CiQing batch-overload сохраняет дробление consumable/useless по stacking-
//! limit и поштучное создание остальных типов; ordinary upgrade mutation ниже
//! остаётся RAW.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::cgoods::{CGoods, GoodsAddonProperty, GoodsAddonPropertyValue};
use super::cgoodsbaseproperties::{
    CGoodsBaseProperties, GAP_ARMOR_CORRECTION, GAP_ARMOR_UPGRADE, GAP_ATTACK_SPEED_CORRECTION,
    GAP_ATTACK_SPEED_UPGRADE, GAP_BF_ABRAVE_ADDON, GAP_BF_ABRAVE_GROW, GAP_BF_AGILITY_ADDON,
    GAP_BF_AGILITY_GROW, GAP_BF_ATTACK_ADDON, GAP_BF_ATTACK_GROW, GAP_BF_LIFE_ADDON,
    GAP_BF_LIFE_GROW, GAP_BF_MP_ADDON, GAP_BF_MP_GROW, GAP_BF_SPRITE_ADDON, GAP_BF_SPRITE_GROW,
    GAP_BF_SPRITUALISE_ADDON, GAP_BF_SPRITUALISE_GROW, GAP_BF_STRENGH_ADDON, GAP_BF_STRENGH_GROW,
    GAP_BF_WEAPON_LEVEL, GAP_BURDEN_UPPER_LIMIT_CORRECTION,
    GAP_BURDEN_UPPER_LIMIT_CORRECTION_UPGRADE, GAP_DODGE_CORRECTION, GAP_DODGE_UPGRADE,
    GAP_ELEMENT_ATTACK_CORRECTION, GAP_ELEMENT_ATTACK_UPGRADE, GAP_ELEMENT_RESISTANCE_CORRECTION,
    GAP_ELEMENT_RESISTANCE_CORRECTION_UPGRADE, GAP_FATAL_BLOW_RATE_CORRECTION,
    GAP_FATAL_BLOW_RATE_UPGRADE, GAP_GOODS_MAXIMUM_DURABILITY,
    GAP_GOODS_MAXIMUM_DURABILITY_UPGRADE, GAP_GOODS_STACKING_LIMIT, GAP_HIT_RATE_CORRECTION,
    GAP_HIT_RATE_UPGRADE, GAP_HP_UPPER_LIMIT_CORRECTION, GAP_HP_UPPER_LIMIT_CORRECTION_UPGRADE,
    GAP_MAXIMUM_ATTACK_CORRECTION, GAP_MAXIMUM_ATTACK_UPGRADE, GAP_MINIMUM_ATTACK_CORRECTION,
    GAP_MINIMUM_ATTACK_UPGRADE, GAP_MP_UPPER_LIMIT_CORRECTION,
    GAP_MP_UPPER_LIMIT_CORRECTION_UPGRADE, GAP_ROLE_MINIMUM_AGILITY_LIMIT,
    GAP_ROLE_MINIMUM_AGILITY_LIMIT_UPGRADE, GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT,
    GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT_UPGRADE, GAP_ROLE_MINIMUM_LEVEL_LIMIT,
    GAP_ROLE_MINIMUM_LEVEL_LIMIT_UPGRADE, GAP_ROLE_MINIMUM_STRENGTH_LIMIT,
    GAP_ROLE_MINIMUM_STRENGTH_LIMIT_UPGRADE, GAP_ROLE_MINIMUM_WAKAN_LIMIT,
    GAP_ROLE_MINIMUM_WAKAN_LIMIT_UPGRADE, GAP_SKILL_REUSE_TIME_CORRECTION,
    GAP_SKILL_REUSE_TIME_CORRECTION_UPGRADE, GAP_STIFFEN_PROBABILITY_CORRECTION,
    GAP_STIFFEN_PROBABILITY_CORRECTION_UPGRADE, GAP_WEAPON_DAMAGE_LEVEL, GAP_WEAPON_DAMAGE_UPGRADE,
    GAP_WEAPON_LEVEL, GOODS_TYPE_CONSUMABLE, GOODS_TYPE_USELESS, GoodsBasePropertiesDecodeError,
    ICON_TYPE_GROUND,
};
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGoodsFactory {
    goods: BTreeMap<u32, CGoodsBaseProperties>,
    original_name_index: BTreeMap<Vec<u8>, u32>,
    name_index: BTreeMap<Vec<u8>, u32>,
}

impl CGoodsFactory {
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

    /// Переходит к target level по одному шагу. На каждом шаге growth-addon-ы
    /// применяются в instance insertion order, а level меняется через modifier
    /// value-id 1; отсутствие такого value завершает уже применённый prefix.
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
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(4)) else {
        return Err(GoodsFactoryDecodeError::UnexpectedEnd {
            field,
            offset,
            available,
        });
    };
    *cursor += 4;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("goods scalar содержит четыре байта"),
    ))
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
// FUNCTION: CGoodsFactory::RepairEquipment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:605
// RVA: 0x00063A50
// ADDRESS: 00463a50
// PROTOTYPE: int __cdecl RepairEquipment(CGoods * param_1)
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
// FUNCTION: CGoodsFactory::CalculateRepairPrice
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:497
// RVA: 0x00063D50
// ADDRESS: 00463d50
// PROTOTYPE: ulong __cdecl CalculateRepairPrice(CGoods * param_1)
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

// ============================================================================
// FUNCTION: CGoodsFactory::CalculateVendPrice
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:544
// RVA: 0x00064180
// ADDRESS: 00464180
// PROTOTYPE: ulong __cdecl CalculateVendPrice(CGoods * param_1)
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
