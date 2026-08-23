//! Владелец товара исторического `WorldServer`.
//!
//! `CGoods::Serialize/Unserialize`,
//! byte-array wrappers, `Release`,
//! scalar `GetAddonPropertyValues`, `GetMaxStackNumber`
//! `GetWeight`,
//! `GetAllAddonProperties/GetGoodsName/IsAddonProperyExist`
//! vector `GetAddonPropertyValues`
//! wire-based `Clone`,
//! `SetExID`, base-подобъекта и defaults конструктора
//! а также непосредственной destructor-цепочки
//! и сломанный `CanUpgraded`, а также локальный adapter
//! применения joined addon-строк `CDBGoods::LoadGoods` входят в контракт owner-а.
//!
//! Layout сохраняет размеры старых `CShape/CGoods` `0x6C/0xA4`. Constructor
//! по передаёт неизменённый `this` в `CShape::CShape`, а по
//! выполняет `mov [esi+4], 0x2BC`: object type `700` хранится в
//! унаследованном `CBaseObject::m_lType`. Запись по повторно
//! обнуляет inherited `m_lID +0x8`, а не собственное поле, как показывал оригинал.
//! Собственный `m_dwBasePropertiesIndex +0x6C` этот constructor не назначает,
//! поэтому Rust хранит его как `Option<u32>` до setter/decode; amount получает
//! `1`, price `0`, description и addon-vector пусты.
//!
//! оригинал destructor ошибочно показывал ранний возврат после освобождения
//! heap-строки. сохраняет порядок
//! `Release -> vector tidy -> string cleanup -> CShape::~CShape` для обеих
//! форм строки. Rust-композиция материализует только единственный действующий
//! `CShape` base-подобъект и type-default. Helper не называется `new`,
//! собственный cleanup не подменяется пустым `Drop`, а Rust layout не
//! объявляется копией старого ABI.

//! Goods wire сначала включает готовый `CShape`, затем unsigned base index,
//! amount и price, NUL-terminated description, unsigned addon count и каждый
//! `tagAddonProperty`: signed enum, два четырёхбайтовых флага, unsigned count и
//! тройки `u32/i32/i32`. `Vec` заменяет только последовательный STL storage.
//! `Unserialize` сначала выполняет точный `Release`: index/amount становятся
//! нулями, description/addons очищаются, price сохраняется до последующей
//! записи из wire. подтвердил normal return `1`
//! перед security-cookie epilogue.
//!
//! Description читался без length в stack buffer `0x404`. Safe slice/cursor
//! принимает только NUL в этой границе; overflow/overread возвращает локальную
//! типизированную ошибку. Невозможный для старого 32-битного процесса размер
//! Rust-vector больше `u32::MAX` и попытка serialize ещё не назначенного base
//! index возвращаются typed ошибками, а не получают выдуманные wire-байты.
//!
//! Scalar addon lookup сохраняет первое property/value совпадение и signed
//! 32-битное сложение `lBaseValue + lModifier`. Max-stack запрашивает
//! base-properties index, допускает только `GT_USELESS/GT_CONSUMABLE`, берёт
//! `lBaseValue` первого stacking-value с `dwId == 1` и иначе возвращает `1`.
//! PDB исправляет ошибочное имя folded-вызова `CShape::GetDir` на
//! `CGoodsBaseProperties::GetGoodsType`;
//! подтверждает возврат именно signed DWORD по `tagAddonPropertyValue +4`,
//! побитово наблюдаемого как исходный `unsigned long`. После этих двух ответов
//! точечный проверка прекращён.
//! Weight запрашивает те же base-properties, возвращает `0` при отсутствии и
//! умножает unsigned вес одной единицы на amount с точным 32-битным wrapping;
//! использует `IMUL EAX, ESI`.
//! `CanUpgraded` в matching EXE после всех lookup/allocation путей безусловно
//! выполняет `xor eax,eax` по: результат всегда `0`. Rust удаляет
//! только ненаблюдаемые map lookup и временный vector, но сохраняет этот
//! внешний запрет upgrade буквально; исправленное donor-тело сюда не входит.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use crate::dbaccess::worlddb::dbgoods::{
    GoodsAddonPropertySnapshot as DbGoodsAddonPropertySnapshot,
    GoodsAddonPropertyValue as DbGoodsAddonPropertyValue, GoodsAddonValueCountBlock,
    GoodsObjectSnapshot, GoodsPropertiesSnapshot,
};

