//! Базовые свойства `CGoodsBaseProperties` из `cgoodsbaseproperties.cpp/.h`.
//!
//! Владелец хранит byte-exact имена и описание, цену, вес, goods/equipment type,
//! icons и дерево addon properties. Serialize намеренно не включает description:
//! wire содержит два имени, scalars, icons и полное addon-дерево.
//!
//! Lookup addon values останавливается на первом property нужного типа и
//! сохраняет порядок всех его values. Известные numeric enum/property IDs и
//! signedness соответствуют World wire. `Vec` и borrowed slices заменяют STL
//! копии и ручной lifecycle без изменения результатов.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

pub const GOODS_TYPE_USELESS: i32 = 0;
pub const GOODS_TYPE_CONSUMABLE: i32 = 1;
pub const GOODS_TYPE_EQUIPMENT: i32 = 2;
pub const GAP_PARTICULAR_ATTRIBUTE: i32 = 0x0d;
pub const GAP_GOODS_STACKING_LIMIT: i32 = 0x26;
pub const GAP_WEAPON_LEVEL: i32 = 0x30;
pub const ICON_TYPE_CONTAINER: i32 = 0;
pub const ICON_TYPE_GROUND: i32 = 1;
pub const ICON_TYPE_EQUIPPED: i32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoodsBasePropertiesCodecError {
    field: &'static str,
    count: usize,
}

impl fmt::Display for GoodsBasePropertiesCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "коллекция {} содержит {} элементов вне 32-битного legacy-диапазона",
            self.field, self.count
        )
    }
}

impl Error for GoodsBasePropertiesCodecError {}

#[derive(Clone)]
struct GoodsBaseIcon {
    icon_type: i32,
    icon_id: u32,
}

#[derive(Clone)]
pub struct GoodsBaseAddonPropertyValueModifier {
    probability: u32,
    lower_limit: i32,
    upper_limit: i32,
}

#[derive(Clone)]
pub struct GoodsBaseAddonPropertyValue {
    id: u32,
    base_value: i32,
    is_modifier_enabled: i32,
    modifiers: Vec<GoodsBaseAddonPropertyValueModifier>,
}

#[derive(Clone)]
struct GoodsBaseAddonProperty {
    property_type: i32,
    is_enabled: i32,
    is_implicit_attribute: i32,
    occur_probability: u32,
    values: Vec<GoodsBaseAddonPropertyValue>,
}

#[derive(Clone)]
pub struct CGoodsBaseProperties {
    original_name: Vec<u8>,
    name: Vec<u8>,
    description: Vec<u8>,
    price: u32,
    weight: u32,
    icons: Vec<GoodsBaseIcon>,
    goods_type: i32,
    equip_place: i32,
    addon_properties: Vec<GoodsBaseAddonProperty>,
}

/// Реестр base properties по goods index — исходная карта `CGoodsFactory`
/// без повторного lookup-правила; совпадает со старым
/// `GoodsBasePropertiesRegistry` по типу.
pub type GoodsBasePropertiesRegistry = BTreeMap<u32, Option<CGoodsBaseProperties>>;

impl CGoodsBaseProperties {
    pub const fn with_constructor_defaults() -> Self {
        Self {
            original_name: Vec::new(),
            name: Vec::new(),
            description: Vec::new(),
            price: 0,
            weight: 0,
            icons: Vec::new(),
            goods_type: GOODS_TYPE_USELESS,
            equip_place: 0,
            addon_properties: Vec::new(),
        }
    }

    pub fn get_original_name(&self) -> &[u8] {
        &self.original_name
    }

    pub fn get_name(&self) -> &[u8] {
        &self.name
    }

    pub fn get_description(&self) -> &[u8] {
        &self.description
    }

    pub const fn get_price(&self) -> u32 {
        self.price
    }

    pub const fn get_weight(&self) -> u32 {
        self.weight
    }

    pub const fn get_goods_type(&self) -> i32 {
        self.goods_type
    }

    pub const fn get_equip_place(&self) -> i32 {
        self.equip_place
    }

    pub fn get_icon_id(&self, icon_type: i32) -> u32 {
        self.icons
            .iter()
            .find(|icon| icon.icon_type == icon_type)
            .map_or(0, |icon| icon.icon_id)
    }

