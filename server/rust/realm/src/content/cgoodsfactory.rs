//! Фабрика `CGoodsFactory` из `cgoodsfactory.cpp/.h`, подтверждённая
//! `worldserver.exe` и `worldserver.pdb`.
//! Перенесена в Realm `content/`.
//!
//! Реестры по ID и original-name сохраняют числовой/byte-exact порядок и
//! допускают отсутствующее значение properties. Load очищает их до разбора;
//! обрезанный файл оставляет фабрику пустой. Стандартный reader и `BTreeMap`
//! заменяют `CRFile`, MSVC tree и ручное владение.
//!
//! `UnserializeGoods` всегда декодирует child-data и оставляет объект только
//! при существующих base-properties. `CreateGoods` сохраняет число и порядок
//! всех вызовов legacy `random(bound)`; сам генератор передаётся callback-ом.
//!
//! Gold/YuanBao/JiFen indices разрешаются через исходные StringTable имена.
//! Public equipment upgrade этой версии всегда запрещён `CanUpgraded == 0`;
//! недостижимое mutation-тело не включается в поведение фабрики.

use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::path::Path;

use crate::content::cgoods::{CGoods, FirstAddonModifierAdjustment, GoodsCodecError};
use crate::content::goods::{
    CGoodsBaseProperties, GoodsBasePropertiesCodecError, GoodsBasePropertiesRegistry,
    ICON_TYPE_CONTAINER, ICON_TYPE_EQUIPPED, ICON_TYPE_GROUND,
};

pub type GoodsOriginalNameIndex = BTreeMap<Vec<u8>, u32>;

pub type GoodsNameIndex = BTreeMap<Vec<u8>, u32>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoodsUpgradeBlock {
    DestinationAddonHasNoValues { property_type: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoodsRegistryLoadError {
    InvalidHeader,
    UnexpectedEnd {
        offset: usize,
        requested: usize,
        remaining: usize,
    },
}

#[derive(Debug)]
pub enum GoodsRegistryFileLoadError {
    Io(std::io::Error),
    Format(GoodsRegistryLoadError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoodsRegistrySerializeError {
    CollectionLengthOutsideLegacyRange { count: usize },
    MissingBaseProperties { goods_id: u32 },
    BaseProperties(GoodsBasePropertiesCodecError),
}

impl fmt::Display for GoodsRegistrySerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CollectionLengthOutsideLegacyRange { count } => write!(
                formatter,
                "registry содержит {count} записей вне 32-битного legacy-диапазона"
            ),
            Self::MissingBaseProperties { goods_id } => {
                write!(
                    formatter,
                    "registry goods id {goods_id} содержит null properties"
                )
            }
            Self::BaseProperties(error) => error.fmt(formatter),
        }
    }
}

impl Error for GoodsRegistrySerializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BaseProperties(error) => Some(error),
            _ => None,
        }
    }
}

pub fn release_goods_registry(
    registry: &mut GoodsBasePropertiesRegistry,
    original_name_index: &mut GoodsOriginalNameIndex,
    name_index: &mut GoodsNameIndex,
) {
    registry.clear();
    original_name_index.clear();
    name_index.clear();
}

pub fn load_goods_registry_from_file<ResolveString>(
    path: impl AsRef<Path>,
    registry: &mut GoodsBasePropertiesRegistry,
    original_name_index: &mut GoodsOriginalNameIndex,
    name_index: &mut GoodsNameIndex,
    resolve_string_id: &mut ResolveString,
) -> Result<(), GoodsRegistryFileLoadError>
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    release_goods_registry(registry, original_name_index, name_index);
    let source = std::fs::read(path).map_err(GoodsRegistryFileLoadError::Io)?;
    load_goods_registry(
        &source,
        registry,
        original_name_index,
        name_index,
        resolve_string_id,
    )
    .map_err(GoodsRegistryFileLoadError::Format)
}

pub fn load_goods_registry<ResolveString>(
    source: &[u8],
    registry: &mut GoodsBasePropertiesRegistry,
    original_name_index: &mut GoodsOriginalNameIndex,
    name_index: &mut GoodsNameIndex,
    resolve_string_id: &mut ResolveString,
) -> Result<(), GoodsRegistryLoadError>
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    release_goods_registry(registry, original_name_index, name_index);

    let mut reader = GoodsConfigReader::new(source);
    if reader.read_exact(5)? != b"GOODS" {
        return Err(GoodsRegistryLoadError::InvalidHeader);
    }
    let _version = reader.read_u32()?;
    let goods_count = reader.read_i32()?;

    let mut loaded_registry = GoodsBasePropertiesRegistry::new();
    let mut loaded_original_name_index = GoodsOriginalNameIndex::new();
    let mut loaded_name_index = GoodsNameIndex::new();
    for _ in 0..goods_count.max(0) {
        let goods_id = reader.read_u32()?;
        let properties = load_base_properties(&mut reader, resolve_string_id)?;
        let original_name = properties.get_original_name().to_vec();
        let name = properties.get_name().to_vec();

        loaded_registry.insert(goods_id, Some(properties));
        loaded_original_name_index.insert(original_name, goods_id);
        loaded_name_index.insert(name, goods_id);
    }

    *registry = loaded_registry;
    *original_name_index = loaded_original_name_index;
    *name_index = loaded_name_index;
    Ok(())
}

