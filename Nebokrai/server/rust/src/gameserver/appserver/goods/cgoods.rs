//! Достигнутый object/addon core `CGoods` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходные owners
//! `server/gameserver/appserver/goods/cgoods.h/.cpp`. Материализованы shape
//! identity, base-properties index, amount/price/add-ticket/description,
//! ordered addon storage, first-match lookup с fallback в registry, exact
//! instance-addon mutation, stack classification/limit, equipment-upgrade
//! eligibility, timed equipment start-point и wrapping weight.
//! `Vec` и owned bytes заменяют MSVC storage, не меняя порядка и signed 32-bit
//! arithmetic.
//! Единственный legacy null-deref в `CanStacked` при потерянном registry key
//! выражен typed block-ом, а не тихим `false`.
//!
//! Constructor/release, fairy/battle-fairy, остальная durability/time, полный
//! codec и mutation gameplay ниже остаются RAW: достигнутый core не выдаётся
//! за весь 0xCC-byte legacy object. `CGoodsFactory` передаётся явно вместо
//! исходного process-global registry.

use super::cgoodsbaseproperties::{
    CGoodsBaseProperties, GAP_DAKONG_1, GAP_GOODS_LIFE_TYPE, GAP_GOODS_STACKING_LIMIT,
    GAP_GOODS_START_POINT, GAP_WEAPON_LEVEL, GOODS_TYPE_CONSUMABLE, GOODS_TYPE_EQUIPMENT,
    GOODS_TYPE_USELESS,
};
use super::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::public::guid::CGuid;

