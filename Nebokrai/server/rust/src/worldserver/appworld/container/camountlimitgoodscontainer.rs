//! Контейнер товаров с ограничением количества из
//! `camountlimitgoodscontainer.cpp/.h`, подтверждённый `worldserver.exe` и
//! `worldserver.pdb`.
//!
//! `BTreeMap<CGuid, Box<CGoods>>` заменяет MSVC hash-map и ручное владение.
//! Порядок хранения не влияет на подтверждённые суммы и callbacks; повторная
//! вставка освобождает вытесненный товар вместо исторической утечки.
//!
//! Serialize считает только товары с base-properties. Unserialize сначала
//! очищает контейнер, затем сохраняет уже добавленные записи при поздней ошибке;
//! limit и owner при `Clear` не меняются. `Release` дополнительно сбрасывает
//! owner, limit и locked GUID.
//!
//! Locked-товар скрыт не всеми query: position и existence сохраняют прежнее
//! различие. `Lock` не проверяет принадлежность контейнеру, `Unlock` удаляет
//! первое совпадение, GUID-remove возвращает владение вызывающему. Частичный
//! stack-remove создаёт новый товар до уменьшения исходного количества.
//!
//! `AddFromDB` допускает overwrite скрытого locked-товара и не вызывает
//! listener. Исторический shallow `Clone` заменён deep clone, поскольку alias
//! вёл только к double-free; набор данных и порядок целевых изменений сохранены.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;
use crate::worldserver::appworld::listener::ccontainerlistener::{
    CContainerListener, TraversedContainerObject,
};

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, create_goods, query_goods_base_properties, unserialize_goods,
};
use super::cgoodscontainer::{
    CGoodsContainerState, GoodsContainerPositionStorage, remove_from_position,
};
use super::ccontainer::{ContainerGuidStorage, find_by_object_guid};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AmountContainerCodecError {
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

impl fmt::Display for AmountContainerCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Goods(error) => error.fmt(formatter),
            Self::ValidGoodsCountOutsideLegacyRange { count } => write!(
                formatter,
                "amount-container содержит {count} валидных товаров вне 32-битного legacy-диапазона"
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

impl Error for AmountContainerCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Goods(error) => Some(error),
            _ => None,
        }
    }
}

impl From<GoodsCodecError> for AmountContainerCodecError {
    fn from(error: GoodsCodecError) -> Self {
        Self::Goods(error)
    }
}

pub(crate) struct CAmountLimitGoodsContainer {
    container_base: CGoodsContainerState,
    goods: BTreeMap<CGuid, Box<CGoods>>,
    goods_amount_limit: u32,
    locked_goods: Vec<CGuid>,
}

impl ContainerGuidStorage for CAmountLimitGoodsContainer {
    type Object = CGoods;
    type Removed = Box<CGoods>;

    fn find_by_guid(&self, ex_id: &CGuid) -> Option<&Self::Object> {
        self.find(ex_id)
    }

    fn remove_by_guid(&mut self, ex_id: &CGuid) -> Option<Self::Removed> {
        self.remove(ex_id)
    }
}

impl GoodsContainerPositionStorage for CAmountLimitGoodsContainer {
    fn goods_at(&mut self, position: u32) -> Option<&mut CGoods> {
        self.get_goods_mut(position)
    }

    fn remove_by_guid(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        self.remove(ex_id)
    }
}