pub fn serialize_goods_registry(
    registry: &GoodsBasePropertiesRegistry,
    destination: &mut Vec<u8>,
) -> Result<(), GoodsRegistrySerializeError> {
    let count = u32::try_from(registry.len()).map_err(|_| {
        GoodsRegistrySerializeError::CollectionLengthOutsideLegacyRange {
            count: registry.len(),
        }
    })?;
    destination.extend_from_slice(&count.to_le_bytes());
    for (&goods_id, properties) in registry {
        let properties = properties
            .as_ref()
            .ok_or(GoodsRegistrySerializeError::MissingBaseProperties { goods_id })?;
        destination.extend_from_slice(&goods_id.to_le_bytes());
        properties
            .serialize(destination)
            .map_err(GoodsRegistrySerializeError::BaseProperties)?;
    }
    Ok(())
}

/// Создаёт private `CGoodsFactory::Upgrade`.
///
/// `random` вызывается ровно тогда, когда source upper-value положительно,
/// с signed wrapping `upper - lower`; callback несёт уже действующую
/// legacy-семантику `random(bound)`. Non-zero `increase` исходника выражен
/// bool-границей. Пустой destination value-vector был оригинал dereference и
/// становится typed block без mutation.
pub fn upgrade_goods_addon<Random>(
    goods: &mut CGoods,
    source_property_type: i32,
    destination_property_type: i32,
    increase: bool,
    random: &mut Random,
) -> Result<bool, GoodsUpgradeBlock>
where
    Random: FnMut(i32) -> i32,
{
    let lower = goods.get_addon_property_value(source_property_type, 1);
    if lower < 1 {
        return Ok(false);
    }
    let upper = goods.get_addon_property_value(source_property_type, 2);
    let delta = if upper > 0 {
        lower.wrapping_add(random(upper.wrapping_sub(lower)))
    } else {
        lower
    };

    match goods.adjust_first_addon_modifier(destination_property_type, delta, increase) {
        FirstAddonModifierAdjustment::MissingProperty => Ok(false),
        FirstAddonModifierAdjustment::MissingValue => {
            Err(GoodsUpgradeBlock::DestinationAddonHasNoValues {
                property_type: destination_property_type,
            })
        }
        FirstAddonModifierAdjustment::Adjusted => Ok(true),
    }
}

fn load_base_properties<ResolveString>(
    reader: &mut GoodsConfigReader<'_>,
    resolve_string_id: &mut ResolveString,
) -> Result<CGoodsBaseProperties, GoodsRegistryLoadError>
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    let mut properties = CGoodsBaseProperties::with_constructor_defaults();
    let original_name = reader.read_legacy_string()?;
    let name_id = reader.read_legacy_string()?;
    let name = resolve_legacy_string(resolve_string_id, &name_id);
    let _ignored_byte = reader.read_u8()?;
    let price = reader.read_u32()?;
    let raw_type = reader.read_u32()?;
    let weight = reader.read_u32()?;
    properties.set_loaded_scalars(price, weight, raw_type);

    for icon_type in [ICON_TYPE_CONTAINER, ICON_TYPE_GROUND, ICON_TYPE_EQUIPPED] {
        properties.push_loaded_icon(icon_type, reader.read_u32()?);
    }

    let _ignored_dword_1 = reader.read_u32()?;
    let _ignored_dword_2 = reader.read_u32()?;
    let _ignored_dword_3 = reader.read_u32()?;
    let _ignored_byte = reader.read_u8()?;
    let _ignored_dword_4 = reader.read_u32()?;
    let description_id = reader.read_legacy_string()?;
    let description = resolve_legacy_string(resolve_string_id, &description_id);
    properties.set_loaded_names(original_name, name, description);

    let addon_count = reader.read_i32()?;
    for _ in 0..addon_count.max(0) {
        let property_type = reader.read_u16()?;
        let is_enabled = reader.read_u8()? != 0;
        let is_implicit_attribute = reader.read_u8()? != 0;
        let first_base_value = reader.read_i32()?;
        let second_base_value = reader.read_i32()?;
        let occur_probability = reader.read_u16()?;
        properties.push_loaded_addon_property(
            property_type,
            is_enabled,
            is_implicit_attribute,
            first_base_value,
            second_base_value,
            occur_probability,
        );
    }

    for property_index in 0..addon_count.max(0) as usize {
        let modifier_count = reader.read_i32()?;
        for _ in 0..modifier_count.max(0) {
            let raw = reader.read_exact(16)?;
            let value_id = u32::from(raw[0]) + 1;
            let lower_limit = i32::from_le_bytes(raw[4..8].try_into().unwrap());
            let upper_limit = i32::from_le_bytes(raw[8..12].try_into().unwrap());
            let probability = u16::from_le_bytes(raw[12..14].try_into().unwrap());
            let _ = properties.push_loaded_modifier(
                property_index,
                value_id,
                lower_limit,
                upper_limit,
                probability,
            );
        }
    }
    Ok(properties)
}