use super::super::shape::CShape;
use super::super::shape::ShapeDecodeError;
use super::cgoodsbaseproperties::{
    GAP_GOODS_STACKING_LIMIT, GOODS_TYPE_CONSUMABLE, GOODS_TYPE_USELESS,
};
use super::cgoodsfactory::{GoodsBasePropertiesRegistry, query_goods_base_properties};

/// PDB enum `CGoodsBaseProperties::GAP_GOODS_PACKAGE_EXTENTION` (`0xEA`).
pub(crate) const GAP_GOODS_PACKAGE_EXTENTION: i32 = 234;

/// Ошибка безопасной границы `CGoods` wire-owner-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsCodecError {
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

/// Неразрешённая граница материализации frozen `CGoods` для WorldDB.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsDbSnapshotBlock {
 /// Constructor/decode ещё не назначили обязательный base-properties index.
    MissingBasePropertiesIndex,
 /// Исходный `unsigned char` loop не достигает конца слишком длинного values.
    AddonValueCount {
        property_index: usize,
        value_count: usize,
    },
}

/// Safe-граница применения одной joined addon-строки `CDBGoods::LoadGoods`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsLoadedAddonBlock {
    MissingBaseProperties { index: u32 },
    ExistingPropertyHasFewerThanTwoValues { property_type: i32, count: usize },
}

#[derive(Clone, Copy)]
pub(super) struct GoodsAddonPropertyValue {
    id: u32,
    base_value: i32,
    modifier: i32,
}

pub(super) struct GoodsAddonProperty {
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
pub(super) enum FirstAddonModifierAdjustment {
    MissingProperty,
    MissingValue,
    Adjusted,
}

/// Действующая base-часть исходного `CGoods`.
pub(crate) struct CGoods {
    shape_base: CShape,
    base_properties_index: Option<u32>,
    amount: u32,
    price: u32,
    description: Vec<u8>,
    addon_properties: Vec<GoodsAddonProperty>,
}

impl CGoods {
 /// Создаёт только доказанный base-подобъект с object type `700`.
    pub(crate) const fn with_constructor_base_and_type() -> Self {
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

 /// Возвращает унаследованный object type без дополнительных эффектов.
    pub(crate) const fn get_type(&self) -> i32 {
        self.shape_base.get_type()
    }

 /// Возвращает унаследованный signed object ID.
    pub(crate) const fn get_id(&self) -> i32 {
        self.shape_base.get_id()
    }

 /// Присваивает унаследованный signed object ID.
    pub(crate) const fn set_id(&mut self, id: i32) {
        self.shape_base.set_id(id);
    }

 /// Заимствует унаследованный GUID товара.
    pub(crate) const fn get_ex_id(&self) -> &crate::public::guid::CGuid {
        self.shape_base.get_ex_id()
    }

 /// Копирует унаследованный GUID товара.
    pub(crate) const fn set_ex_id(&mut self, ex_id: &crate::public::guid::CGuid) {
        self.shape_base.set_ex_id(ex_id);
    }

 /// Присваивает унаследованное byte- имя до первого NUL.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.shape_base.set_name(name);
    }

 /// Заимствует inherited goods-name без завершающего NUL.
    pub(crate) fn get_goods_name(&self) -> &[u8] {
        self.shape_base.get_name()
    }

 /// Присваивает унаследованный signed graphics ID.
    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.shape_base.set_graphics_id(graphics_id);
    }

 /// Возвращает назначенный unsigned индекс base-properties.
    pub(crate) const fn get_base_properties_index(&self) -> Option<u32> {
        self.base_properties_index
    }

 /// Присваивает unsigned индекс base-properties.
    pub(crate) const fn set_base_properties_index(&mut self, index: u32) {
        self.base_properties_index = Some(index);
    }

