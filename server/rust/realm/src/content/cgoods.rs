//! Товар `CGoods` из `cgoods.cpp/.h`; источник контракта WorldServer — точная
//! пара `Nworldserver.exe` + `WorldServer.pdb` (идентификаторы —
//! `server/rust/src/manifest/_worldserver_export_manifest.toml`).
//!
//! Constructor создаёт базовый `CShape`, type `700`, amount `1` и оставляет
//! base-properties index неназначенным. Goods wire дописывает к Shape индекс,
//! amount, price, NUL-строку description и ordered addon records.
//! `Unserialize` сначала выполняет `Release`: очищает index, amount, description
//! и addons, но сохраняет price до чтения нового значения.
//!
//! Короткая или слишком длинная description отклоняется вместо переполнения
//! старого stack-buffer. Addon lookup берёт первое совпадение и использует
//! signed wrapping; max-stack и weight зависят от base-properties, weight
//! умножается с 32-битным wrapping.
//!
//! `CanUpgraded` возвращает true только когда base-properties по индексу
//! найдены, goods type == `GOODS_TYPE_EQUIPMENT` и addon values
//! `GAP_WEAPON_LEVEL` (0x30) непусты; иначе false.
//! Rust-владение и `Vec` заменяют ручной cleanup, не меняя wire или этот
//! предикат.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use crate::content::goodsdb::{
    GoodsAddonPropertySnapshot as DbGoodsAddonPropertySnapshot,
    GoodsAddonPropertyValue as DbGoodsAddonPropertyValue, GoodsAddonValueCountBlock,
    GoodsObjectSnapshot, GoodsPropertiesSnapshot,
};

use crate::regions::shape::CShape;
use crate::regions::shapetypes::ShapeDecodeError;
use crate::content::goods::{
    GAP_GOODS_STACKING_LIMIT, GAP_WEAPON_LEVEL, GOODS_TYPE_CONSUMABLE, GOODS_TYPE_EQUIPMENT,
    GOODS_TYPE_USELESS, GoodsBasePropertiesRegistry,
};
use crate::content::cgoodsfactory::query_goods_base_properties;

pub const GAP_GOODS_PACKAGE_EXTENTION: i32 = 234;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoodsCodecError {
    Shape(ShapeDecodeError),
    MissingBasePropertiesIndex,
    CollectionLengthOutsideLegacyRange {
        field: &'static str,
        count: usize,
    },
    UnterminatedDescription {
        offset: usize,
        available: usize,
    },
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for GoodsCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape(error) => error.fmt(formatter),
            Self::MissingBasePropertiesIndex => {
                formatter.write_str("m_dwBasePropertiesIndex ещё не назначен")
            }
            Self::CollectionLengthOutsideLegacyRange { field, count } => write!(
                formatter,
                "коллекция {field} содержит {count} элементов вне 32-битного legacy-диапазона"
            ),
            Self::UnterminatedDescription { offset, available } => write!(
                formatter,
                "m_strDescribe с offset {offset} не завершён NUL в legacy-буфере 1028 байт; доступно {available}"
            ),
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
        }
    }
}