fn resolve_legacy_string<ResolveString>(
    resolve_string_id: &mut ResolveString,
    string_id: &[u8],
) -> Vec<u8>
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    let resolved = resolve_string_id(string_id).unwrap_or_default();
    truncate_at_nul(&resolved).to_vec()
}

fn truncate_at_nul(value: &[u8]) -> &[u8] {
    let length = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..length]
}

struct GoodsConfigReader<'source> {
    source: &'source [u8],
    cursor: usize,
}

impl<'source> GoodsConfigReader<'source> {
    const fn new(source: &'source [u8]) -> Self {
        Self { source, cursor: 0 }
    }

    fn read_exact(&mut self, requested: usize) -> Result<&'source [u8], GoodsRegistryLoadError> {
        let remaining = self.source.len().saturating_sub(self.cursor);
        let Some(end) = self.cursor.checked_add(requested) else {
            return Err(GoodsRegistryLoadError::UnexpectedEnd {
                offset: self.cursor,
                requested,
                remaining,
            });
        };
        let Some(value) = self.source.get(self.cursor..end) else {
            return Err(GoodsRegistryLoadError::UnexpectedEnd {
                offset: self.cursor,
                requested,
                remaining,
            });
        };
        self.cursor = end;
        Ok(value)
    }

    fn read_u8(&mut self) -> Result<u8, GoodsRegistryLoadError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> Result<u16, GoodsRegistryLoadError> {
        Ok(u16::from_le_bytes(self.read_exact(2)?.try_into().unwrap()))
    }

    fn read_u32(&mut self) -> Result<u32, GoodsRegistryLoadError> {
        Ok(u32::from_le_bytes(self.read_exact(4)?.try_into().unwrap()))
    }

    fn read_i32(&mut self) -> Result<i32, GoodsRegistryLoadError> {
        Ok(i32::from_le_bytes(self.read_exact(4)?.try_into().unwrap()))
    }

    fn read_legacy_string(&mut self) -> Result<Vec<u8>, GoodsRegistryLoadError> {
        let length = self.read_u32()? as usize;
        Ok(truncate_at_nul(self.read_exact(length)?).to_vec())
    }
}

pub fn query_goods_base_properties(
    registry: &GoodsBasePropertiesRegistry,
    index: u32,
) -> Option<&CGoodsBaseProperties> {
    registry.get(&index).and_then(Option::as_ref)
}

pub fn query_goods_name(
    registry: &GoodsBasePropertiesRegistry,
    index: u32,
) -> Option<&[u8]> {
    query_goods_base_properties(registry, index).map(CGoodsBaseProperties::get_name)
}

pub fn query_goods_id_by_original_name(
    index: &GoodsOriginalNameIndex,
    original_name: Option<&CStr>,
) -> u32 {
    query_goods_id_by_original_name_bytes(index, original_name.map(CStr::to_bytes))
}

pub fn query_goods_base_properties_by_original_name<'registry>(
    registry: &'registry GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    original_name: Option<&CStr>,
) -> Option<&'registry CGoodsBaseProperties> {
    let goods_id = query_goods_id_by_original_name(original_name_index, original_name);
    query_goods_base_properties(registry, goods_id)
}

pub fn garbage_collect(goods: Option<&mut Option<Box<CGoods>>>) -> bool {
    let Some(goods) = goods else {
        return false;
    };
    let _ = goods.take();
    true
}

pub fn upgrade_equipment(goods: Option<&mut CGoods>, target_level: i32) -> bool {
    let Some(goods) = goods else {
        return false;
    };
    let _ = target_level;
    if !goods.can_upgraded() {
        return false;
    }
    false
}

