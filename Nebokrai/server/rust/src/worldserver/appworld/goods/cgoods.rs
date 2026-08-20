//! Владелец товара исторического `WorldServer`.
//!
//! Статус `CGoods::Serialize/Unserialize` RVA `0x000518E0/0x00053220`,
//! byte-array wrappers `0x000516E0/0x00051700`, `Release` RVA `0x000533C0`,
//! scalar `GetAddonPropertyValues` RVA `0x000517B0`, `GetMaxStackNumber` RVA
//! `0x00052530`, `GetWeight` RVA `0x00051730`,
//! `SetExID` RVA `0x000530E0`, base-подобъекта и defaults конструктора RVA
//! `0x00053060`, а также непосредственной destructor-цепочки RVA `0x000534B0`
//! — `IMPLEMENTED`;
//! остальной корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.h` и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:16-36`.
//!
//! Exact PDB задаёт размеры старых `CShape/CGoods` `0x6C/0xA4`. Constructor
//! по `0x00453079` передаёт неизменённый `this` в `CShape::CShape`, а по
//! `0x004530B5` выполняет `mov [esi+4], 0x2BC`: object type `700` хранится в
//! унаследованном `CBaseObject::m_lType`. Запись по `0x004530AF` повторно
//! обнуляет inherited `m_lID +0x8`, а не собственное поле, как показывал raw.
//! Собственный `m_dwBasePropertiesIndex +0x6C` этот constructor не назначает,
//! поэтому Rust хранит его как `Option<u32>` до setter/decode; amount получает
//! `1`, price `0`, description и addon-vector пусты.
//!
//! Raw destructor ошибочно показывал ранний возврат после освобождения
//! heap-строки. Exact EXE `0x004534DB..0x0045351F` сохраняет порядок
//! `Release -> vector tidy -> string cleanup -> CShape::~CShape` для обеих
//! форм строки. Rust-композиция материализует только единственный достигнутый
//! `CShape` base-подобъект и type-default. Helper не называется `new`,
//! собственный cleanup не подменяется пустым `Drop`, а Rust layout не
//! объявляется копией старого ABI.

//! Goods wire сначала включает готовый `CShape`, затем unsigned base index,
//! amount и price, NUL-terminated description, unsigned addon count и каждый
//! `tagAddonProperty`: signed enum, два четырёхбайтовых флага, unsigned count и
//! тройки `u32/i32/i32`. `Vec` заменяет только последовательный STL storage.
//! `Unserialize` сначала выполняет точный `Release`: index/amount становятся
//! нулями, description/addons очищаются, price сохраняется до последующей
//! записи из wire. Exact `0x00453390..0x004533B5` подтвердил normal return `1`
//! перед security-cookie epilogue; после ответа reverse прекращён.
//!
//! Description читался без length в stack buffer `0x404`. Safe slice/cursor
//! принимает только NUL в этой границе; overflow/overread остаётся локальным
//! `BLOCKED_MISSING_FACT`. Невозможный для старого 32-битного процесса размер
//! Rust-vector больше `u32::MAX` и попытка serialize ещё не назначенного base
//! index возвращаются typed ошибками, а не получают выдуманные wire-байты.
//!
//! Scalar addon lookup сохраняет первое property/value совпадение и signed
//! 32-битное сложение `lBaseValue + lModifier`. Max-stack запрашивает exact
//! base-properties index, допускает только `GT_USELESS/GT_CONSUMABLE`, берёт
//! `lBaseValue` первого stacking-value с `dwId == 1` и иначе возвращает `1`.
//! PDB исправляет ошибочное имя folded-вызова `CShape::GetDir` на
//! `CGoodsBaseProperties::GetGoodsType`; exact EXE `0x004525E1..0x004525EE`
//! подтверждает возврат именно signed DWORD по `tagAddonPropertyValue +4`,
//! побитово наблюдаемого как исходный `unsigned long`. После этих двух ответов
//! точечный reverse прекращён.
//! Weight запрашивает те же base-properties, возвращает `0` при отсутствии и
//! умножает unsigned вес одной единицы на amount с точным 32-битным wrapping;
//! exact EXE использует `IMUL EAX, ESI`.

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
#[derive(Clone, Copy, Debug)]
pub(crate) enum GoodsDbSnapshotBlock {
    /// Constructor/decode ещё не назначили обязательный base-properties index.
    MissingBasePropertiesIndex,
    /// Исходный `unsigned char` loop не достигает конца слишком длинного values.
    AddonValueCount {
        property_index: usize,
        value_count: usize,
    },
}

#[derive(Clone, Copy)]
struct GoodsAddonPropertyValue {
    id: u32,
    base_value: i32,
    modifier: i32,
}

struct GoodsAddonProperty {
    property_type: i32,
    is_enabled: i32,
    is_implicit_attribute: i32,
    values: Vec<GoodsAddonPropertyValue>,
}