impl Error for GoodsCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Shape(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ShapeDecodeError> for GoodsCodecError {
    fn from(error: ShapeDecodeError) -> Self {
        Self::Shape(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoodsDbSnapshotBlock {
    MissingBasePropertiesIndex,
    AddonValueCount {
        property_index: usize,
        value_count: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoodsLoadedAddonBlock {
    MissingBaseProperties { index: u32 },
    ExistingPropertyHasFewerThanTwoValues { property_type: i32, count: usize },
}

#[derive(Clone, Copy)]
pub struct GoodsAddonPropertyValue {
    id: u32,
    base_value: i32,
    modifier: i32,
}

pub struct GoodsAddonProperty {
    property_type: i32,
    is_enabled: i32,
    is_implicit_attribute: i32,
    values: Vec<GoodsAddonPropertyValue>,
}

/// Итог safe-доступа `CGoodsFactory::Upgrade` к первому destination-value.
///
/// оригинал owner находил первый совпавший property и без проверки разыменовывал
/// `vValues._Myfirst`. Пустой value-vector не является нормальным игровым
/// состоянием и не получает искусственной mutation в Rust.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirstAddonModifierAdjustment {
    MissingProperty,
    MissingValue,
    Adjusted,
}

pub struct CGoods {
    shape_base: CShape,
    base_properties_index: Option<u32>,
    amount: u32,
    price: u32,
    description: Vec<u8>,
    addon_properties: Vec<GoodsAddonProperty>,
}

impl CGoods {
    pub const fn with_constructor_base_and_type() -> Self {
        let mut shape_base = CShape::with_constructor_region_default();
        shape_base.set_type(700);
        Self {
            shape_base,
            base_properties_index: None,
            amount: 1,
            price: 0,
            description: Vec::new(),
            addon_properties: Vec::new(),
        }
    }

    pub const fn get_type(&self) -> i32 {
        self.shape_base.get_type()
    }

    pub const fn get_id(&self) -> i32 {
        self.shape_base.get_id()
    }

    pub const fn set_id(&mut self, id: i32) {
        self.shape_base.set_id(id);
    }

    pub const fn get_ex_id(&self) -> &nebokrai_shared::values::CGuid {
        self.shape_base.get_ex_id()
    }

    pub const fn set_ex_id(&mut self, ex_id: &nebokrai_shared::values::CGuid) {
        self.shape_base.set_ex_id(ex_id);
    }

    pub fn set_name(&mut self, name: &[u8]) {
        self.shape_base.set_name(name);
    }

    pub fn get_goods_name(&self) -> &[u8] {
        self.shape_base.get_name()
    }

    pub const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.shape_base.set_graphics_id(graphics_id);
    }

    pub const fn get_base_properties_index(&self) -> Option<u32> {
        self.base_properties_index
    }

    pub const fn set_base_properties_index(&mut self, index: u32) {
        self.base_properties_index = Some(index);
    }

    pub const fn set_amount(&mut self, amount: u32) {
        self.amount = amount;
    }

    pub const fn get_amount(&self) -> u32 {
        self.amount
    }

    pub fn get_addon_property_value(&self, property_type: i32, id: u32) -> i32 {
        self.get_addon_property_values(property_type)
            .iter()
            .find(|value| value.id == id)
            .map_or(0, |value| value.base_value.wrapping_add(value.modifier))
    }

    pub fn get_all_addon_properties(&self) -> &[GoodsAddonProperty] {
        &self.addon_properties
    }

    pub fn get_all_addon_properties_mut(&mut self) -> &mut Vec<GoodsAddonProperty> {
        &mut self.addon_properties
    }

    pub fn is_addon_property_exist(&self, property_type: i32) -> bool {
        self.addon_properties
            .iter()
            .any(|property| property.property_type == property_type)
    }

    pub fn apply_loaded_addon(
        &mut self,
        property_type: i32,
        first_modifier: i32,
        second_modifier: i32,
        registry: &GoodsBasePropertiesRegistry,
        dakong_addon_types: &BTreeSet<i32>,
    ) -> Result<(), GoodsLoadedAddonBlock> {
        if let Some(property) = self
            .addon_properties
            .iter_mut()
            .find(|property| property.property_type == property_type)
        {
            if property.values.len() < 2 {
                return Err(GoodsLoadedAddonBlock::ExistingPropertyHasFewerThanTwoValues {
                    property_type,
                    count: property.values.len(),
                });
            }
            property.values[0].modifier = first_modifier;
            if property_type == 0x25 {
                property.values[1].base_value = second_modifier;
            } else {
                property.values[1].modifier = second_modifier;
            }
            return Ok(());
        }

        let index = self
            .base_properties_index
            .ok_or(GoodsLoadedAddonBlock::MissingBaseProperties { index: 0 })?;
        let properties = query_goods_base_properties(registry, index)
            .ok_or(GoodsLoadedAddonBlock::MissingBaseProperties { index })?;
        let base_values = properties.get_addon_property_values(property_type);
        let values = if base_values.len() >= 2 {
            vec![
                (
                    base_values[0].id(),
                    base_values[0].base_value(),
                    first_modifier,
                ),
                (
                    base_values[1].id(),
                    base_values[1].base_value(),
                    second_modifier,
                ),
            ]
        } else if self.is_addon_property_exist(140)
            && dakong_addon_types.contains(&property_type)
        {
            vec![(1, 0, first_modifier), (2, 0, second_modifier)]
        } else {
            return Ok(());
        };
        self.push_factory_addon_property(
            property_type,
            properties.is_implicit(property_type),
            values,
        );
        Ok(())
    }

    pub fn get_addon_property_values(
        &self,
        property_type: i32,
    ) -> &[GoodsAddonPropertyValue] {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(&[], |property| property.values.as_slice())
    }

 /// Меняет modifier первого addon-value точным private factory-путём.
 ///
 /// Сначала выбирается первый property данного numeric-типа, затем его
 /// первый value. Сложение/вычитание происходят как signed 32-bit x86
 /// arithmetic; increase ветвь ограничивает только верх `65535`, а
 /// decrease ветвь — только нижний ноль.
    pub fn adjust_first_addon_modifier(
        &mut self,
        property_type: i32,
        delta: i32,
        increase: bool,
    ) -> FirstAddonModifierAdjustment {
        let Some(property) = self
            .addon_properties
            .iter_mut()
            .find(|property| property.property_type == property_type)
        else {
            return FirstAddonModifierAdjustment::MissingProperty;
        };
        let Some(value) = property.values.first_mut() else {
            return FirstAddonModifierAdjustment::MissingValue;
        };

        if increase {
            value.modifier = value.modifier.wrapping_add(delta);
            if value.modifier > 0xffff {
                value.modifier = 0xffff;
            }
        } else {
            value.modifier = value.modifier.wrapping_sub(delta);
            if value.modifier < 0 {
                value.modifier = 0;
            }
        }
        FirstAddonModifierAdjustment::Adjusted
    }

    pub fn get_max_stack_number(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, GoodsCodecError> {
        let index = self
            .base_properties_index
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        let Some(base_properties) = query_goods_base_properties(registry, index) else {
            return Ok(1);
        };
        if !matches!(
            base_properties.get_goods_type(),
            GOODS_TYPE_USELESS | GOODS_TYPE_CONSUMABLE
        ) {
            return Ok(1);
        }
        Ok(base_properties
            .get_addon_property_values(GAP_GOODS_STACKING_LIMIT)
            .iter()
            .find(|value| value.id() == 1)
            .map_or(1, |value| value.base_value() as u32))
    }

    pub fn get_weight(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, GoodsCodecError> {
        let index = self
            .base_properties_index
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        Ok(
            query_goods_base_properties(registry, index).map_or(0, |properties| {
                properties.get_weight().wrapping_mul(self.amount)
            }),
        )
    }

    /// `CGoods::CanUpgraded`.
    ///
    /// Индекс идёт в map-lookup global-реестра; промах → false;
    /// `GetGoodsType() != 2` → false; пустой result `GetAddonValues(0x30)` →
    /// false; иначе true. Ctor-неназначенный индекс (`None`) соответствует
    /// промаху lookup по неинициализированному base-полю. Машинное основание
    /// — docs/reconstruction/realm-services.md.
    pub fn can_upgraded(&self, registry: &GoodsBasePropertiesRegistry) -> bool {
        let Some(index) = self.base_properties_index else {
            return false;
        };
        let Some(properties) = query_goods_base_properties(registry, index) else {
            return false;
        };
        if properties.get_goods_type() != GOODS_TYPE_EQUIPMENT {
            return false;
        }
        !self.get_addon_property_values(GAP_WEAPON_LEVEL).is_empty()
    }

    /// DIRECT-case машинного `UpgradeEquipment`: у property по индексу
    /// итерируемого addon-вектора ищется первое value с id == 1 и его
    /// modifier меняется на ±1 обычным x86 wrapping без всякого clamp;
    /// changed-флаг цикла взводится только при найденном value.
    /// Машинное основание — docs/reconstruction/realm-services.md.
    pub fn adjust_indexed_id_one_modifier(&mut self, property_index: usize, increase: bool) -> bool {
        let Some(property) = self.addon_properties.get_mut(property_index) else {
            return false;
        };
        let Some(value) = property.values.iter_mut().find(|value| value.id == 1) else {
            return false;
        };
        if increase {
            value.modifier = value.modifier.wrapping_add(1);
        } else {
            value.modifier = value.modifier.wrapping_sub(1);
        }
        true
    }

    pub const fn set_price(&mut self, price: u32) {
        self.price = price;
    }

    pub const fn get_price(&self) -> u32 {
        self.price
    }

    pub fn db_save_snapshot(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<GoodsObjectSnapshot, GoodsDbSnapshotBlock> {
        let base_properties_index = self
            .base_properties_index
            .ok_or(GoodsDbSnapshotBlock::MissingBasePropertiesIndex)?;
        let properties = if self.addon_properties.is_empty() {
            GoodsPropertiesSnapshot::Available(Vec::new())
        } else if let Some(base_properties) =
            query_goods_base_properties(registry, base_properties_index)
        {
            let mut snapshots = Vec::with_capacity(self.addon_properties.len());
            for (property_index, property) in self.addon_properties.iter().enumerate() {
                let values = property
                    .values
                    .iter()
                    .map(|value| DbGoodsAddonPropertyValue {
                        id: value.id,
                        base_value: value.base_value,
                        modifier: value.modifier,
                    })
                    .collect();
                let snapshot = DbGoodsAddonPropertySnapshot::from_legacy_parts(
                    property.property_type as u32,
                    base_properties.get_occur_probability(property.property_type),
                    values,
                )
                .map_err(|GoodsAddonValueCountBlock { value_count }| {
                    GoodsDbSnapshotBlock::AddonValueCount {
                        property_index,
                        value_count,
                    }
                })?;
                snapshots.push(snapshot);
            }
            GoodsPropertiesSnapshot::Available(snapshots)
        } else {
            GoodsPropertiesSnapshot::MissingBaseProperties
        };

        Ok(GoodsObjectSnapshot {
            goods_id: *self.get_ex_id(),
            base_properties_index,
            name: self.shape_base.get_name().to_vec(),
            price: self.price,
            amount: self.amount,
            properties,
        })
    }

    pub fn set_goods_description(&mut self, description: &[u8]) {
        let visible = description
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(description.len());
        self.description.clear();
        self.description.extend_from_slice(&description[..visible]);
    }

    pub fn push_factory_addon_property(
        &mut self,
        property_type: i32,
        is_implicit_attribute: i32,
        values: Vec<(u32, i32, i32)>,
    ) {
        self.addon_properties.push(GoodsAddonProperty {
            property_type,
            is_enabled: 1,
            is_implicit_attribute,
            values: values
                .into_iter()
                .map(|(id, base_value, modifier)| GoodsAddonPropertyValue {
                    id,
                    base_value,
                    modifier,
                })
                .collect(),
        });
    }

    pub fn release(&mut self) {
        self.base_properties_index = Some(0);
        self.amount = 0;
        for property in &mut self.addon_properties {
            property.clear();
        }
        self.addon_properties.clear();
        self.description.clear();
    }

    pub fn clone_into(&self, target: &mut CGoods) -> Result<bool, GoodsCodecError> {
        let mut wire = Vec::new();
        let _ = self.serialize(&mut wire, true)?;
        let mut cursor = 0;
        target.unserialize(&wire, &mut cursor, true)
    }

    pub fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        let _ = self
            .shape_base
            .add_to_byte_array(destination, include_child);
        let base_properties_index = self
            .base_properties_index
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        destination.extend_from_slice(&base_properties_index.to_le_bytes());
        destination.extend_from_slice(&self.amount.to_le_bytes());
        destination.extend_from_slice(&self.price.to_le_bytes());
        append_goods_c_string(destination, &self.description);
        append_goods_count(
            destination,
            "m_vAddonProperties",
            self.addon_properties.len(),
        )?;
        for property in &self.addon_properties {
            property.serialize(destination)?;
        }
        Ok(true)
    }

    pub fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        self.release();
        let _ = self
            .shape_base
            .decord_from_byte_array(source, cursor, include_child)?;
        self.base_properties_index =
            Some(read_goods_u32(source, cursor, "m_dwBasePropertiesIndex")?);
        self.amount = read_goods_u32(source, cursor, "m_dwAmount")?;
        self.price = read_goods_u32(source, cursor, "m_dwPrice")?;
        self.description = read_goods_description(source, cursor)?;

        let count = read_goods_u32(source, cursor, "m_vAddonProperties count")?;
        for _ in 0..count {
            self.addon_properties
                .push(GoodsAddonProperty::unserialize(source, cursor)?);
        }
        Ok(true)
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        self.serialize(destination, include_child)
    }

    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        self.unserialize(source, cursor, include_child)
    }
}

impl GoodsAddonProperty {
    pub const fn property_type(&self) -> i32 {
        self.property_type
    }

    const fn with_constructor_defaults() -> Self {
        Self {
            property_type: 0,
            is_enabled: 0,
            is_implicit_attribute: 0,
            values: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.property_type = 0;
        self.is_enabled = 0;
        self.is_implicit_attribute = 0;
        for value in &mut self.values {
            value.clear();
        }
        self.values.clear();
    }

    fn serialize(&self, destination: &mut Vec<u8>) -> Result<(), GoodsCodecError> {
        destination.extend_from_slice(&self.property_type.to_le_bytes());
        destination.extend_from_slice(&self.is_enabled.to_le_bytes());
        destination.extend_from_slice(&self.is_implicit_attribute.to_le_bytes());
        append_goods_count(destination, "tagAddonProperty.vValues", self.values.len())?;
        for value in &self.values {
            destination.extend_from_slice(&value.id.to_le_bytes());
            destination.extend_from_slice(&value.base_value.to_le_bytes());
            destination.extend_from_slice(&value.modifier.to_le_bytes());
        }
        Ok(())
    }

    fn unserialize(source: &[u8], cursor: &mut usize) -> Result<Self, GoodsCodecError> {
        let mut property = Self::with_constructor_defaults();
        property.property_type = read_goods_i32(source, cursor, "tagAddonProperty.gapType")?;
        property.is_enabled = read_goods_i32(source, cursor, "tagAddonProperty.bIsEnabled")?;
        property.is_implicit_attribute =
            read_goods_i32(source, cursor, "tagAddonProperty.bIsImplicitAttribute")?;
        let count = read_goods_u32(source, cursor, "tagAddonProperty.vValues count")?;
        for _ in 0..count {
            let mut value = GoodsAddonPropertyValue::with_constructor_defaults();
            value.id = read_goods_u32(source, cursor, "tagAddonPropertyValue.dwId")?;
            value.base_value = read_goods_i32(source, cursor, "tagAddonPropertyValue.lBaseValue")?;
            value.modifier = read_goods_i32(source, cursor, "tagAddonPropertyValue.lModifier")?;
            property.values.push(value);
        }
        Ok(property)
    }
}

impl GoodsAddonPropertyValue {
    const fn with_constructor_defaults() -> Self {
        Self {
            id: 0,
            base_value: 0,
            modifier: 0,
        }
    }

    const fn clear(&mut self) {
        self.id = 0;
        self.base_value = 0;
        self.modifier = 0;
    }
}

fn append_goods_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    let visible = value
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(value.len());
    destination.extend_from_slice(&value[..visible]);
    destination.push(0);
}

fn append_goods_count(
    destination: &mut Vec<u8>,
    field: &'static str,
    count: usize,
) -> Result<(), GoodsCodecError> {
    let count = u32::try_from(count)
        .map_err(|_| GoodsCodecError::CollectionLengthOutsideLegacyRange { field, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn read_goods_description(source: &[u8], cursor: &mut usize) -> Result<Vec<u8>, GoodsCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let searchable = available.min(0x404);
    let Some(bytes) = source.get(offset..offset.saturating_add(searchable)) else {
        return Err(GoodsCodecError::UnexpectedEnd {
            field: "m_strDescribe",
            offset,
            needed: 1,
            available,
        });
    };
    if let Some(length) = bytes.iter().position(|&byte| byte == 0) {
        *cursor = offset + length + 1;
        return Ok(bytes[..length].to_vec());
    }

    // `_GetStringFromByteArray` не получал capacity и
    // продолжал запись за stack buffer либо чтение за source.
    Err(GoodsCodecError::UnterminatedDescription { offset, available })
}

fn read_goods_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, GoodsCodecError> {
    Ok(i32::from_le_bytes(read_goods_array(source, cursor, field)?))
}

fn read_goods_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, GoodsCodecError> {
    Ok(u32::from_le_bytes(read_goods_array(source, cursor, field)?))
}

fn read_goods_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], GoodsCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(N) else {
        return Err(GoodsCodecError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        // Старый вспомогательный код не получал длину источника. При коротком
        // буфере он выходил за его границы; Rust не воспроизводит это UB.
        return Err(GoodsCodecError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    *cursor = end;
    Ok(bytes
        .try_into()
        .expect("slice содержит ровно запрошенное число байт"))
}
