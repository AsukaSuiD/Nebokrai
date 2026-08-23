//! Владелец базовых свойств товаров исторического `WorldServer`.
//!
//! Конструктор RVA `0x000D5010`, `Serialize` RVA `0x000D4BD0`,
//! `tagAddonProperty::Serialize` RVA `0x000D4B20`,
//! `GetPrice/GetWeight/GetName/GetDescribe/GetIconID` RVA
//! `0x000D4930/0x000D4940/0x000D4960/0x000D4970/0x000D4980`,
//! `GetGoodsType/GetEquipPlace` RVA `0x000DEA40/0x000DEA50`,
//! `GetAddonPropertyValues/GetValidAddonProperties` RVA
//! `0x000D4E50/0x000D4D90`, `GetOccurProbability/IsImplicit` RVA
//! `0x000D49C0/0x000D4A10`, lifecycle addon-ов RVA
//! `0x000D4D70/0x000D4EE0/0x000D4F00/0x000D4F90` и destructor RVA
//! `0x000D5080` реализованы в Rust. Точная пара доказательных артефактов:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:285`;
//! inline getter скомпонован линкером по одному RVA с равными scalar-getter-ами.
//!
//! Exact PDB задаёт `GOODS_TYPE` как signed `int`: `GT_USELESS = 0`,
//! `GT_CONSUMABLE = 1`, `GT_EQUIPMENT = 2`, а соседний signed
//! `EQUIP_PLACE m_epEquipPlace` лежит по `+0x60`. Его значения `0..16`
//! буквально соответствуют `EP_UNKNOWN..EP_LINGBAO`. Он же подтверждает
//! unsigned `m_dwWeight` по `+0x3C`; exact getter состоит из одной загрузки.
//! PDB также подтверждает
//! `GAP_PARTICULAR_ATTRIBUTE = 0x0D`, `GAP_GOODS_STACKING_LIMIT = 0x26`,
//! `GAP_WEAPON_LEVEL = 0x30` и
//! layout `tagAddonPropertyValue`: unsigned `dwId` по `+0`, signed
//! `lBaseValue` по `+4`, modifier-флаг по `+8`, затем vector modifier-ов.
//! Rust-owner хранит все поля записи, заполняемые exact
//! `CGoodsFactory::Load`: original/localized name, описание, цену, вес, тип,
//! equip-place, три icon-а и addon-ы с modifier-ами. Неиспользуемые поля
//! входного формата фабрика только потребляет, как и оригинал.
//! Serialize намеренно не включает description: exact wire-owner пишет два
//! C-string имени, type/place/price/weight, icons и полное addon-дерево.
//!
//! `GetAddonPropertyValues` останавливается на первом property совпавшего типа
//! и копирует все его values. Заимствованный slice заменяет временную копию
//! только внутри синхронного read-only вызова `CGoods::GetMaxStackNumber`:
//! порядок, первое совпадение и значения сохраняются, а STL allocation/copy/
//! destruction не являются наблюдаемым контрактом.
//!
//! Exact ASM destructor-а опровергает ложные ранние `return` декомпилятора:
//! icons, addon-дерево и три строки освобождаются безусловно. В Rust тот же
//! lifecycle обеспечивает владение `Vec`; порядок внутренних освобождений не
//! наблюдаем, потому что у элементов нет внешних callback-ов.

use std::error::Error;
use std::fmt;