    pub fn valid_addon_property_types(&self) -> impl Iterator<Item = i32> + '_ {
        self.addon_properties
            .iter()
            .filter(|property| property.is_enabled == 1)
            .map(|property| property.property_type)
    }

    pub fn is_implicit(&self, property_type: i32) -> i32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(0, |property| property.is_implicit_attribute)
    }

    pub fn get_addon_property_values(&self, property_type: i32) -> &[GoodsBaseAddonPropertyValue] {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(&[], |property| property.values.as_slice())
    }

    pub fn get_occur_probability(&self, property_type: i32) -> u32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(0, |property| property.occur_probability)
    }

    pub fn serialize(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), GoodsBasePropertiesCodecError> {
        append_c_string(destination, &self.original_name);
        append_c_string(destination, &self.name);
        destination.extend_from_slice(&self.goods_type.to_le_bytes());
        destination.extend_from_slice(&self.equip_place.to_le_bytes());
        destination.extend_from_slice(&self.price.to_le_bytes());
        destination.extend_from_slice(&self.weight.to_le_bytes());
        append_count(destination, "m_vIcons", self.icons.len())?;
        for icon in &self.icons {
            destination.extend_from_slice(&icon.icon_type.to_le_bytes());
            destination.extend_from_slice(&icon.icon_id.to_le_bytes());
        }
        append_count(
            destination,
            "m_vAddonProperties",
            self.addon_properties.len(),
        )?;
        for property in &self.addon_properties {
            property.serialize(destination)?;
        }
        Ok(())
    }

    pub fn set_loaded_names(
        &mut self,
        original_name: Vec<u8>,
        name: Vec<u8>,
        description: Vec<u8>,
    ) {
        self.original_name = original_name;
        self.name = name;
        self.description = description;
    }

    pub const fn set_loaded_scalars(&mut self, price: u32, weight: u32, raw_type: u32) {
        self.price = price;
        self.weight = weight;
        if raw_type == 1 {
            self.goods_type = GOODS_TYPE_CONSUMABLE;
        } else if raw_type >= 2 && raw_type <= 17 {
            self.goods_type = GOODS_TYPE_EQUIPMENT;
            self.equip_place = raw_type as i32 - 1;
        }
    }

    pub fn push_loaded_icon(&mut self, icon_type: i32, icon_id: u32) {
        self.icons.push(GoodsBaseIcon { icon_type, icon_id });
    }

    pub fn push_loaded_addon_property(
        &mut self,
        property_type: u16,
        is_enabled: bool,
        is_implicit_attribute: bool,
        first_base_value: i32,
        second_base_value: i32,
        occur_probability: u16,
    ) {
        let mut property = GoodsBaseAddonProperty::with_constructor_defaults();
        property.property_type = i32::from(property_type);
        property.is_enabled = i32::from(is_enabled);
        property.is_implicit_attribute = i32::from(is_implicit_attribute);
        property.occur_probability = u32::from(occur_probability);

        let mut first_value = GoodsBaseAddonPropertyValue::with_constructor_defaults();
        first_value.id = 1;
        first_value.base_value = first_base_value;
        property.values.push(first_value);

        let mut second_value = GoodsBaseAddonPropertyValue::with_constructor_defaults();
        second_value.id = 2;
        second_value.base_value = second_base_value;
        property.values.push(second_value);

        self.addon_properties.push(property);
    }

    pub fn push_loaded_modifier(
        &mut self,
        property_index: usize,
        value_id: u32,
        lower_limit: i32,
        upper_limit: i32,
        probability: u16,
    ) -> bool {
        let Some(value) = self
            .addon_properties
            .get_mut(property_index)
            .and_then(|property| {
                property
                    .values
                    .iter_mut()
                    .find(|value| value.id == value_id)
            })
        else {
            return false;
        };
        value.is_modifier_enabled = 1;
        value.modifiers.push(GoodsBaseAddonPropertyValueModifier {
            probability: u32::from(probability),
            lower_limit,
            upper_limit,
        });
        true
    }
}

impl GoodsBaseAddonPropertyValue {
    const fn with_constructor_defaults() -> Self {
        Self {
            id: 0,
            base_value: 0,
            is_modifier_enabled: 0,
            modifiers: Vec::new(),
        }
    }

    pub const fn id(&self) -> u32 {
        self.id
    }

    pub const fn base_value(&self) -> i32 {
        self.base_value
    }

    pub const fn is_modifier_enabled(&self) -> bool {
        self.is_modifier_enabled != 0
    }

    pub fn modifiers(&self) -> &[GoodsBaseAddonPropertyValueModifier] {
        &self.modifiers
    }

    fn serialize(&self, destination: &mut Vec<u8>) -> Result<(), GoodsBasePropertiesCodecError> {
        destination.extend_from_slice(&self.id.to_le_bytes());
        destination.extend_from_slice(&self.base_value.to_le_bytes());
        destination.extend_from_slice(&self.is_modifier_enabled.to_le_bytes());
        append_count(
            destination,
            "tagAddonPropertyValue.vModifiers",
            self.modifiers.len(),
        )?;
        for modifier in &self.modifiers {
            modifier.serialize(destination);
        }
        Ok(())
    }
}

impl GoodsBaseAddonPropertyValueModifier {
    pub const fn probability(&self) -> u32 {
        self.probability
    }

    pub const fn lower_limit(&self) -> i32 {
        self.lower_limit
    }

    pub const fn upper_limit(&self) -> i32 {
        self.upper_limit
    }

    fn serialize(&self, destination: &mut Vec<u8>) {
        destination.extend_from_slice(&self.probability.to_le_bytes());
        destination.extend_from_slice(&self.lower_limit.to_le_bytes());
        destination.extend_from_slice(&self.upper_limit.to_le_bytes());
    }
}

impl GoodsBaseAddonProperty {
    const fn with_constructor_defaults() -> Self {
        Self {
            property_type: 0,
            is_enabled: 0,
            is_implicit_attribute: 0,
            occur_probability: 0,
            values: Vec::new(),
        }
    }

    fn serialize(&self, destination: &mut Vec<u8>) -> Result<(), GoodsBasePropertiesCodecError> {
        destination.extend_from_slice(&self.property_type.to_le_bytes());
        destination.extend_from_slice(&self.is_enabled.to_le_bytes());
        destination.extend_from_slice(&self.is_implicit_attribute.to_le_bytes());
        destination.extend_from_slice(&self.occur_probability.to_le_bytes());
        append_count(destination, "tagAddonProperty.vValues", self.values.len())?;
        for value in &self.values {
            value.serialize(destination)?;
        }
        Ok(())
    }
}

fn append_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    let visible = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    destination.extend_from_slice(&value[..visible]);
    destination.push(0);
}

fn append_count(
    destination: &mut Vec<u8>,
    field: &'static str,
    count: usize,
) -> Result<(), GoodsBasePropertiesCodecError> {
    let legacy_count =
        u32::try_from(count).map_err(|_| GoodsBasePropertiesCodecError { field, count })?;
    destination.extend_from_slice(&legacy_count.to_le_bytes());
    Ok(())
}
