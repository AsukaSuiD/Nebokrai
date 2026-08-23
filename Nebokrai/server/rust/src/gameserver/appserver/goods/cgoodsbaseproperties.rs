//! Базовые свойства товара GameServer и их startup wire-decoder.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/goods/cgoodsbaseproperties.cpp`. Парный
//! WorldServer serializer пишет два NUL-terminated имени, type/place/price/
//! weight, icons и вложенное дерево addon properties. Порядок элементов,
//! signedness scalar-ов, first-match lookup и last-write-free vector semantics
//! сохранены.
//!
//! `Vec` и обычное владение Rust заменяют MSVC allocator/destructor plumbing.
//! Повреждённые count/string границы, где старый код уходил в out-of-bounds,
//! завершаются typed error-ом с уже применённым prefix state.

use std::error::Error;
use std::fmt;

pub(crate) const GOODS_TYPE_USELESS: i32 = 0;
pub(crate) const GOODS_TYPE_CONSUMABLE: i32 = 1;
pub(crate) const GOODS_TYPE_EQUIPMENT: i32 = 2;
pub(crate) const GAP_PARTICULAR_ATTRIBUTE: i32 = 0x0d;
pub(crate) const GAP_GOODS_STACKING_LIMIT: i32 = 0x26;
pub(crate) const GAP_WEAPON_LEVEL: i32 = 0x30;
pub(crate) const GAP_GEM_TYPE: i32 = 79;
pub(crate) const GAP_BAOSHI_COLOR: i32 = 139;
pub(crate) const GAP_DAKONG_1: i32 = 140;
pub(crate) const GAP_BF_GEM: i32 = 177;
pub(crate) const GAP_GOODS_PACKAGE_EXTENTION: i32 = 234;
pub(crate) const ICON_TYPE_GROUND: i32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsBasePropertiesDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        required: usize,
        available: usize,
    },
    MissingStringTerminator {
        field: &'static str,
        offset: usize,
        available: usize,
    },
}

impl fmt::Display for GoodsBasePropertiesDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                required,
                available,
            } => write!(
                formatter,
                "goods properties обрываются на {field} в {offset}: нужно {required}, доступно {available}"
            ),
            Self::MissingStringTerminator {
                field,
                offset,
                available,
            } => write!(
                formatter,
                "goods properties не содержат NUL для {field} в {offset} ({available} байт)"
            ),
        }
    }
}

impl Error for GoodsBasePropertiesDecodeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsBaseIcon {
    pub(crate) icon_type: i32,
    pub(crate) icon_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsBaseAddonPropertyValueModifier {
    pub(crate) probability: u32,
    pub(crate) lower_limit: i32,
    pub(crate) upper_limit: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsBaseAddonPropertyValue {
    pub(crate) id: u32,
    pub(crate) base_value: i32,
    pub(crate) is_modifier_enabled: i32,
    pub(crate) modifiers: Vec<GoodsBaseAddonPropertyValueModifier>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsBaseAddonProperty {
    pub(crate) property_type: i32,
    pub(crate) is_enabled: i32,
    pub(crate) is_implicit_attribute: i32,
    pub(crate) occur_probability: u32,
    pub(crate) values: Vec<GoodsBaseAddonPropertyValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGoodsBaseProperties {
    original_name: Vec<u8>,
    name: Vec<u8>,
    description: Vec<u8>,
    goods_type: i32,
    equip_place: i32,
    price: u32,
    weight: u32,
    icons: Vec<GoodsBaseIcon>,
    addon_properties: Vec<GoodsBaseAddonProperty>,
}

impl Default for CGoodsBaseProperties {
    fn default() -> Self {
        Self {
            original_name: Vec::new(),
            name: Vec::new(),
            description: Vec::new(),
            goods_type: GOODS_TYPE_USELESS,
            equip_place: 0,
            price: 0,
            weight: 0,
            icons: Vec::new(),
            addon_properties: Vec::new(),
        }
    }
}

impl CGoodsBaseProperties {
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), GoodsBasePropertiesDecodeError> {
        self.clear();
        self.original_name = read_c_string(source, cursor, "original name")?;
        self.name = read_c_string(source, cursor, "name")?;
        self.goods_type = read_i32(source, cursor, "goods type")?;
        self.equip_place = read_i32(source, cursor, "equip place")?;
        self.price = read_u32(source, cursor, "price")?;
        self.weight = read_u32(source, cursor, "weight")?;

        let icon_count = read_u32(source, cursor, "icon count")?;
        for _ in 0..icon_count {
            self.icons.push(GoodsBaseIcon {
                icon_type: read_i32(source, cursor, "icon type")?,
                icon_id: read_u32(source, cursor, "icon id")?,
            });
        }

        let property_count = read_u32(source, cursor, "addon property count")?;
        for _ in 0..property_count {
            self.addon_properties
                .push(GoodsBaseAddonProperty::unserialize(source, cursor)?);
        }
        Ok(())
    }

    pub(crate) fn original_name(&self) -> &[u8] {
        &self.original_name
    }

    pub(crate) fn name(&self) -> &[u8] {
        &self.name
    }

    pub(crate) fn description(&self) -> &[u8] {
        &self.description
    }

    pub(crate) const fn goods_type(&self) -> i32 {
        self.goods_type
    }

    pub(crate) const fn equip_place(&self) -> i32 {
        self.equip_place
    }

    pub(crate) const fn price(&self) -> u32 {
        self.price
    }

    pub(crate) const fn weight(&self) -> u32 {
        self.weight
    }

    pub(crate) fn icons(&self) -> &[GoodsBaseIcon] {
        &self.icons
    }

    pub(crate) fn addon_properties(&self) -> &[GoodsBaseAddonProperty] {
        &self.addon_properties
    }

    pub(crate) fn get_icon_id(&self, icon_type: i32) -> u32 {
        self.icons
            .iter()
            .find(|icon| icon.icon_type == icon_type)
            .map_or(0, |icon| icon.icon_id)
    }

    pub(crate) fn get_occur_probability(&self, property_type: i32) -> u32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(0, |property| property.occur_probability)
    }

    pub(crate) fn is_implicit(&self, property_type: i32) -> i32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(0, |property| property.is_implicit_attribute)
    }

    pub(crate) fn has_addon_property(&self, property_type: i32) -> bool {
        self.addon_properties
            .iter()
            .any(|property| property.property_type == property_type)
    }

    pub(crate) fn get_addon_property_value(&self, property_type: i32, has_values: bool) -> i32 {
        self.addon_properties
            .iter()
            .find(|property| {
                property.property_type == property_type && !property.values.is_empty() == has_values
            })
            .and_then(|property| property.values.first())
            .map_or(0, |value| value.base_value)
    }

    pub(crate) fn get_addon_property_values(
        &self,
        property_type: i32,
    ) -> &[GoodsBaseAddonPropertyValue] {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(&[], |property| property.values.as_slice())
    }

    pub(crate) fn query_addon_max_property_value(&self, property_type: i32, value_id: u32) -> i32 {
        let Some(value) = self
            .get_addon_property_values(property_type)
            .iter()
            .find(|value| value.id == value_id)
        else {
            return 0;
        };
        let maximum_modifier = value
            .modifiers
            .iter()
            .map(|modifier| modifier.upper_limit)
            .max()
            .unwrap_or(0)
            .max(0);
        value.base_value.wrapping_add(maximum_modifier)
    }

    pub(crate) fn valid_addon_properties(&self) -> impl Iterator<Item = i32> + '_ {
        self.addon_properties
            .iter()
            .filter(|property| property.is_enabled == 1)
            .map(|property| property.property_type)
    }