pub(crate) const GOODS_TYPE_USELESS: i32 = 0;
pub(crate) const GOODS_TYPE_CONSUMABLE: i32 = 1;
pub(crate) const GOODS_TYPE_EQUIPMENT: i32 = 2;
pub(crate) const GAP_PARTICULAR_ATTRIBUTE: i32 = 0x0d;
pub(crate) const GAP_GOODS_STACKING_LIMIT: i32 = 0x26;
pub(crate) const GAP_WEAPON_LEVEL: i32 = 0x30;
pub(crate) const ICON_TYPE_CONTAINER: i32 = 0;
pub(crate) const ICON_TYPE_GROUND: i32 = 1;
pub(crate) const ICON_TYPE_EQUIPPED: i32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsBasePropertiesCodecError {
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
pub(super) struct GoodsBaseAddonPropertyValueModifier {
    probability: u32,
    lower_limit: i32,
    upper_limit: i32,
}

/// Достигнутые scalar-поля исходного `tagAddonPropertyValue`.
#[derive(Clone)]
pub(crate) struct GoodsBaseAddonPropertyValue {
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

/// Достигнутая stacking-часть исходного `CGoodsBaseProperties`.
#[derive(Clone)]
pub(crate) struct CGoodsBaseProperties {
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

impl CGoodsBaseProperties {
    /// Создаёт exact пустое состояние constructor-а `0x004D5010`.
    pub(super) const fn with_constructor_defaults() -> Self {
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

    /// Заимствует byte-exact исходное имя без завершающего NUL.
    pub(crate) fn get_original_name(&self) -> &[u8] {
        &self.original_name
    }

    /// Заимствует byte-exact имя без завершающего NUL.
    pub(crate) fn get_name(&self) -> &[u8] {
        &self.name
    }

    /// Заимствует byte-exact описание без завершающего NUL.
    pub(crate) fn get_description(&self) -> &[u8] {
        &self.description
    }

    /// Возвращает exact unsigned базовую цену.
    pub(crate) const fn get_price(&self) -> u32 {
        self.price
    }

    /// Возвращает exact unsigned вес одной единицы товара.
    pub(crate) const fn get_weight(&self) -> u32 {
        self.weight
    }

    /// Возвращает exact signed `GOODS_TYPE` без дополнительных эффектов.
    pub(crate) const fn get_goods_type(&self) -> i32 {
        self.goods_type
    }

    /// Возвращает exact signed `EQUIP_PLACE` без дополнительных эффектов.
    pub(crate) const fn get_equip_place(&self) -> i32 {
        self.equip_place
    }

    /// Возвращает icon первого совпавшего numeric-типа либо исходный `0`.
    pub(crate) fn get_icon_id(&self, icon_type: i32) -> u32 {
        self.icons
            .iter()
            .find(|icon| icon.icon_type == icon_type)
            .map_or(0, |icon| icon.icon_id)
    }

    /// Обходит тип каждого enabled property в исходном vector-order.
    pub(super) fn valid_addon_property_types(&self) -> impl Iterator<Item = i32> + '_ {
        self.addon_properties
            .iter()
            .filter(|property| property.is_enabled == 1)
            .map(|property| property.property_type)
    }

    /// Возвращает implicit-флаг первого property совпавшего типа.
    pub(crate) fn is_implicit(&self, property_type: i32) -> i32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(0, |property| property.is_implicit_attribute)
    }

    /// Возвращает values первого property совпавшего numeric-типа.
    pub(crate) fn get_addon_property_values(
        &self,
        property_type: i32,
    ) -> &[GoodsBaseAddonPropertyValue] {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(&[], |property| property.values.as_slice())
    }

    /// Возвращает probability первого property совпавшего numeric-типа.
    pub(crate) fn get_occur_probability(&self, property_type: i32) -> u32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(0, |property| property.occur_probability)
    }

    /// Кодирует exact client-facing base-properties wire без description.
    pub(crate) fn serialize(
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

    pub(super) fn set_loaded_names(
        &mut self,
        original_name: Vec<u8>,
        name: Vec<u8>,
        description: Vec<u8>,
    ) {
        self.original_name = original_name;
        self.name = name;
        self.description = description;
    }

    pub(super) const fn set_loaded_scalars(&mut self, price: u32, weight: u32, raw_type: u32) {
        self.price = price;
        self.weight = weight;
        if raw_type == 1 {
            self.goods_type = GOODS_TYPE_CONSUMABLE;
        } else if raw_type >= 2 && raw_type <= 17 {
            self.goods_type = GOODS_TYPE_EQUIPMENT;
            self.equip_place = raw_type as i32 - 1;
        }
    }

    pub(super) fn push_loaded_icon(&mut self, icon_type: i32, icon_id: u32) {
        self.icons.push(GoodsBaseIcon { icon_type, icon_id });
    }

    pub(super) fn push_loaded_addon_property(
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

    /// Возвращает `false` только для неизвестного value-id, как original loop.
    pub(super) fn push_loaded_modifier(
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
    /// Создаёт exact пустое scalar-состояние с пустым vector modifier-ов.
    const fn with_constructor_defaults() -> Self {
        Self {
            id: 0,
            base_value: 0,
            is_modifier_enabled: 0,
            modifiers: Vec::new(),
        }
    }

    /// Восстанавливает scalar-ы и vector в исходное пустое состояние.
    fn clear(&mut self) {
        self.id = 0;
        self.base_value = 0;
        self.is_modifier_enabled = 0;
        self.modifiers.clear();
    }

    /// Возвращает exact unsigned идентификатор значения.
    pub(crate) const fn id(&self) -> u32 {
        self.id
    }

    /// Возвращает exact signed базовое значение.
    pub(crate) const fn base_value(&self) -> i32 {
        self.base_value
    }

    pub(super) const fn is_modifier_enabled(&self) -> bool {
        self.is_modifier_enabled != 0
    }

    pub(super) fn modifiers(&self) -> &[GoodsBaseAddonPropertyValueModifier] {
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
    pub(super) const fn probability(&self) -> u32 {
        self.probability
    }

    pub(super) const fn lower_limit(&self) -> i32 {
        self.lower_limit
    }

    pub(super) const fn upper_limit(&self) -> i32 {
        self.upper_limit
    }

    fn serialize(&self, destination: &mut Vec<u8>) {
        destination.extend_from_slice(&self.probability.to_le_bytes());
        destination.extend_from_slice(&self.lower_limit.to_le_bytes());
        destination.extend_from_slice(&self.upper_limit.to_le_bytes());
    }
}

impl GoodsBaseAddonProperty {
    /// Создаёт exact состояние constructor-а `0x004D4EE0`.
    const fn with_constructor_defaults() -> Self {
        Self {
            property_type: 0,
            is_enabled: 0,
            is_implicit_attribute: 0,
            occur_probability: 0,
            values: Vec::new(),
        }
    }

    /// Повторяет `Clear`: сначала scalar-ы, затем values в исходном порядке.
    fn clear(&mut self) {
        self.property_type = 0;
        self.is_enabled = 0;
        self.is_implicit_attribute = 0;
        self.occur_probability = 0;
        for value in &mut self.values {
            value.clear();
        }
        self.values.clear();
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