pub fn get_gold_coin_index<ResolveString>(
    original_name_index: &GoodsOriginalNameIndex,
    resolve_string_id: &mut ResolveString,
) -> u32
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    resolve_special_goods_index(original_name_index, b"WS0108", resolve_string_id)
}

pub fn get_yuan_bao_index<ResolveString>(
    original_name_index: &GoodsOriginalNameIndex,
    resolve_string_id: &mut ResolveString,
) -> u32
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    resolve_special_goods_index(original_name_index, b"WS0109", resolve_string_id)
}

pub fn get_ji_fen_index<ResolveString>(
    original_name_index: &GoodsOriginalNameIndex,
    resolve_string_id: &mut ResolveString,
) -> u32
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    resolve_special_goods_index(original_name_index, b"WS0110", resolve_string_id)
}

fn resolve_special_goods_index<ResolveString>(
    original_name_index: &GoodsOriginalNameIndex,
    string_id: &[u8],
    resolve_string_id: &mut ResolveString,
) -> u32
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    let original_name = resolve_legacy_string(resolve_string_id, string_id);
    query_goods_id_by_original_name_bytes(original_name_index, Some(&original_name))
}

pub fn query_goods_id_by_original_name_bytes(
    index: &GoodsOriginalNameIndex,
    original_name: Option<&[u8]>,
) -> u32 {
    original_name
        .map(|name| {
            let visible = name.iter().position(|byte| *byte == 0).unwrap_or(name.len());
            &name[..visible]
        })
        .and_then(|name| index.get(name).copied())
        .unwrap_or(0)
}

pub fn unserialize_goods(
    source: &[u8],
    cursor: &mut usize,
    registry: &GoodsBasePropertiesRegistry,
) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
    let mut goods = Box::new(CGoods::with_constructor_base_and_type());
    let _ = goods.unserialize(source, cursor, true)?;
    let index = goods
        .get_base_properties_index()
        .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
    if query_goods_base_properties(registry, index).is_none() {
        return Ok(None);
    }
    Ok(Some(goods))
}

pub fn create_goods<Random>(
    registry: &GoodsBasePropertiesRegistry,
    index: u32,
    random: &mut Random,
) -> Option<Box<CGoods>>
where
    Random: FnMut(i32) -> i32 + ?Sized,
{
    let properties = query_goods_base_properties(registry, index)?;
    let mut goods = create_goods_base(index, properties);

    for property_type in properties.valid_addon_property_types() {
        if random(10_000) as u32 >= properties.get_occur_probability(property_type) {
            continue;
        }

        let mut values = Vec::new();
        for source in properties.get_addon_property_values(property_type) {
            let mut rolled_modifier = 0;
            if source.is_modifier_enabled() {
                let roll = random(10_000);
                let mut accumulated = 0i32;
                for modifier in source.modifiers() {
                    if (roll.wrapping_sub(accumulated) as u32) < modifier.probability() {
                        let range = modifier.upper_limit().wrapping_sub(modifier.lower_limit());
                        rolled_modifier = random(range).wrapping_add(modifier.lower_limit());
                        break;
                    }
                    accumulated = accumulated.wrapping_add(modifier.probability() as i32);
                }
            }
            values.push((source.id(), source.base_value(), rolled_modifier));
        }

        goods.push_factory_addon_property(
            property_type,
            properties.is_implicit(property_type),
            values,
        );
    }

    Some(goods)
}

pub fn create_goods_no_probability(
    registry: &GoodsBasePropertiesRegistry,
    index: u32,
) -> Option<Box<CGoods>> {
    let properties = query_goods_base_properties(registry, index)?;
    let mut goods = create_goods_base(index, properties);

    for property_type in properties.valid_addon_property_types() {
        if properties.get_occur_probability(property_type) != 10_000 {
            continue;
        }
        let values = properties
            .get_addon_property_values(property_type)
            .iter()
            .map(|source| (source.id(), source.base_value(), 0))
            .collect();
        goods.push_factory_addon_property(
            property_type,
            properties.is_implicit(property_type),
            values,
        );
    }

    Some(goods)
}

fn create_goods_base(index: u32, properties: &CGoodsBaseProperties) -> Box<CGoods> {
    let mut goods = Box::new(CGoods::with_constructor_base_and_type());
    goods.set_base_properties_index(index);
    goods.set_name(properties.get_name());
    goods.set_goods_description(properties.get_description());
    goods.set_price(properties.get_price());
    goods.set_graphics_id(properties.get_icon_id(ICON_TYPE_GROUND) as i32);
    goods
}