impl CAmountLimitGoodsContainer {
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            container_base: CGoodsContainerState::with_constructor_defaults(),
            goods: BTreeMap::new(),
            goods_amount_limit: 1,
            locked_goods: Vec::new(),
        }
    }

    pub(crate) const fn set_goods_amount_limit(&mut self, limit: u32) {
        self.goods_amount_limit = limit;
    }

    pub(crate) const fn get_goods_amount_limit(&self) -> u32 {
        self.goods_amount_limit
    }

    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.container_base.set_owner(owner_type, owner_id);
    }

    pub(crate) fn get_goods_amount(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, AmountContainerCodecError> {
        let mut count = 0usize;
        for goods in self.goods.values() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if query_goods_base_properties(registry, index).is_some() {
                count += 1;
            }
        }
        u32::try_from(count)
            .map_err(|_| AmountContainerCodecError::ValidGoodsCountOutsideLegacyRange { count })
    }

    pub(crate) fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, AmountContainerCodecError> {
        Ok(self.goods_amount_limit <= self.get_goods_amount(registry)?)
    }

    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, AmountContainerCodecError> {
        if self.is_full(registry)? {
            return Ok(Some(goods));
        }
        self.insert_unchecked(goods);
        Ok(None)
    }

    pub(crate) fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, AmountContainerCodecError> {
        if self.is_full(registry)? || self.get_goods(position).is_some() {
            return Ok(Some(goods));
        }
        self.insert_unchecked(goods);
        Ok(None)
    }

    pub(super) fn insert_unchecked(&mut self, goods: Box<CGoods>) {
        let ex_id = *goods.get_ex_id();
        let _ = self.goods.insert(ex_id, goods);
    }

    pub(super) fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        let goods = self.goods.get(ex_id)?;
        if self.is_locked(goods) {
            return None;
        }
        self.goods.remove(ex_id)
    }

    pub(crate) fn remove_at<Random>(
        &mut self,
        position: u32,
        amount: u32,
        registry: &GoodsBasePropertiesRegistry,
        random: &mut Random,
    ) -> Result<Option<Box<CGoods>>, AmountContainerCodecError>
    where
        Random: FnMut(i32) -> i32,
    {
        let mut create = |index| create_goods(registry, index, random);
        let mut no_op_listener = |_: &CGoods| {};
        remove_from_position(
            self,
            position,
            amount,
            registry,
            &mut create,
            &mut no_op_listener,
        )
        .map_err(Into::into)
    }

    pub(crate) fn clear(&mut self) {
        self.goods.clear();
        self.locked_goods.clear();
    }

    pub(crate) fn release(&mut self) {
        self.goods_amount_limit = 1;
        self.goods.clear();
        self.locked_goods.clear();
        self.container_base.release();
    }

 /// Обходит все goods в storage-order ровно один раз для virtual `CGoods::AI`.
 /// Сам товарный AI остаётся owner-ом `CGoods`, поэтому callback внедряется
 /// явно вместо прежнего virtual dispatch через оригинал pointer.
    pub(crate) fn ai(&mut self, mut on_goods_ai: impl FnMut(&mut CGoods)) {
        for goods in self.goods.values_mut() {
            on_goods_ai(goods);
        }
    }

    pub(crate) fn clone_into(
        &self,
        target: &mut CAmountLimitGoodsContainer,
    ) -> Result<bool, GoodsCodecError> {
        target.goods_amount_limit = self.goods_amount_limit;

        let mut cloned_goods = BTreeMap::new();
        for (&ex_id, goods) in &self.goods {
            let mut cloned = Box::new(CGoods::with_constructor_base_and_type());
            let _ = goods.as_ref().clone_into(cloned.as_mut())?;
            cloned_goods.insert(ex_id, cloned);
        }
        target.goods = cloned_goods;
        Ok(true)
    }

    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        let goods = self.goods.get(ex_id).map(Box::as_ref)?;
        (!self.is_locked(goods)).then_some(goods)
    }

    pub(crate) fn find_object(&self, goods: Option<&CGoods>) -> Option<&CGoods> {
        find_by_object_guid(self, goods.map(CGoods::get_ex_id))
    }

    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        if position >= self.goods_amount_limit {
            return None;
        }

        let goods = self
            .goods
            .values()
            .nth(position as usize)
            .map(Box::as_ref)?;
        (!self.is_locked(goods)).then_some(goods)
    }

    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        let ex_id = *self.get_goods(position)?.get_ex_id();
        self.goods_mut(&ex_id)
    }

    pub(crate) fn get_the_first_goods(&self, base_properties_index: u32) -> Option<&CGoods> {
        self.goods.values().map(Box::as_ref).find(|goods| {
            goods.get_base_properties_index() == Some(base_properties_index)
                && !self.is_locked(goods)
        })
    }

    pub(crate) fn is_goods_existed(&self, base_properties_index: u32) -> bool {
        self.goods
            .values()
            .any(|goods| goods.get_base_properties_index() == Some(base_properties_index))
    }

    pub(crate) fn query_goods_position(&self, goods: Option<&CGoods>) -> Option<u32> {
        let goods = goods?;
        self.goods
            .values()
            .position(|stored| std::ptr::eq(stored.as_ref(), goods))
            .and_then(|position| u32::try_from(position).ok())
    }

    pub(crate) fn query_goods_position_by_guid(&self, ex_id: &CGuid) -> Option<u32> {
        self.goods
            .values()
            .position(|goods| goods.get_ex_id() == ex_id)
            .and_then(|position| u32::try_from(position).ok())
    }

    pub(crate) fn get_goods_by_base_index<'container>(
        &'container self,
        base_properties_index: u32,
        mut output: Vec<&'container CGoods>,
    ) {
        output.extend(self.goods.values().map(Box::as_ref).filter(|goods| {
            goods.get_base_properties_index() == Some(base_properties_index)
                && !self.is_locked(goods)
        }));
    }

    pub(crate) fn lock(&mut self, goods: Option<&CGoods>) -> i32 {
        let Some(goods) = goods else {
            return 0;
        };
        if self.is_locked(goods) {
            return 0;
        }

        self.locked_goods.push(*goods.get_ex_id());
        1
    }

    pub(crate) fn unlock(&mut self, goods: Option<&CGoods>) -> i32 {
        let Some(goods) = goods else {
            return 0;
        };
        let Some(position) = self
            .locked_goods
            .iter()
            .position(|locked| locked == goods.get_ex_id())
        else {
            return 0;
        };

        self.locked_goods.remove(position);
        1
    }

    fn is_locked(&self, goods: &CGoods) -> bool {
        self.locked_goods
            .iter()
            .any(|locked| locked == goods.get_ex_id())
    }

    pub(crate) fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        let Some(listener) = listener else {
            return;
        };

        for goods in self.goods.values().map(Box::as_ref) {
            let _ = listener.on_traversing_container(TraversedContainerObject::Goods(goods));
        }
    }

    pub(crate) fn get_contents_weight(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, AmountContainerCodecError> {
        self.goods.values().try_fold(0u32, |weight, goods| {
            Ok(weight.wrapping_add(goods.get_weight(registry)?))
        })
    }

    pub(super) fn goods(&self) -> impl Iterator<Item = &CGoods> {
        self.goods.values().map(Box::as_ref)
    }

    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.goods()
            .map(|goods| {
                Ok(TraversedGoods {
                    goods: goods.db_save_snapshot(registry)?,
                    position: self.query_goods_position(Some(goods)).unwrap_or(0) as u8,
                })
            })
            .collect()
    }

    pub(super) fn goods_mut(&mut self, ex_id: &CGuid) -> Option<&mut CGoods> {
        self.goods.get_mut(ex_id).map(Box::as_mut)
    }

    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, AmountContainerCodecError> {
        let count = self.get_goods_amount(registry)?;
        destination.extend_from_slice(&count.to_le_bytes());
        for goods in self.goods.values() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if query_goods_base_properties(registry, index).is_some() {
                let _ = goods.serialize(destination, include_child)?;
            }
        }
        Ok(true)
    }

    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, AmountContainerCodecError> {
        self.clear();
        let count = read_amount_u32(source, cursor, "goods count")?;
        for _ in 0..count {
            if let Some(goods) = unserialize_goods(source, cursor, registry)? {
                let _ = self.add(goods, registry)?;
            }
        }
        Ok(true)
    }
}

fn read_amount_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, AmountContainerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(4) else {
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
 // Старый вспомогательный код не получал длину источника. При коротком
 // буфере он выходил за его границы; Rust не воспроизводит это UB.
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    *cursor = end;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("slice содержит ровно четыре байта legacy count"),
    ))
}
