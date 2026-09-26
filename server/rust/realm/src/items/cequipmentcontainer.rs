//! Экипировка `CEquipmentContainer` из `cequipmentcontainer.cpp/.h`.
//!
//! Товар принимается только в колонку, разрешённую его base-properties;
//! ornaments последовательно пробуют slots 6 и 7. Numeric map-order определяет
//! wire, GUID lookup, вес и вызовы `CGoods::AI`. Глобальный equipment limit
//! равен 9, а `IsFull` намеренно проверяет равенство, поэтому большее число
//! занятых колонок не считается full.
//!
//! Decoder сначала вызывает virtual `Clear`, затем читает cell и полный товар;
//! отказ `Add` не отменяет уже разобранные записи. `AddFromDB` выполняет те же
//! type/place проверки, но вставляет без callback. Встроенные callbacks — no-op.
//! `BTreeMap` и `Box` заменяют MSVC tree и ручное владение, устраняя только
//! внутренние утечки отклонённых или вытесненных товаров.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::content::goodslistener::TraversedGoods;
use nebokrai_shared::values::CGuid;
use crate::items::ccontainerlistener::{
    CContainerListener, TraversedContainerObject,
};

use crate::content::cgoods::{CGoods, GoodsCodecError};
use crate::content::goods::{GOODS_TYPE_EQUIPMENT, GoodsBasePropertiesRegistry};
use crate::content::cgoodsfactory::unserialize_goods;
use crate::items::camountlimitgoodscontainer::AmountContainerCodecError;
use crate::items::ccontainer::ContainerGuidStorage;
use crate::items::cgoodscontainer::CGoodsContainerState;
use crate::items::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};

const EQUIPMENT_FULL_LIMIT: usize = 9;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum EquipmentColumn {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentContainerCodecError {
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

pub struct CEquipmentContainer {
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
    pub const fn with_constructor_defaults() -> Self {
        Self {
            container_base: CGoodsContainerState::with_constructor_defaults(),
            equipment: BTreeMap::new(),
        }
    }

    pub fn add_at(
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
            crate::content::cgoodsfactory::query_goods_base_properties(registry, index)
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

    pub fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, EquipmentContainerCodecError> {
        let index = goods
            .get_base_properties_index()
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        let Some(properties) =
            crate::content::cgoodsfactory::query_goods_base_properties(registry, index)
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

    pub fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, EquipmentContainerCodecError> {
        self.add_at(position, goods, registry)
    }

    pub fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        <Self as ContainerGuidStorage>::remove_by_guid(self, ex_id)
    }

    pub fn clear(&mut self) {
        self.equipment.clear();
    }

    pub fn release(&mut self) {
        self.equipment.clear();
        self.container_base.release();
    }

 /// Обходит товары в numeric map-order для virtual `CGoods::AI`.
 ///
 /// Товарный AI остаётся owner-ом `CGoods`, поэтому callback передаётся
 /// явно вместо прежнего virtual dispatch через оригинал pointer.
    pub fn ai(&mut self, mut on_goods_ai: impl FnMut(&mut CGoods)) {
        for goods in self.equipment.values_mut() {
            on_goods_ai(goods);
        }
    }

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        EquipmentColumn::from_wire(position)
            .and_then(|column| self.equipment.get(&column))
            .map(Box::as_ref)
    }

    pub fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        EquipmentColumn::from_wire(position)
            .and_then(|column| self.equipment.get_mut(&column))
            .map(Box::as_mut)
    }

    pub fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        let Some(listener) = listener else {
            return;
        };
        for goods in self.equipment.values().map(Box::as_ref) {
            let _ = listener.on_traversing_container(TraversedContainerObject::Goods(goods));
        }
    }

    pub fn get_contents_weight(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, EquipmentContainerCodecError> {
        self.equipment.values().try_fold(0u32, |weight, goods| {
            Ok(weight.wrapping_add(goods.get_weight(registry)?))
        })
    }

    pub fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        <Self as ContainerGuidStorage>::find_by_guid(self, ex_id)
    }

    pub fn query_goods_position_by_object(&self, goods: Option<&CGoods>) -> Option<u32> {
        let goods = goods?;
        self.equipment
            .iter()
            .find(|(_, stored)| std::ptr::eq(stored.as_ref(), goods))
            .map(|(column, _)| *column as u32)
    }

    pub fn query_goods_position(&self, ex_id: &CGuid) -> Option<u32> {
        self.equipment
            .iter()
            .find(|(_, goods)| goods.get_ex_id() == ex_id)
            .map(|(column, _)| *column as u32)
    }

    pub fn is_full(&self) -> bool {
        self.equipment.len() == EQUIPMENT_FULL_LIMIT
    }

    pub fn get_the_first_goods(&self, base_properties_index: u32) -> Option<&CGoods> {
        self.equipment
            .values()
            .map(Box::as_ref)
            .find(|goods| goods.get_base_properties_index() == Some(base_properties_index))
    }

    pub fn is_goods_existed(&self, base_properties_index: u32) -> bool {
        self.get_the_first_goods(base_properties_index).is_some()
    }

    pub fn get_goods_by_base_index(
        &self,
        base_properties_index: u32,
    ) -> impl Iterator<Item = &CGoods> {
        self.equipment
            .values()
            .map(Box::as_ref)
            .filter(move |goods| goods.get_base_properties_index() == Some(base_properties_index))
    }

    pub fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, crate::content::cgoods::GoodsDbSnapshotBlock> {
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

    pub fn get_goods_amount(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, EquipmentContainerCodecError> {
        let mut count = 0usize;
        for goods in self.equipment.values() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if crate::content::cgoodsfactory::query_goods_base_properties(registry, index)
                .is_some()
            {
                count += 1;
            }
        }
        u32::try_from(count)
            .map_err(|_| EquipmentContainerCodecError::ValidGoodsCountOutsideLegacyRange { count })
    }

    pub fn serialize(
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
            if crate::content::cgoodsfactory::query_goods_base_properties(registry, index)
                .is_some()
            {
                destination.extend_from_slice(&(*column as u32).to_le_bytes());
                let _ = goods.serialize(destination, include_child)?;
            }
        }
        Ok(true)
    }

    pub fn unserialize(
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
    pub fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.clear();
        self.unserialize_records_after_clear(source, cursor, registry)
    }

    pub fn unserialize_records_after_clear(
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
        // Старый вспомогательный код не получал длину источника. При коротком
        // буфере он выходил за его границы; Rust не воспроизводит это UB.
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
        // Старый вспомогательный код не получал длину источника. При коротком
        // буфере он выходил за его границы; Rust не воспроизводит это UB.
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