const GOODS_OBJECT_TYPE: i32 = 700;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsBasePropertyBlock {
    pub(crate) index: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsAddonPropertyValue {
    pub(crate) id: u32,
    pub(crate) base_value: i32,
    pub(crate) modifier: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsAddonProperty {
    pub(crate) property_type: i32,
    pub(crate) is_enabled: i32,
    pub(crate) is_implicit_attribute: i32,
    pub(crate) values: Vec<GoodsAddonPropertyValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGoods {
    shape: CShape,
    base_properties_index: u32,
    amount: u32,
    price: u32,
    price_type: u32,
    add_ticket: u32,
    description: Vec<u8>,
    addon_properties: Vec<GoodsAddonProperty>,
}

impl Default for CGoods {
    fn default() -> Self {
        Self::with_reached_constructor_defaults()
    }
}

impl CGoods {
    /// Достигнутый scalar/storage prefix constructor-а RVA `0x000CC3A0`.
    pub(crate) const fn with_reached_constructor_defaults() -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.base_object_mut().set_type(GOODS_OBJECT_TYPE);
        Self {
            shape,
            base_properties_index: 0,
            amount: 1,
            price: 0,
            price_type: 0,
            add_ticket: 0,
            description: Vec::new(),
            addon_properties: Vec::new(),
        }
    }

    pub(crate) const fn identity(&self) -> ShapeIdentity {
        self.shape.identity()
    }

    pub(crate) const fn set_ex_id(&mut self, ex_id: CGuid) {
        self.shape.base_object_mut().set_ex_id(ex_id);
    }

    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.shape.base_object_mut().set_name(name);
    }

    pub(crate) fn name(&self) -> &[u8] {
        self.shape.base_object().get_name()
    }

    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.shape.base_object_mut().set_graphics_id(graphics_id);
    }

    pub(crate) const fn set_base_properties_index(&mut self, index: u32) {
        self.base_properties_index = index;
    }

    pub(crate) const fn base_properties_index(&self) -> u32 {
        self.base_properties_index
    }

    pub(crate) const fn set_amount(&mut self, amount: u32) {
        self.amount = amount;
    }

    pub(crate) const fn amount(&self) -> u32 {
        self.amount
    }

    pub(crate) const fn set_price(&mut self, price: u32) {
        self.price = price;
    }

    pub(crate) const fn price(&self) -> u32 {
        self.price
    }

    pub(crate) const fn price_type(&self) -> u32 {
        self.price_type
    }

    pub(crate) const fn add_ticket(&self) -> u32 {
        self.add_ticket
    }

    pub(crate) fn set_add_ticket(&mut self, add_ticket: u32) {
        if !self.query_attribute(GAP_GOODS_STACKING_LIMIT) {
            self.add_ticket = add_ticket;
        }
    }

    pub(crate) fn set_description(&mut self, description: &[u8]) {
        let visible = description
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(description.len());
        self.description.clear();
        self.description.extend_from_slice(&description[..visible]);
    }

    pub(crate) fn description(&self) -> &[u8] {
        &self.description
    }

    pub(crate) fn clear_addon_properties(&mut self) {
        self.addon_properties.clear();
    }

    pub(crate) fn addon_properties(&self) -> &[GoodsAddonProperty] {
        &self.addon_properties
    }

    pub(crate) fn addon_properties_mut(&mut self) -> &mut Vec<GoodsAddonProperty> {
        &mut self.addon_properties
    }

    pub(crate) fn push_addon_property(&mut self, property: GoodsAddonProperty) {
        self.addon_properties.push(property);
    }

    /// Prefix `CopyAddonProperties`; fairy reload остаётся у незамкнутого
    /// suffix-owner-а и потому не скрывается этим именем.
    pub(crate) fn copy_addon_properties_core_from(&mut self, source: &Self) {
        self.addon_properties.clone_from(&source.addon_properties);
    }

    pub(crate) fn query_attribute(&self, property_type: i32) -> bool {
        self.addon_properties
            .iter()
            .any(|property| property.property_type == property_type)
    }

    /// Exact `HasAddonPropertyValues` смотрит только catalog addon-values и
    /// не проверяет instance storage.
    pub(crate) fn has_addon_property_values(
        &self,
        factory: &CGoodsFactory,
        property_type: i32,
    ) -> bool {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .is_some_and(|properties| {
                !properties
                    .get_addon_property_values(property_type)
                    .is_empty()
            })
    }

    /// Exact `CanUpgraded` проверяет catalog type, но сам upgrade marker ищет
    /// только среди instance-addon-ов. Registry fallback здесь не применяется.
    pub(crate) fn can_upgraded(&self, factory: &CGoodsFactory) -> bool {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .is_some_and(|properties| {
                properties.goods_type() == GOODS_TYPE_EQUIPMENT
                    && self.query_attribute(GAP_WEAPON_LEVEL)
            })
    }

    /// Exact `QueryDaKongCount` считает только непрерывный prefix семи
    /// instance/base addon-слотов со значением value-id 1 в диапазоне 2..=8.
    pub(crate) fn da_kong_count(&self, factory: &CGoodsFactory) -> u32 {
        let mut count = 0;
        for offset in 0..=6 {
            let value = self.addon_property_value(factory, GAP_DAKONG_1 + offset, 1);
            if value < 2 {
                return count;
            }
            if 8 < value {
                break;
            }
            count += 1;
        }
        count
    }

    pub(crate) fn addon_property_value(
        &self,
        factory: &CGoodsFactory,
        property_type: i32,
        value_id: u32,
    ) -> i32 {
        if let Some(value) = self
            .addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .and_then(|property| property.values.iter().find(|value| value.id == value_id))
        {
            return value.base_value.wrapping_add(value.modifier);
        }
        factory
            .query_goods_base_properties(self.base_properties_index)
            .and_then(|properties| {
                properties
                    .get_addon_property_values(property_type)
                    .iter()
                    .find(|value| value.id == value_id)
            })
            .map_or(0, |value| value.base_value)
    }

    /// Storage-prefix `SetAddonPropertyValue` меняет modifier первого
    /// совпавшего value во всех instance-addon-ах данного типа и не создаёт
    /// отсутствующие записи. Registry fallback при записи не используется;
    /// последующий fairy/battle-fairy reload остаётся у их owner-ов.
    pub(crate) fn set_addon_property_value_core(
        &mut self,
        property_type: i32,
        value_id: u32,
        value: i32,
    ) -> bool {
        let mut changed = false;
        for property in self
            .addon_properties
            .iter_mut()
            .filter(|property| property.property_type == property_type)
        {
            if let Some(found) = property
                .values
                .iter_mut()
                .find(|candidate| candidate.id == value_id)
            {
                found.modifier = value.wrapping_sub(found.base_value);
                changed = true;
            }
        }
        changed
    }

    /// Safe replacement pointer-а `lCurrentExp`: первый instance value
    /// возвращает именно modifier, не сумму base+modifier.
    pub(crate) fn instance_addon_modifier(&self, property_type: i32, value_id: u32) -> Option<i32> {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .and_then(|property| property.values.iter().find(|value| value.id == value_id))
            .map(|value| value.modifier)
    }

    pub(crate) fn set_instance_addon_modifier(
        &mut self,
        property_type: i32,
        value_id: u32,
        modifier: i32,
    ) -> bool {
        let Some(value) = self
            .addon_properties
            .iter_mut()
            .find(|property| property.property_type == property_type)
            .and_then(|property| {
                property
                    .values
                    .iter_mut()
                    .find(|value| value.id == value_id)
            })
        else {
            return false;
        };
        value.modifier = modifier;
        true
    }

    pub(crate) fn goods_time_type(&self, factory: &CGoodsFactory) -> u32 {
        self.addon_property_value(factory, GAP_GOODS_LIFE_TYPE, 2) as u32
    }

    pub(crate) fn start_point(&self, factory: &CGoodsFactory) -> u64 {
        let high = self.addon_property_value(factory, GAP_GOODS_START_POINT, 1) as u32;
        let low = self.addon_property_value(factory, GAP_GOODS_START_POINT, 2) as u32;
        (u64::from(high) << 32) | u64::from(low)
    }

    /// Legacy setter последовательно пишет low, затем high. На повреждённой
    /// addon-схеме первая запись может состояться без второй, что намеренно не
    /// сворачивается в атомарную замену.
    pub(crate) fn set_start_point(&mut self, start_point: u64) {
        let _ =
            self.set_addon_property_value_core(GAP_GOODS_START_POINT, 2, start_point as u32 as i32);
        let _ = self.set_addon_property_value_core(
            GAP_GOODS_START_POINT,
            1,
            (start_point >> 32) as u32 as i32,
        );
    }

    /// Timed-prefix `CEquipmentContainer::Add`: типы 2/4 получают текущую
    /// точку только при нулевом старте. Возвращает факт попытки legacy setter-а.
    pub(crate) fn initialize_equipment_start_point(
        &mut self,
        factory: &CGoodsFactory,
        now: u64,
    ) -> bool {
        if matches!(self.goods_time_type(factory), 2 | 4) && self.start_point(factory) == 0 {
            self.set_start_point(now);
            true
        } else {
            false
        }
    }

    pub(crate) fn can_stack(
        &self,
        factory: &CGoodsFactory,
    ) -> Result<bool, GoodsBasePropertyBlock> {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .ok_or(GoodsBasePropertyBlock {
                index: self.base_properties_index,
            })
            .map(|properties| {
                matches!(
                    properties.goods_type(),
                    GOODS_TYPE_CONSUMABLE | GOODS_TYPE_USELESS
                ) && properties.has_addon_property(GAP_GOODS_STACKING_LIMIT)
            })
    }

    pub(crate) fn max_stack_number(&self, factory: &CGoodsFactory) -> u32 {
        let Some(properties) = factory.query_goods_base_properties(self.base_properties_index)
        else {
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
            .map_or(1, |value| value.base_value as u32)
    }

    pub(crate) fn weight(&self, factory: &CGoodsFactory) -> u32 {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .map_or(0, |properties: &CGoodsBaseProperties| {
                properties.weight().wrapping_mul(self.amount)
            })
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp

// ============================================================================
// FUNCTION: CGoods::IsFairy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.h:270
// RVA: 0x000AEB20
// ADDRESS: 004aeb20
// PROTOTYPE: bool __thiscall IsFairy(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:599
// RVA: 0x000C9730
// ADDRESS: 004c9730
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:604
// RVA: 0x000C9750
// ADDRESS: 004c9750
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetOriginalName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1839
// RVA: 0x000C97D0
// ADDRESS: 004c97d0
// PROTOTYPE: char * __thiscall GetOriginalName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::HatchBegin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1245
// RVA: 0x000C9840
// ADDRESS: 004c9840
// PROTOTYPE: bool __thiscall HatchBegin(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::HatchStop
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1260
// RVA: 0x000C9880
// ADDRESS: 004c9880
// PROTOTYPE: bool __thiscall HatchStop(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CanUpgraded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:130
// RVA: 0x000C9930
// ADDRESS: 004c9930
// PROTOTYPE: int __thiscall CanUpgraded(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CanBFEquipeUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1769
// RVA: 0x000C9A10
// ADDRESS: 004c9a10
// PROTOTYPE: int __thiscall CanBFEquipeUpgrade(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetAddonPropertyBaseValues
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:375
// RVA: 0x000C9AB0
// ADDRESS: 004c9ab0
// PROTOTYPE: int __thiscall SetAddonPropertyBaseValues(GOODS_ADDON_PROPERTIES param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:733
// RVA: 0x000C9B80
// ADDRESS: 004c9b80
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetCurDurability
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:989
// RVA: 0x000C9C80
// ADDRESS: 004c9c80
// PROTOTYPE: long __thiscall GetCurDurability(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetCurDurability
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1012
// RVA: 0x000C9D30
// ADDRESS: 004c9d30
// PROTOTYPE: long __thiscall SetCurDurability(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::LoadFairyPropertiesFromGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1042
// RVA: 0x000C9E00
// ADDRESS: 004c9e00
// PROTOTYPE: bool __thiscall LoadFairyPropertiesFromGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SaveFairyPropertiesToGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1165
// RVA: 0x000CA1C0
// ADDRESS: 004ca1c0
// PROTOTYPE: void __thiscall SaveFairyPropertiesToGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::LoadBFPropertyFromGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1660
// RVA: 0x000CA340
// ADDRESS: 004ca340
// PROTOTYPE: bool __thiscall LoadBFPropertyFromGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetAddonPropertyModifier
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:397
// RVA: 0x000CA6C0
// ADDRESS: 004ca6c0
// PROTOTYPE: int __thiscall SetAddonPropertyModifier(GOODS_ADDON_PROPERTIES param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetAddonPropertyValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:427
// RVA: 0x000CA7B0
// ADDRESS: 004ca7b0
// PROTOTYPE: int __thiscall SetAddonPropertyValue(GOODS_ADDON_PROPERTIES param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:540
// RVA: 0x000CA8C0
// ADDRESS: 004ca8c0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetGoodsLifeTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1806
// RVA: 0x000CA9B0
// ADDRESS: 004ca9b0
// PROTOTYPE: void __thiscall SetGoodsLifeTime(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetGoodsTimeType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1815
// RVA: 0x000CA9D0
// ADDRESS: 004ca9d0
// PROTOTYPE: void __thiscall SetGoodsTimeType(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetStartPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1825
// RVA: 0x000CA9F0
// ADDRESS: 004ca9f0
// PROTOTYPE: void __thiscall SetStartPoint(__uint64 param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::SetName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:833
// RVA: 0x000CADC0
// ADDRESS: 004cadc0
// PROTOTYPE: void __thiscall SetName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:718
// RVA: 0x000CB030
// ADDRESS: 004cb030
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::HasAddonPropertyValues
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:360
// RVA: 0x000CB3B0
// ADDRESS: 004cb3b0
// PROTOTYPE: bool __thiscall HasAddonPropertyValues(GOODS_ADDON_PROPERTIES param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::Clone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:521
// RVA: 0x000CB470
// ADDRESS: 004cb470
// PROTOTYPE: int __thiscall Clone(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::tagAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:702
// RVA: 0x000CB530
// ADDRESS: 004cb530
// PROTOTYPE: undefined __thiscall tagAddonProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::QueryAttrbuteInGoodsList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1308
// RVA: 0x000CB550
// ADDRESS: 004cb550
// PROTOTYPE: bool __thiscall QueryAttrbuteInGoodsList(GOODS_ADDON_PROPERTIES param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::QueryDaKongCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1336
// RVA: 0x000CB640
// ADDRESS: 004cb640
// PROTOTYPE: int __thiscall QueryDaKongCount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::QueryEnchanseColor
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1519
// RVA: 0x000CB680
// ADDRESS: 004cb680
// PROTOTYPE: int __thiscall QueryEnchanseColor(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::QueryNoInSelfPropertyType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1561
// RVA: 0x000CB7C0
// ADDRESS: 004cb7c0
// PROTOTYPE: long __thiscall QueryNoInSelfPropertyType(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SaveBFPropertyToGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1575
// RVA: 0x000CB830
// ADDRESS: 004cb830
// PROTOTYPE: void __thiscall SaveBFPropertyToGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetGoodsLifeTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1810
// RVA: 0x000CBA00
// ADDRESS: 004cba00
// PROTOTYPE: ulong __thiscall GetGoodsLifeTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetGoodsTimeType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1820
// RVA: 0x000CBA10
// ADDRESS: 004cba10
// PROTOTYPE: ulong __thiscall GetGoodsTimeType(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetStartPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1831
// RVA: 0x000CBA20
// ADDRESS: 004cba20
// PROTOTYPE: __uint64 __thiscall GetStartPoint(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CanReparied
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:110
// RVA: 0x000CBA80
// ADDRESS: 004cba80
// PROTOTYPE: int __thiscall CanReparied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetEnabledAddonProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:206
// RVA: 0x000CBAC0
// ADDRESS: 004cbac0
// PROTOTYPE: void __thiscall GetEnabledAddonProperties(vector<CGoodsBaseProperties::GOODS_ADDON_PROPERTIES,std::allocator<CGoodsBaseProperties::GOODS_ADDON_PROPERTIES>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetAddonPropertyValues
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:250
// RVA: 0x000CBBE0
// ADDRESS: 004cbbe0
// PROTOTYPE: void __thiscall GetAddonPropertyValues(GOODS_ADDON_PROPERTIES param_1, vector<CGoods::tagAddonPropertyValue,std::allocator<CGoods::tagAddonPropertyValue>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:749
// RVA: 0x000CBDC0
// ADDRESS: 004cbdc0
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SerializeForOldClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:862
// RVA: 0x000CBE90
// ADDRESS: 004cbe90
// PROTOTYPE: int __thiscall SerializeForOldClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:23
// RVA: 0x000CC3A0
// ADDRESS: 004cc3a0
// PROTOTYPE: undefined __thiscall CGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:617
// RVA: 0x000CC440
// ADDRESS: 004cc440
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::~CGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:48
// RVA: 0x000CC580
// ADDRESS: 004cc580
// PROTOTYPE: void __thiscall ~CGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetFuMoProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:458
// RVA: 0x000CC610
// ADDRESS: 004cc610
// PROTOTYPE: int __thiscall SetFuMoProperty(GOODS_ADDON_PROPERTIES param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:559
// RVA: 0x000CC7F0
// ADDRESS: 004cc7f0
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CopyAddonProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1236
// RVA: 0x000CC9A0
// ADDRESS: 004cc9a0
// PROTOTYPE: void __thiscall CopyAddonProperties(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CutAddonPropertyValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1386
// RVA: 0x000CC9D0
// ADDRESS: 004cc9d0
// PROTOTYPE: bool __thiscall CutAddonPropertyValue(GOODS_ADDON_PROPERTIES param_1, ulong param_2, int param_3, int param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CopyBFAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1762
// RVA: 0x000CCDC0
// ADDRESS: 004ccdc0
// PROTOTYPE: void __thiscall CopyBFAddonProperty(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonPropertyValue::~tagAddonPropertyValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:661
// RVA: 0x000D3D00
// ADDRESS: 004d3d00
// PROTOTYPE: void __thiscall ~tagAddonPropertyValue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d421d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D421D
// ADDRESS: 004d421d
// PROTOTYPE: undefined Catch@004d421d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d42d6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D42D6
// ADDRESS: 004d42d6
// PROTOTYPE: undefined Catch@004d42d6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d44c1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D44C1
// ADDRESS: 004d44c1
// PROTOTYPE: undefined Catch@004d44c1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::~tagAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:711
// RVA: 0x000D4660
// ADDRESS: 004d4660
// PROTOTYPE: void __thiscall ~tagAddonProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d490a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D490A
// ADDRESS: 004d490a
// PROTOTYPE: undefined Catch@004d490a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d49be
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D49BE
// ADDRESS: 004d49be
// PROTOTYPE: undefined Catch@004d49be()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d4c1e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D4C1E
// ADDRESS: 004d4c1e
// PROTOTYPE: undefined Catch@004d4c1e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d4cdd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D4CDD
// ADDRESS: 004d4cdd
// PROTOTYPE: undefined Catch@004d4cdd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d4e7c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D4E7C
// ADDRESS: 004d4e7c
// PROTOTYPE: undefined Catch@004d4e7c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d5290
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D5290
// ADDRESS: 004d5290
// PROTOTYPE: undefined Catch@004d5290()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d5354
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D5354
// ADDRESS: 004d5354
// PROTOTYPE: undefined Catch@004d5354()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d5554
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D5554
// ADDRESS: 004d5554
// PROTOTYPE: undefined Catch@004d5554()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d55e3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D55E3
// ADDRESS: 004d55e3
// PROTOTYPE: undefined Catch@004d55e3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