/// Достигнутая base-часть исходного `CGoods`.
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
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .and_then(|property| property.values.iter().find(|value| value.id == id))
            .map_or(0, |value| value.base_value.wrapping_add(value.modifier))
    }

    /// Возвращает exact unsigned stacking-limit для текущих base-properties.
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

    /// Возвращает exact unsigned общий вес с 32-битным переполнением.
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

    /// Сбрасывает точные собственные поля `CGoods::Release`.
    pub(crate) fn release(&mut self) {
        self.base_properties_index = Some(0);
        self.amount = 0;
        self.addon_properties.clear();
        self.description.clear();
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
        let property_type = read_goods_i32(source, cursor, "tagAddonProperty.gapType")?;
        let is_enabled = read_goods_i32(source, cursor, "tagAddonProperty.bIsEnabled")?;
        let is_implicit_attribute =
            read_goods_i32(source, cursor, "tagAddonProperty.bIsImplicitAttribute")?;
        let count = read_goods_u32(source, cursor, "tagAddonProperty.vValues count")?;
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(GoodsAddonPropertyValue {
                id: read_goods_u32(source, cursor, "tagAddonPropertyValue.dwId")?,
                base_value: read_goods_i32(source, cursor, "tagAddonPropertyValue.lBaseValue")?,
                modifier: read_goods_i32(source, cursor, "tagAddonPropertyValue.lModifier")?,
            });
        }
        Ok(Self {
            property_type,
            is_enabled,
            is_implicit_attribute,
            values,
        })
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

    // BLOCKED_MISSING_FACT: `_GetStringFromByteArray` не получал capacity и
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
        // BLOCKED_MISSING_FACT: legacy helper не получал длину source.
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.h

// ============================================================================
// FUNCTION: CGoods::GetBasePropertiesIndex
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:43
// RVA: 0x000516B0
// ADDRESS: 004516b0
// PROTOTYPE: ulong __thiscall GetBasePropertiesIndex(void)
//
// IMPLEMENTED выше; Rust `Option` сохраняет constructor-uninitialized границу.

// ============================================================================
// FUNCTION: CGoods::SetAmount
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:51
// RVA: 0x000516C0
// ADDRESS: 004516c0
// PROTOTYPE: void __thiscall SetAmount(ulong param_1)
//
// IMPLEMENTED выше; unsigned присваивание не меняет форму значения.

// ============================================================================
// FUNCTION: CGoods::GetAmount
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:63
// RVA: 0x000516D0
// ADDRESS: 004516d0
// PROTOTYPE: ulong __thiscall GetAmount(void)
//
// IMPLEMENTED выше.

// ============================================================================
// FUNCTION: CGoods::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:349
// RVA: 0x000516E0
// ADDRESS: 004516e0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// IMPLEMENTED выше; wrapper возвращает bool результата `Serialize`.

// ============================================================================
// FUNCTION: CGoods::DecordFromByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:354
// RVA: 0x00051700
// ADDRESS: 00451700
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// IMPLEMENTED выше; wrapper возвращает bool результата `Unserialize`.

// ============================================================================
// FUNCTION: CGoods::SetBasePropertiesIndex
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:383
// RVA: 0x00051720
// ADDRESS: 00451720
// PROTOTYPE: void __thiscall SetBasePropertiesIndex(ulong param_1)
//
// IMPLEMENTED выше; setter переводит `Option` в назначенное состояние.

// ============================================================================
// FUNCTION: CGoods::GetWeight
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:509
// RVA: 0x00051730
// ADDRESS: 00451730
// PROTOTYPE: ulong __thiscall GetWeight(void)
//
// IMPLEMENTED выше; explicit registry заменяет process-global factory, а
// unsigned multiplication остаётся 32-битной wrapping-операцией.

// ============================================================================
// FUNCTION: CGoods::SetPrice
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:531
// RVA: 0x00051760
// ADDRESS: 00451760
// PROTOTYPE: void __thiscall SetPrice(ulong param_1)
//
// IMPLEMENTED выше.

// ============================================================================
// FUNCTION: CGoods::GetPrice
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:539
// RVA: 0x00051770
// ADDRESS: 00451770
// PROTOTYPE: ulong __thiscall GetPrice(void)
//
// IMPLEMENTED выше.

// ============================================================================
// FUNCTION: CGoods::GetAllAddonProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:585
// RVA: 0x00051780
// ADDRESS: 00451780
// PROTOTYPE: vector<CGoods::tagAddonProperty,std::allocator<CGoods::tagAddonProperty>_> * __thiscall GetAllAddonProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonPropertyValue::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:407
// RVA: 0x00051790
// ADDRESS: 00451790
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetGoodsName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:564
// RVA: 0x000517A0
// ADDRESS: 004517a0
// PROTOTYPE: char * __thiscall GetGoodsName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetAddonPropertyValues
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:232
// RVA: 0x000517B0
// ADDRESS: 004517b0
// PROTOTYPE: long __thiscall GetAddonPropertyValues(GOODS_ADDON_PROPERTIES param_1, ulong param_2)
//
// IMPLEMENTED выше как `get_addon_property_value`: первое совпадение и
// signed wrapping `lBaseValue + lModifier` сохранены.
//

