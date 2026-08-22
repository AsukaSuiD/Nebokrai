//! Фабрика товаров исторического `WorldServer`.
//!
//! Статус `GarbageCollect` RVA `0x00055C20`,
//! `QueryGoodsBaseProperties/QueryGoodsName` RVA
//! `0x00055DB0/0x00055DE0`,
//! `UnserializeGoods` RVA `0x00055E20` и `QueryGoodsIDByOriginalName` RVA
//! `0x000566F0`, `QueryGoodsBasePropertiesByOriginalName` RVA `0x00057390`,
//! `GetGoldCoinIndex/GetYuanBaoIndex/GetJiFenIndex` RVA
//! `0x00056810/0x000568B0/0x00056950`,
//! private `Upgrade` и сломанный `UpgradeEquipment` RVA
//! `0x00055F20/0x000561C0`,
//! `CreateGoods/CreateGoodsNoProbability` RVA
//! `0x00059460/0x000597C0`, `Release/Load` RVA
//! `0x00058380/0x00059EE0`, `Serialize` RVA `0x00056130` — `IMPLEMENTED`;
//! остальной корпус ниже остаётся
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:38,88,696`.
//!
//! Старые static `std::map<unsigned long, CGoodsBaseProperties*>` и
//! `std::map<std::string, unsigned long>` заменены caller-owned `BTreeMap`:
//! unsigned key-order и возможность null mapped-value первой карты сохранены,
//! а глобальный mutable pointer-lifetime не вводится до восстановления
//! `Load/Release` фабрики. Original-name остаётся последовательностью legacy-
//! байтов, а не обязанной быть UTF-8 строкой; `CStr` сохраняет доказанную
//! границу нуль-терминированного `char const*`. `nullptr` результата base-
//! lookup выражен `Option`; успешный factory-result остаётся heap-owned
//! `Box<CGoods>`.
//!
//! `UnserializeGoods` создаёт default `CGoods`, вызывает его decoder, затем
//! оставляет объект только при non-null lookup его base-properties index.
//! Exact инструкции `0x00455E9A..0x00455EA6` подтверждают третий аргумент
//! virtual slot `+0xAC`: `push 1`, cursor, source, то есть `include_child=true`.
//! После ответа reverse прекращён. Вызванный затем `CShape::GetDir` не меняет
//! состояние и не влияет на решение; Rust не сохраняет этот пустой getter-call.
//! Нулевой source pointer исходно давал `nullptr`, а Rust API принимает
//! только non-null slice. Ошибки безопасного `CGoods` decoder-а остаются
//! typed-ошибками вместо старого безразмерного чтения.
//!
//! Для original-name lookup exact `0x004566F0..0x00456803` подтверждает:
//! null-вход возвращает `0`, отсутствующий key возвращает `0`, найденный узел
//! возвращает mapped `u32` по `+0x28`. Два временных `std::string`, tree node
//! и security-cookie являются библиотечной/компиляторной формой; Rust
//! выполняет тот же точный поиск непосредственно в `BTreeMap`.
//!
//! `CreateGoods` сохраняет порядок всех observable roll-ов: один
//! `random(10000)` на каждый enabled addon-type, затем ещё один на modifier и
//! `random(upper-lower)` только для выбранного probability-interval. Сам
//! process-global PRNG не подменяется другим алгоритмом: caller передаёт узкий
//! callback с exact legacy `random(bound)` семантикой. Rust `Box/Vec` заменяют
//! только allocation/STL plumbing и автоматически освобождают частичный result.
//!
//! `Load` открывает файл через `std::fs`, а parser принимает byte-slice и
//! сохраняет exact `GOODS`-формат, порядок
//! двух StringTable lookup-ов и построения трёх map-ов. Старые unchecked
//! `CRFile::ReadData`, ручные `new[]` и утечки при duplicate-id заменены
//! проверяемым reader-ом и Rust ownership. Для повреждённого/обрезанного файла
//! возвращается typed-ошибка, а registry остаётся очищенным, как в безопасном
//! donor-пути; валидный вход и его observable state не меняются.
//! `UpgradeEquipment` в matching EXE заблокирован exact всегда-нулевым
//! `CGoods::CanUpgraded`; поэтому исправленное mutation-тело Linux-donor-а не
//! является поведением этой версии. Private `Upgrade` материализован отдельным
//! callback-adapter-ом, но публичный контур по-прежнему не достигает его.

use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::path::Path;

use super::cgoods::{CGoods, FirstAddonModifierAdjustment, GoodsCodecError};
use super::cgoodsbaseproperties::{
    CGoodsBaseProperties, GoodsBasePropertiesCodecError, ICON_TYPE_CONTAINER, ICON_TYPE_EQUIPPED,
    ICON_TYPE_GROUND,
};

/// Достигнутая lookup-форма static base-properties map.
pub(crate) type GoodsBasePropertiesRegistry = BTreeMap<u32, Option<CGoodsBaseProperties>>;

/// Достигнутый индекс exact legacy original-name в unsigned goods id.
pub(crate) type GoodsOriginalNameIndex = BTreeMap<Vec<u8>, u32>;

/// Достигнутый индекс exact legacy localized-name в unsigned goods id.
pub(crate) type GoodsNameIndex = BTreeMap<Vec<u8>, u32>;

/// Safe-граница private `CGoodsFactory::Upgrade` для malformed destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsUpgradeBlock {
    DestinationAddonHasNoValues { property_type: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsRegistryLoadError {
    InvalidHeader,
    UnexpectedEnd {
        offset: usize,
        requested: usize,
        remaining: usize,
    },
}

#[derive(Debug)]
pub(crate) enum GoodsRegistryFileLoadError {
    Io(std::io::Error),
    Format(GoodsRegistryLoadError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsRegistrySerializeError {
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

/// Очищает три owner-map в exact исходном порядке.
pub(crate) fn release_goods_registry(
    registry: &mut GoodsBasePropertiesRegistry,
    original_name_index: &mut GoodsOriginalNameIndex,
    name_index: &mut GoodsNameIndex,
) {
    registry.clear();
    original_name_index.clear();
    name_index.clear();
}

/// Открывает config стандартной библиотекой и передаёт exact format parser-у.
pub(crate) fn load_goods_registry_from_file<ResolveString>(
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

/// Загружает exact `GOODS`-поток и разрешает два legacy StringTable id записи.
pub(crate) fn load_goods_registry<ResolveString>(
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

/// Кодирует registry в exact ascending-id wire фабрики.
pub(crate) fn serialize_goods_registry(
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

/// Материализует private `CGoodsFactory::Upgrade`.
///
/// `random` вызывается ровно тогда, когда source upper-value положительно,
/// с exact signed wrapping `upper - lower`; callback несёт уже достигнутую
/// legacy-семантику `random(bound)`. Non-zero `increase` исходника выражен
/// bool-границей. Пустой destination value-vector был raw dereference и
/// становится typed block без mutation.
pub(crate) fn upgrade_goods_addon<Random>(
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

/// Возвращает non-null base-properties для точного unsigned index.
pub(crate) fn query_goods_base_properties(
    registry: &GoodsBasePropertiesRegistry,
    index: u32,
) -> Option<&CGoodsBaseProperties> {
    registry.get(&index).and_then(Option::as_ref)
}

/// Возвращает byte-exact localized-name известного non-null товара.
pub(crate) fn query_goods_name(
    registry: &GoodsBasePropertiesRegistry,
    index: u32,
) -> Option<&[u8]> {
    query_goods_base_properties(registry, index).map(CGoodsBaseProperties::get_name)
}

/// Возвращает goods id по точному legacy original-name или исходный `0`.
pub(crate) fn query_goods_id_by_original_name(
    index: &GoodsOriginalNameIndex,
    original_name: Option<&CStr>,
) -> u32 {
    query_goods_id_by_original_name_bytes(index, original_name.map(CStr::to_bytes))
}

/// Сохраняет исходную двухступенчатую семантику: missing name сначала даёт id 0.
pub(crate) fn query_goods_base_properties_by_original_name<'registry>(
    registry: &'registry GoodsBasePropertiesRegistry,
    original_name_index: &GoodsOriginalNameIndex,
    original_name: Option<&CStr>,
) -> Option<&'registry CGoodsBaseProperties> {
    let goods_id = query_goods_id_by_original_name(original_name_index, original_name);
    query_goods_base_properties(registry, goods_id)
}

/// Освобождает optional heap-owner; существующий slot возвращает success даже пустым.
pub(crate) fn garbage_collect(goods: Option<&mut Option<Box<CGoods>>>) -> bool {
    let Some(goods) = goods else {
        return false;
    };
    let _ = goods.take();
    true
}

/// Сохраняет exact ранний отказ matching EXE без мутаций и RNG-вызовов.
pub(crate) fn upgrade_equipment(goods: Option<&mut CGoods>, target_level: i32) -> bool {
    let Some(goods) = goods else {
        return false;
    };
    let _ = target_level;
    if !goods.can_upgraded() {
        return false;
    }
    false
}

pub(crate) fn get_gold_coin_index<ResolveString>(
    original_name_index: &GoodsOriginalNameIndex,
    resolve_string_id: &mut ResolveString,
) -> u32
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    resolve_special_goods_index(original_name_index, b"WS0108", resolve_string_id)
}

pub(crate) fn get_yuan_bao_index<ResolveString>(
    original_name_index: &GoodsOriginalNameIndex,
    resolve_string_id: &mut ResolveString,
) -> u32
where
    ResolveString: FnMut(&[u8]) -> Option<Vec<u8>>,
{
    resolve_special_goods_index(original_name_index, b"WS0109", resolve_string_id)
}

pub(crate) fn get_ji_fen_index<ResolveString>(
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

pub(crate) fn query_goods_id_by_original_name_bytes(
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

/// Декодирует heap-owned товар и отбрасывает неизвестный base-properties index.
pub(crate) fn unserialize_goods(
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

/// Создаёт товар и выполняет exact addon probability/modifier roll-order.
pub(crate) fn create_goods<Random>(
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

/// Создаёт только гарантированные addon-ы без probability/modifier roll-ов.
pub(crate) fn create_goods_no_probability(
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp

// ============================================================================
// FUNCTION: CGoodsFactory::GarbageCollect
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:681
// RVA: 0x00055C20
// ADDRESS: 00455c20
// PROTOTYPE: int __cdecl GarbageCollect(CGoods * * param_1)
//
// IMPLEMENTED выше как `garbage_collect`; внешний null slot даёт `false`,
// существующий slot всегда `true` и после вызова пуст, Rust `Drop` заменяет vcall.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsBaseProperties
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:38
// RVA: 0x00055DB0
// ADDRESS: 00455db0
// PROTOTYPE: CGoodsBaseProperties * __cdecl QueryGoodsBaseProperties(ulong param_1)
//
// IMPLEMENTED выше; отсутствие key и null mapped-value дают один `None`.

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsName
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:76
// RVA: 0x00055DE0
// ADDRESS: 00455de0
// PROTOTYPE: char * __cdecl QueryGoodsName(ulong param_1)
//
// IMPLEMENTED выше как `query_goods_name`; lookup miss и null mapped-value
// остаются одним `None`, successful result заимствует bytes owner-записи.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::UnserializeGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:696
// RVA: 0x00055E20
// ADDRESS: 00455e20
// PROTOTYPE: CGoods * __cdecl UnserializeGoods(uchar * param_1, long * param_2)
//
// IMPLEMENTED выше; exact `push 1` закрывает include-child call-site.

// ============================================================================
// FUNCTION: CGoodsFactory::Upgrade
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:859
// RVA: 0x00055F20
// ADDRESS: 00455f20
// PROTOTYPE: int __cdecl Upgrade(CGoods * param_1, GOODS_ADDON_PROPERTIES param_2, GOODS_ADDON_PROPERTIES param_3, int param_4)
//
// VERIFIED_DISASSEMBLY: exact `0x00455F20..0x0045603F` вызывает `random` при
// любом `upper > 0`, передавая signed wrapping `upper - lower`, в отличие от
// Linux-donor clamp. Реализовано `upgrade_goods_addon`; null goods не входит в
// typed Rust API, а пустой destination value-vector становится safe block
// вместо raw dereference. `UpgradeEquipment` этой EXE всё ещё не достигает
// helper из-за exact `CanUpgraded == 0`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::Serialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:122
// RVA: 0x00056130
// ADDRESS: 00456130
// PROTOTYPE: int __cdecl Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// IMPLEMENTED выше как `serialize_goods_registry`; `BTreeMap` сохраняет exact
// ascending unsigned id-order, а null legacy pointer становится typed-ошибкой.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::UpgradeEquipment
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:731
// RVA: 0x000561C0
// ADDRESS: 004561c0
// PROTOTYPE: int __cdecl UpgradeEquipment(CGoods * param_1, long param_2)
//
// IMPLEMENTED выше; exact вызов `CanUpgraded` по `0x004561D3` всегда получает
// `0`, поэтому переход `0x004561DC -> 0x00456553` исключает mutation/RNG body.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsIDByOriginalName
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:88
// RVA: 0x000566F0
// ADDRESS: 004566f0
// PROTOTYPE: ulong __cdecl QueryGoodsIDByOriginalName(char * param_1)
//
// IMPLEMENTED выше; exact ASM закрывает ошибочно потерянный Ghidra return.

// ============================================================================
// FUNCTION: CGoodsFactory::GetGoldCoinIndex
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:362
// RVA: 0x00056810
// ADDRESS: 00456810
// PROTOTYPE: ulong __cdecl GetGoldCoinIndex(void)
//
// IMPLEMENTED выше через exact StringTable id `WS0108`; Linux-donor fallback
// `MONEY` отсутствует в EXE и намеренно не перенесён.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::GetYuanBaoIndex
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:370
// RVA: 0x000568B0
// ADDRESS: 004568b0
// PROTOTYPE: ulong __cdecl GetYuanBaoIndex(void)
//
// IMPLEMENTED выше через exact StringTable id `WS0109` без donor fallback.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::GetJiFenIndex
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:379
// RVA: 0x00056950
// ADDRESS: 00456950
// PROTOTYPE: ulong __cdecl GetJiFenIndex(void)
//
// IMPLEMENTED выше через exact StringTable id `WS0110` без donor fallback.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsBasePropertiesByOriginalName
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:50
// RVA: 0x00057390
// ADDRESS: 00457390
// PROTOTYPE: CGoodsBaseProperties * __cdecl QueryGoodsBasePropertiesByOriginalName(char * param_1)
//
// IMPLEMENTED выше как `query_goods_base_properties_by_original_name`; важная
// двухступенчатая семантика id `0` при отсутствующем имени сохранена буквально.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:312
// RVA: 0x00058380
// ADDRESS: 00458380
// PROTOTYPE: void __cdecl Release(void)
//
// IMPLEMENTED выше как `release_goods_registry`; три `BTreeMap::clear`
// сохраняют exact порядок очистки, а Rust `Drop` заменяет ручной delete/STL.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CreateGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:165
// RVA: 0x00059460
// ADDRESS: 00459460
// PROTOTYPE: CGoods * __cdecl CreateGoods(ulong param_1)
//
// IMPLEMENTED выше; registry и legacy-random передаются явно, allocation/STL/EH
// заменены `Box/Vec`, порядок lookup и всех roll-ов сохранён.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CreateGoodsNoProbability
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:243
// RVA: 0x000597C0
// ADDRESS: 004597c0
// PROTOTYPE: CGoods * __cdecl CreateGoodsNoProbability(ulong param_1)
//
// IMPLEMENTED выше; выбираются только enabled типы с probability ровно `10000`,
// все modifier-ы остаются нулевыми, technical debug-log не материализуется.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::Load
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp:396
// RVA: 0x00059EE0
// ADDRESS: 00459ee0
// PROTOTYPE: int __cdecl Load(char * param_1)
//
// IMPLEMENTED выше как `load_goods_registry`; exact ASM `0x00459EE0..0045A7B9`
// подтверждает layout, signed loop-counts, type/equip mapping, resolver-order и
// три финальных map insert-а. `std::fs` заменяет только CRFile/debug plumbing.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004dc3a5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x000DC3A5
// ADDRESS: 004dc3a5
// PROTOTYPE: undefined Catch@004dc3a5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004dc486
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x000DC486
// ADDRESS: 004dc486
// PROTOTYPE: undefined Catch@004dc486()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004dc56c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x000DC56C
// ADDRESS: 004dc56c
// PROTOTYPE: undefined Catch@004dc56c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00535870
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x00135870
// ADDRESS: 00535870
// PROTOTYPE: undefined Unwind@00535870()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0053587b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsfactory.cpp
// RVA: 0x0013587B
// ADDRESS: 0053587b
// PROTOTYPE: undefined Unwind@0053587b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