 /// Присваивает исходное unsigned количество товара.
    pub(crate) const fn set_amount(&mut self, amount: u32) {
        self.amount = amount;
    }

 /// Возвращает исходное unsigned количество товара.
    pub(crate) const fn get_amount(&self) -> u32 {
        self.amount
    }

 /// Возвращает сумму первого совпавшего addon-value либо signed ноль.
    pub(crate) fn get_addon_property_value(&self, property_type: i32, id: u32) -> i32 {
        self.get_addon_property_values(property_type)
            .iter()
            .find(|value| value.id == id)
            .map_or(0, |value| value.base_value.wrapping_add(value.modifier))
    }

 /// Заимствует все addon-ы в vector-order.
    pub(super) fn get_all_addon_properties(&self) -> &[GoodsAddonProperty] {
        &self.addon_properties
    }

 /// Заимствует все addon-ы для mutation в vector-order.
    pub(super) fn get_all_addon_properties_mut(&mut self) -> &mut Vec<GoodsAddonProperty> {
        &mut self.addon_properties
    }

 /// Проверяет наличие numeric-типа независимо от enabled-флага и values.
    pub(crate) fn is_addon_property_exist(&self, property_type: i32) -> bool {
        self.addon_properties
            .iter()
            .any(|property| property.property_type == property_type)
    }

 /// Применяет одну non-null строку `extend_properties` в row-order.
    pub(crate) fn apply_loaded_addon(
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

 /// Возвращает values первого addon-а совпавшего numeric-типа.
    pub(super) fn get_addon_property_values(
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
    pub(super) fn adjust_first_addon_modifier(
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

 /// Возвращает unsigned stacking-limit для текущих base-properties.
    pub(crate) fn get_max_stack_number(
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

 /// Возвращает unsigned общий вес с 32-битным переполнением.
    pub(crate) fn get_weight(
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

 /// Возвращает сломанный результат matching EXE: upgrade запрещён всегда.
    pub(crate) const fn can_upgraded(&self) -> bool {
        false
    }

 /// Присваивает исходную unsigned цену.
    pub(crate) const fn set_price(&mut self, price: u32) {
        self.price = price;
    }

 /// Возвращает исходную unsigned цену.
    pub(crate) const fn get_price(&self) -> u32 {
        self.price
    }

 /// Материализует точный caller-owned view для `CDBGoods::SaveGoods`.
    pub(crate) fn db_save_snapshot(
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

 /// Сохраняет description bytes как исходный `std::string` owner.
    pub(crate) fn set_goods_description(&mut self, description: &[u8]) {
        let visible = description
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(description.len());
        self.description.clear();
        self.description.extend_from_slice(&description[..visible]);
    }

 /// Добавляет один factory-rolled addon в исходный vector-order.
    pub(super) fn push_factory_addon_property(
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

 /// Сбрасывает точные собственные поля `CGoods::Release`.
    pub(crate) fn release(&mut self) {
        self.base_properties_index = Some(0);
        self.amount = 0;
        for property in &mut self.addon_properties {
            property.clear();
        }
        self.addon_properties.clear();
        self.description.clear();
    }

 /// Клонирует через virtual wire-путь исходного owner-а.
    pub(crate) fn clone_into(&self, target: &mut CGoods) -> Result<bool, GoodsCodecError> {
        let mut wire = Vec::new();
        let _ = self.serialize(&mut wire, true)?;
        let mut cursor = 0;
        target.unserialize(&wire, &mut cursor, true)
    }

 /// Кодирует полный goods snapshot в точном legacy-порядке.
    pub(crate) fn serialize(
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

 /// Декодирует полный goods snapshot после точного раннего `Release`.
    pub(crate) fn unserialize(
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

 /// Сохраняет virtual wrapper `CGoods::AddToByteArray`.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        self.serialize(destination, include_child)
    }

 /// Сохраняет virtual wrapper `CGoods::DecordFromByteArray`.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        self.unserialize(source, cursor, include_child)
    }
}

impl GoodsAddonProperty {
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
