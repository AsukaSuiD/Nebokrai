//! Владелец equipment-container и соседнего exported volume decoder-а.
//!
//! Статус `CEquipmentContainer` constructor/destructor RVA
//! `0x000D9F70/0x000D9E70`, auto/positional `Add` RVA
//! `0x000D8B30/0x000DA020`, `Remove/AddFromDB` RVA
//! `0x000D9B30/0x000DA280`,
//! `Clear/Release` RVA `0x000D8E40/0x000D8F30`, `GetGoods/GetGoodsAmount`
//! RVA `0x000D9470/0x000D94B0`, `GetContentsWeight` RVA `0x000D9080`,
//! `Serialize` RVA `0x000D9530` и разделяемого
//! с `CVolumeLimitGoodsContainer` `Unserialize` RVA `0x000D8DA0`, а также
//! read-side family RVA `0x000D9000/0x000D90F0/0x000D9180/0x000D9280/`
//! `0x000D92F0/0x000D9350/0x000D93E0/0x000DA530` и `AI` RVA `0x000D9210`
//! — `IMPLEMENTED`;
//! остальные operations ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Функции остаются именно в этом `.rs`, потому что их точный PDB source-owner —
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cequipmentcontainer.cpp:22,36,43,135,158,173,210,237,257,291,470,648,670,694,713,731,745,773,813,834`.
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//!
//! Exact PDB задаёт `CGoodsContainer` prefix, secondary `CContainerListener`
//! по `+0x20` и `std::map<EQUIPMENT_COLUMN, CGoods*> m_mEquipment` по `+0x24`.
//! Колонки имеют значения `0..16`: head, body, hand, glove, boot, jewelry,
//! два ornaments-slot-а, medal, posterior, headgear, talisman, frock, wing,
//! manteau, fairy и LingBao. `CGoodsBaseProperties::m_epEquipPlace` задаёт
//! допустимую колонку; ornaments — единственный тип, принимающий две колонки.
//! Positional `Add` сначала отвергает занятую колонку, неизвестные properties,
//! не-equipment и несовпадающий slot, затем кладёт pointer и вызывает
//! listeners. Псевдокод потерял return из-за security-cookie; точный диапазон
//! World EXE `0x004DA020..0x004DA239` подтвердил `eax=0` на отказе и `ebx=1`
//! после вставки. После ответа reverse прекращён. Оба встроенных callback-а
//! уже доказанно сведены к no-op RVA `0x000DBD10`.
//!
//! Rust `BTreeMap<EquipmentColumn, Box<CGoods>>` заменяет только MSVC tree и
//! raw ownership, сохраняя numeric key-order wire-а. Старый `Clear` уведомлял
//! listeners и удалял лишь map nodes без `GarbageCollect`; rejected factory-
//! result decoder-а тоже терялся. Это внутренние leaks без внешних callback-
//! эффектов, поэтому Rust исправляет их обычным `Drop`, а не сохраняет
//! quarantine. `Release` сбрасывает inherited owner `0/0` и заново
//! регистрирует тот же no-op listener.
//!
//! Decoder virtual-вызывает `Clear`, читает unsigned count, затем для каждой
//! записи unsigned cell index и готовый `CGoodsFactory::UnserializeGoods`.
//! Non-null результат передаётся positional `Add(index, goods, nullptr)`, чей
//! bool намеренно игнорируется. Rejected factory-result безопасно уничтожается.
//! Rust receiver выбирает concrete volume/equipment owner, поэтому ранний
//! virtual `Clear` и positional `Add` остаются вариантными.
//! Короткий source сохраняет раннюю очистку, cursor и уже добавленные товары.
//!
//! Exact global `s_dwEquipmentLimit` по `0x0056BE18` имеет значение `9`.
//! `IsFull` считает non-null map values и проверяет строгое равенство, поэтому
//! десять предметов снова дают false. Старый Linux-донор подменил limit числом
//! всех колонок `17`; Rust сохраняет подтверждённый результат EXE. Object-
//! overload position-query сравнивает pointer identity, GUID-overload и `Find`
//! — все 16 байт GUID, обход идёт в numeric map-order. Legacy
//! `GetGoods(index, vector-by-value)` не возвращал наполненную копию; этот
//! внутренний дефект исправлен Rust iterator-ом с тем же base-index фильтром.
//! Auto-`Add` выбирает exact колонку по equip-place; ornaments сначала пробует
//! `6`, затем `7`. Exact ASM исправляет ошибку raw и передаёт исходный context
//! в обе ветви, но встроенные callbacks всё равно no-op. `AddFromDB` выполняет
//! те же type/place/column проверки и прямую вставку без callback. `Remove`
//! передаёт ownership первого GUID-совпадения вызывающему.
//! Weight-family обходит все equipment values без фильтра и сохраняет unsigned
//! wrapping-сумму; numeric tree-order на коммутативный результат не влияет.
//! `AI` в том же numeric tree-order вызывает virtual `CGoods::AI` для каждого
//! non-null товара. Его Rust owner ещё не достигнут, поэтому typed callback
//! передаётся явно, сохраняя dispatch и порядок без raw vtable.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;
use crate::worldserver::appworld::listener::ccontainerlistener::{
    CContainerListener, TraversedContainerObject,
};

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsbaseproperties::GOODS_TYPE_EQUIPMENT;
use super::super::goods::cgoodsfactory::{GoodsBasePropertiesRegistry, unserialize_goods};
use super::camountlimitgoodscontainer::AmountContainerCodecError;
use super::ccontainer::ContainerGuidStorage;
use super::cgoodscontainer::CGoodsContainerState;
use super::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};

const EQUIPMENT_FULL_LIMIT: usize = 9;

/// Numeric equipment-column исходного `CEquipmentContainer`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub(crate) enum EquipmentColumn {
    Head = 0,
    Body = 1,
    Hand = 2,
    Glove = 3,
    Boot = 4,
    Jewelry = 5,
    OrnamentsOne = 6,
    OrnamentsTwo = 7,
    Medal = 8,
    Posterior = 9,
    Headgear = 10,
    Talisman = 11,
    Frock = 12,
    Wing = 13,
    Manteau = 14,
    Fairy = 15,
    Lingbao = 16,
}

impl EquipmentColumn {
    fn from_wire(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Head,
            1 => Self::Body,
            2 => Self::Hand,
            3 => Self::Glove,
            4 => Self::Boot,
            5 => Self::Jewelry,
            6 => Self::OrnamentsOne,
            7 => Self::OrnamentsTwo,
            8 => Self::Medal,
            9 => Self::Posterior,
            10 => Self::Headgear,
            11 => Self::Talisman,
            12 => Self::Frock,
            13 => Self::Wing,
            14 => Self::Manteau,
            15 => Self::Fairy,
            16 => Self::Lingbao,
            _ => return None,
        })
    }

    fn accepts_equip_place(self, equip_place: i32) -> bool {
        matches!(
            (equip_place, self),
            (1, Self::Head)
                | (2, Self::Body)
                | (3, Self::Hand)
                | (4, Self::Glove)
                | (5, Self::Boot)
                | (6, Self::OrnamentsOne | Self::OrnamentsTwo)
                | (7, Self::Medal)
                | (8, Self::Posterior)
                | (9, Self::Jewelry)
                | (10, Self::Headgear)
                | (11, Self::Talisman)
                | (12, Self::Frock)
                | (13, Self::Wing)
                | (14, Self::Manteau)
                | (15, Self::Fairy)
                | (16, Self::Lingbao)
        )
    }
}

/// Ошибка безопасной границы equipment-container codec-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquipmentContainerCodecError {
    Goods(GoodsCodecError),
    ValidGoodsCountOutsideLegacyRange {
        count: usize,
    },
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for EquipmentContainerCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Goods(error) => error.fmt(formatter),
            Self::ValidGoodsCountOutsideLegacyRange { count } => write!(
                formatter,
                "equipment-container содержит {count} валидных товаров вне 32-битного legacy-диапазона"
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

impl Error for EquipmentContainerCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Goods(error) => Some(error),
            _ => None,
        }
    }
}

impl From<GoodsCodecError> for EquipmentContainerCodecError {
    fn from(error: GoodsCodecError) -> Self {
        Self::Goods(error)
    }
}

/// Достигнутая owning-часть исходного `CEquipmentContainer`.
pub(crate) struct CEquipmentContainer {
    container_base: CGoodsContainerState,
    equipment: BTreeMap<EquipmentColumn, Box<CGoods>>,
}

/// Numeric map-order остаётся concrete target-ом inherited GUID slots
/// `CContainer`; поиск и извлечение выбирают первое полное совпадение GUID.
impl ContainerGuidStorage for CEquipmentContainer {
    type Object = CGoods;
    type Removed = Box<CGoods>;

    fn find_by_guid(&self, ex_id: &CGuid) -> Option<&Self::Object> {
        self.equipment
            .values()
            .map(Box::as_ref)
            .find(|goods| goods.get_ex_id() == ex_id)
    }

    fn remove_by_guid(&mut self, ex_id: &CGuid) -> Option<Self::Removed> {
        let column = self
            .equipment
            .iter()
            .find(|(_, goods)| goods.get_ex_id() == ex_id)
            .map(|(column, _)| *column)?;
        self.equipment.remove(&column)
    }
}

impl CEquipmentContainer {
    /// Создаёт exact пустое состояние constructor-а с base-owner `0/0`.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            container_base: CGoodsContainerState::with_constructor_defaults(),
            equipment: BTreeMap::new(),
        }
    }

    /// Добавляет товар только в колонку, разрешённую его base-properties.
    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, EquipmentContainerCodecError> {
        let Some(column) = EquipmentColumn::from_wire(position) else {
            return Ok(Some(goods));
        };
        if self.equipment.contains_key(&column) {
            return Ok(Some(goods));
        }
        let index = goods
            .get_base_properties_index()
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        let Some(properties) =
            super::super::goods::cgoodsfactory::query_goods_base_properties(registry, index)
        else {
            return Ok(Some(goods));
        };
        if properties.get_goods_type() != GOODS_TYPE_EQUIPMENT
            || !column.accepts_equip_place(properties.get_equip_place())
        {
            return Ok(Some(goods));
        }
        self.equipment.insert(column, goods);
        Ok(None)
    }

    /// Выбирает exact equipment-column и делегирует positional `Add`.
    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, EquipmentContainerCodecError> {
        let index = goods
            .get_base_properties_index()
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        let Some(properties) =
            super::super::goods::cgoodsfactory::query_goods_base_properties(registry, index)
        else {
            return Ok(Some(goods));
        };
        if properties.get_goods_type() != GOODS_TYPE_EQUIPMENT {
            return Ok(Some(goods));
        }

        let column = match properties.get_equip_place() {
            1 => EquipmentColumn::Head,
            2 => EquipmentColumn::Body,
            3 => EquipmentColumn::Hand,
            4 => EquipmentColumn::Glove,
            5 => EquipmentColumn::Boot,
            6 if !self.equipment.contains_key(&EquipmentColumn::OrnamentsOne) => {
                EquipmentColumn::OrnamentsOne
            }
            6 if !self.equipment.contains_key(&EquipmentColumn::OrnamentsTwo) => {
                EquipmentColumn::OrnamentsTwo
            }
            6 => return Ok(Some(goods)),
            7 => EquipmentColumn::Medal,
            8 => EquipmentColumn::Posterior,
            9 => EquipmentColumn::Jewelry,
            10 => EquipmentColumn::Headgear,
            11 => EquipmentColumn::Talisman,
            12 => EquipmentColumn::Frock,
            13 => EquipmentColumn::Wing,
            14 => EquipmentColumn::Manteau,
            15 => EquipmentColumn::Fairy,
            16 => EquipmentColumn::Lingbao,
            _ => return Ok(Some(goods)),
        };

        self.add_at(column as u32, goods, registry)
    }

    /// Вставляет DB-товар с exact slot/type/place validation без callback.
    pub(crate) fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, EquipmentContainerCodecError> {
        self.add_at(position, goods, registry)
    }

    /// Вынимает первое GUID-совпадение и передаёт ownership вызывающему.
    pub(crate) fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        <Self as ContainerGuidStorage>::remove_by_guid(self, ex_id)
    }

    /// Очищает map после no-op removed callbacks, исправляя внутреннюю leak.
    pub(crate) fn clear(&mut self) {
        self.equipment.clear();
    }

    /// Уничтожает текущие map-товары и сбрасывает inherited owner `0/0`.
    pub(crate) fn release(&mut self) {
        self.equipment.clear();
        self.container_base.release();
    }

    /// Обходит товары в exact numeric map-order для virtual `CGoods::AI`.
    ///
    /// Товарный AI остаётся owner-ом `CGoods`, поэтому callback передаётся
    /// явно вместо прежнего virtual dispatch через raw pointer.
    pub(crate) fn ai(&mut self, mut on_goods_ai: impl FnMut(&mut CGoods)) {
        for goods in self.equipment.values_mut() {
            on_goods_ai(goods);
        }
    }

    /// Возвращает товар exact numeric equipment-column либо `None`.
    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        EquipmentColumn::from_wire(position)
            .and_then(|column| self.equipment.get(&column))
            .map(Box::as_ref)
    }

    /// Возвращает mutable DB-view exact numeric equipment-column.
    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        EquipmentColumn::from_wire(position)
            .and_then(|column| self.equipment.get_mut(&column))
            .map(Box::as_mut)
    }

    /// Передаёт все товары listener-у в exact numeric map-order.
    pub(crate) fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        let Some(listener) = listener else {
            return;
        };
        for goods in self.equipment.values().map(Box::as_ref) {
            let _ = listener.on_traversing_container(TraversedContainerObject::Goods(goods));
        }
    }

    /// Складывает exact unsigned вес всех equipment-товаров.
    pub(crate) fn get_contents_weight(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, EquipmentContainerCodecError> {
        self.equipment.values().try_fold(0u32, |weight, goods| {
            Ok(weight.wrapping_add(goods.get_weight(registry)?))
        })
    }

    /// Ищет первый товар с полным 16-байтовым GUID в numeric map-order.
    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        <Self as ContainerGuidStorage>::find_by_guid(self, ex_id)
    }

    /// Возвращает numeric column по exact object identity.
    pub(crate) fn query_goods_position_by_object(&self, goods: Option<&CGoods>) -> Option<u32> {
        let goods = goods?;
        self.equipment
            .iter()
            .find(|(_, stored)| std::ptr::eq(stored.as_ref(), goods))
            .map(|(column, _)| *column as u32)
    }

    /// Возвращает numeric column первого полного GUID-совпадения.
    pub(crate) fn query_goods_position(&self, ex_id: &CGuid) -> Option<u32> {
        self.equipment
            .iter()
            .find(|(_, goods)| goods.get_ex_id() == ex_id)
            .map(|(column, _)| *column as u32)
    }

    /// Сохраняет exact equality с global equipment limit `9`.
    pub(crate) fn is_full(&self) -> bool {
        self.equipment.len() == EQUIPMENT_FULL_LIMIT
    }

    /// Возвращает первый товар с exact base-properties index в map-order.
    pub(crate) fn get_the_first_goods(&self, base_properties_index: u32) -> Option<&CGoods> {
        self.equipment
            .values()
            .map(Box::as_ref)
            .find(|goods| goods.get_base_properties_index() == Some(base_properties_index))
    }

    /// Проверяет наличие base-properties index через тот же map traversal.
    pub(crate) fn is_goods_existed(&self, base_properties_index: u32) -> bool {
        self.get_the_first_goods(base_properties_index).is_some()
    }

    /// Возвращает usable iterator вместо legacy vector-by-value копии.
    pub(crate) fn get_goods_by_base_index(
        &self,
        base_properties_index: u32,
    ) -> impl Iterator<Item = &CGoods> {
        self.equipment
            .values()
            .map(Box::as_ref)
            .filter(move |goods| goods.get_base_properties_index() == Some(base_properties_index))
    }

    /// Замораживает map traversal с exact numeric equipment-position.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.equipment
            .iter()
            .map(|(column, goods)| {
                Ok(TraversedGoods {
                    goods: goods.db_save_snapshot(registry)?,
                    position: *column as u8,
                })
            })
            .collect()
    }

    /// Считает только товары с non-null base-properties lookup.
    pub(crate) fn get_goods_amount(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, EquipmentContainerCodecError> {
        let mut count = 0usize;
        for goods in self.equipment.values() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if super::super::goods::cgoodsfactory::query_goods_base_properties(registry, index)
                .is_some()
            {
                count += 1;
            }
        }
        u32::try_from(count)
            .map_err(|_| EquipmentContainerCodecError::ValidGoodsCountOutsideLegacyRange { count })
    }

    /// Кодирует valid-count, numeric column и полный goods-wire в map-order.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, EquipmentContainerCodecError> {
        let count = self.get_goods_amount(registry)?;
        destination.extend_from_slice(&count.to_le_bytes());
        for (column, goods) in &self.equipment {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if super::super::goods::cgoodsfactory::query_goods_base_properties(registry, index)
                .is_some()
            {
                destination.extend_from_slice(&(*column as u32).to_le_bytes());
                let _ = goods.serialize(destination, include_child)?;
            }
        }
        Ok(true)
    }

    /// Декодирует equipment records после обязательного раннего `Clear`.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, EquipmentContainerCodecError> {
        self.clear();
        let count = read_equipment_u32(source, cursor, "goods count")?;
        for _ in 0..count {
            let position = read_equipment_u32(source, cursor, "equipment column")?;
            if let Some(goods) = unserialize_goods(source, cursor, registry)? {
                let _ = self.add_at(position, goods, registry)?;
            }
        }
        Ok(true)
    }
}

impl CVolumeLimitGoodsContainer {
    /// Декодирует exported volume-owner после точного раннего `Clear`.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.clear();
        self.unserialize_records_after_clear(source, cursor, registry)
    }

    /// Декодирует records после уже выполненного derived `Clear`.
    pub(super) fn unserialize_records_after_clear(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        let count = read_volume_u32(source, cursor, "goods count")?;
        for _ in 0..count {
            let position = read_volume_u32(source, cursor, "cell index")?;
            if let Some(goods) = unserialize_goods(source, cursor, registry)? {
                let _ = self.add_at(position, goods, registry)?;
            }
        }
        Ok(true)
    }
}

fn read_volume_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, VolumeContainerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(4) else {
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        }
        .into());
    };
    let Some(bytes) = source.get(offset..end) else {
        // Legacy helper не получал длину source; реакция на короткий буфер
        // определялась выходом за его границы и не имитируется.
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        }
        .into());
    };
    *cursor = end;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("slice содержит ровно четыре байта volume wire"),
    ))
}

fn read_equipment_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, EquipmentContainerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(4) else {
        return Err(EquipmentContainerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        // Legacy helper не получал длину source; реакция на короткий буфер
        // определялась выходом за его границы и не имитируется.
        return Err(EquipmentContainerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    *cursor = end;
    Ok(u32::from_le_bytes(bytes.try_into().expect(
        "slice содержит ровно четыре байта equipment wire",
    )))
}