// ============================================================================
// FUNCTION: CGoods::IsAddonProperyExist
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:256
// RVA: 0x00051880
// ADDRESS: 00451880
// PROTOTYPE: bool __thiscall IsAddonProperyExist(GOODS_ADDON_PROPERTIES param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::Serialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:295
// RVA: 0x000518E0
// ADDRESS: 004518e0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// IMPLEMENTED выше; shape, scalar, C-string и addon sequence сохранены.

// ============================================================================
// FUNCTION: CGoods::SetGoodsDescribe
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:571
// RVA: 0x00051A70
// ADDRESS: 00451a70
// PROTOTYPE: void __thiscall SetGoodsDescribe(char * param_1)
//
// /* public: void __thiscall CGoods::SetGoodsDescribe(char const *) */
//
// IMPLEMENTED выше; Rust slice заменяет non-null C-string и усекается по
// первому NUL как исходный `strlen`.

// ============================================================================
// FUNCTION: CGoods::Clone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:276
// RVA: 0x000523B0
// ADDRESS: 004523b0
// PROTOTYPE: int __thiscall Clone(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetMaxStackNumber
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:71
// RVA: 0x00052530
// ADDRESS: 00452530
// PROTOTYPE: ulong __thiscall GetMaxStackNumber(void)
//
// IMPLEMENTED выше как `get_max_stack_number`; exact EXE подтверждает
// `GetGoodsType` и возврат `lBaseValue` по offset `+4`.
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::tagAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:441
// RVA: 0x00052660
// ADDRESS: 00452660
// PROTOTYPE: undefined __thiscall tagAddonProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::~tagAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:450
// RVA: 0x00052680
// ADDRESS: 00452680
// PROTOTYPE: void __thiscall ~tagAddonProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetAddonPropertyValues
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:192
// RVA: 0x00052760
// ADDRESS: 00452760
// PROTOTYPE: void __thiscall GetAddonPropertyValues(GOODS_ADDON_PROPERTIES param_1, vector<CGoods::tagAddonPropertyValue,std::allocator<CGoods::tagAddonPropertyValue>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::Unserialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:488
// RVA: 0x00052810
// ADDRESS: 00452810
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// IMPLEMENTED выше; unsigned count и последовательные `u32/i32/i32` values.

// ============================================================================
// FUNCTION: CGoods::CanUpgraded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:113
// RVA: 0x000528E0
// ADDRESS: 004528e0
// PROTOTYPE: int __thiscall CanUpgraded(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:16
// RVA: 0x00053060
// ADDRESS: 00453060
// PROTOTYPE: undefined __thiscall CGoods(void)
//
// IMPLEMENTED выше; exact constructor facts и uninitialized base index
// сохранены через `Option` без vtable/SEH plumbing.

// ============================================================================
// FUNCTION: CGoods::SetExID
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.h:116
// RVA: 0x000530E0
// ADDRESS: 004530e0
// PROTOTYPE: void __thiscall SetExID(CGUID * param_1)
//
// IMPLEMENTED выше через единственный `CShape/CBaseObject` base-подобъект.

// ============================================================================
// FUNCTION: CGoods::Unserialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:314
// RVA: 0x00053220
// ADDRESS: 00453220
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// IMPLEMENTED выше; exact tail `0x00453390..0x004533B5` подтверждает normal
// return `1`. Temporary copy/clear и EH/security-cookie заменены Rust move/Drop.

// ============================================================================
// FUNCTION: CGoods::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:367
// RVA: 0x000533C0
// ADDRESS: 004533c0
// PROTOTYPE: void __thiscall Release(void)
//
// IMPLEMENTED выше; index/amount reset и оба owned collection/string cleanup
// сохранены, price сознательно не меняется.

// ============================================================================
// FUNCTION: CGoods::~CGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:36
// RVA: 0x000534B0
// ADDRESS: 004534b0
// PROTOTYPE: void __thiscall ~CGoods(void)
//
// Rust drop-порядок owned addon-vector, description и `CShape` сохраняет весь
// наблюдаемый cleanup; предварительный `Release` и STL `_Tidy` не требуют
// отдельных тел, потому что после входа в destructor поля больше не читаются.

// ============================================================================
// FUNCTION: CGoods::tagAddonPropertyValue::tagAddonPropertyValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:391
// RVA: 0x000D4950
// ADDRESS: 004d4950
// PROTOTYPE: undefined __thiscall tagAddonPropertyValue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::Serialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:472
// RVA: 0x000D4A60
// ADDRESS: 004d4a60
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// IMPLEMENTED выше; `param_2` не влияет на доказанный wire.

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoods.cpp:457
// RVA: 0x000D4CF0
// ADDRESS: 004d4cf0
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