    pub(crate) fn all_addon_property_values(&self) -> Vec<GoodsBaseAddonProperty> {
        self.addon_properties.clone()
    }
}

impl GoodsBaseAddonProperty {
    fn unserialize(
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<Self, GoodsBasePropertiesDecodeError> {
        let property_type = read_i32(source, cursor, "addon property type")?;
        let is_enabled = read_i32(source, cursor, "addon enabled")?;
        let is_implicit_attribute = read_i32(source, cursor, "addon implicit")?;
        let occur_probability = read_u32(source, cursor, "addon probability")?;
        let value_count = read_u32(source, cursor, "addon value count")?;
        let mut values = Vec::new();
        for _ in 0..value_count {
            values.push(GoodsBaseAddonPropertyValue::unserialize(source, cursor)?);
        }
        Ok(Self {
            property_type,
            is_enabled,
            is_implicit_attribute,
            occur_probability,
            values,
        })
    }
}

impl GoodsBaseAddonPropertyValue {
    fn unserialize(
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<Self, GoodsBasePropertiesDecodeError> {
        let id = read_u32(source, cursor, "addon value id")?;
        let base_value = read_i32(source, cursor, "addon base value")?;
        let is_modifier_enabled = read_i32(source, cursor, "modifier enabled")?;
        let modifier_count = read_u32(source, cursor, "modifier count")?;
        let mut modifiers = Vec::new();
        for _ in 0..modifier_count {
            modifiers.push(GoodsBaseAddonPropertyValueModifier {
                probability: read_u32(source, cursor, "modifier probability")?,
                lower_limit: read_i32(source, cursor, "modifier lower limit")?,
                upper_limit: read_i32(source, cursor, "modifier upper limit")?,
            });
        }
        Ok(Self {
            id,
            base_value,
            is_modifier_enabled,
            modifiers,
        })
    }
}

fn read_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<Vec<u8>, GoodsBasePropertiesDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let remaining = source.get(offset..).unwrap_or_default();
    let Some(length) = remaining.iter().position(|byte| *byte == 0) else {
        return Err(GoodsBasePropertiesDecodeError::MissingStringTerminator {
            field,
            offset,
            available,
        });
    };
    *cursor = offset + length + 1;
    Ok(remaining[..length].to_vec())
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, GoodsBasePropertiesDecodeError> {
    Ok(i32::from_le_bytes(
        take_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("i32 содержит четыре байта"),
    ))
}

fn read_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, GoodsBasePropertiesDecodeError> {
    Ok(u32::from_le_bytes(
        take_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("u32 содержит четыре байта"),
    ))
}

fn take_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    required: usize,
    field: &'static str,
) -> Result<&'a [u8], GoodsBasePropertiesDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(required) else {
        return Err(GoodsBasePropertiesDecodeError::UnexpectedEnd {
            field,
            offset,
            required,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(GoodsBasePropertiesDecodeError::UnexpectedEnd {
            field,
            offset,
            required,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}
