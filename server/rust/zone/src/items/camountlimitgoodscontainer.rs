//! Storage/query/lock core `CAmountLimitGoodsContainer` исторического
//! GameServer: owned хранилище GUID→`CGoods` с limit, lock-видимостью и
//! позиционными операциями.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/camountlimitgoodscontainer.cpp`.
//! Legacy `stdext::hash_map` хранит GUID→owned `CGoods`, но его traversal идёт
//! по внутреннему list в insertion order. `IndexMap` берёт на себя safe owned
//! storage и GUID-index: duplicate заменяет value на прежней позиции,
//! `shift_remove` сохраняет порядок оставшихся элементов. Position, lock,
//! stacking и partial-effect контракты остаются GameServer-адаптером.
//!
//! Constructor RVA `0x000FCD60` задаёт limit `1`, пустые goods/locks и
//! регистрирует собственный listener-subobject. В Rust его OnObjectAdded/
//! OnObjectRemoved structurally принадлежат concrete owner-у и не требуют
//! самоссылочного pointer handle. Read-only traversal, lock/unlock, lookup,
//! ordered clear и mode-dependent release перенесены буквально; locked goods
//! скрыты от public find/get. `GCM_TEST` не уничтожает отделённые goods: Rust
//! возвращает их вызывающему, сохраняя ownership без legacy raw pointers.
//! Полный persisted codec достигнут общим player GameSave-проходом; listener
//! messages и player AI tree ещё требуют реконструкции.

use super::ccontainer::ContainerListenerHandle;
use super::cgoods::{CGoods, GoodsDecodeError};
use super::cgoodscontainer::{CGoodsContainer, GoodsContainerMode, GoodsStackMergeOutcome};
use crate::content::goodsfactory::CGoodsFactory;
use crate::regions::ShapeIdentity;
use indexmap::IndexMap;
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use nebokrai_shared::values::CGuid;
use thiserror::Error;

#[must_use = "report содержит обязательные listener- и ownership-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmountLimitGoodsAdded {
    pub owner_type: i32,
    pub owner_id: i32,
    pub position: Option<u32>,
    pub identity: ShapeIdentity,
    pub amount: u32,
    pub listeners: Vec<ContainerListenerHandle>,
    pub replaced: Option<CGoods>,
}

#[must_use = "report содержит обязательные listener- и ownership-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmountLimitGoodsRemoved {
    pub owner_type: i32,
    pub owner_id: i32,
    pub position: Option<u32>,
    pub amount: u32,
    pub listeners: Vec<ContainerListenerHandle>,
    pub goods: CGoods,
}

#[must_use = "report содержит обязательные listener- и ownership-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmountLimitGoodsSplit {
    pub owner_type: i32,
    pub owner_id: i32,
    pub position: Option<u32>,
    pub base_properties_index: u32,
    pub source: ShapeIdentity,
    pub amount: u32,
    pub listeners: Vec<ContainerListenerHandle>,
    pub goods: CGoods,
}

#[must_use = "результат удаления нужно передать runtime owner-у"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AmountLimitGoodsTaken {
    Split(AmountLimitGoodsSplit),
    Removed(AmountLimitGoodsRemoved),
}

#[must_use = "очищенные goods и listener-эффекты нельзя потерять"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmountLimitGoodsCleared {
    pub removed: Vec<AmountLimitGoodsRemoved>,
}

#[must_use = "в test mode результат владеет отделёнными goods"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AmountLimitGoodsRelease {
    Collected { count: usize },
    Detached { goods: Vec<CGoods> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CAmountLimitGoodsContainer {
    base: CGoodsContainer,
    goods: IndexMap<CGuid, CGoods>,
    locked_goods: Vec<CGuid>,
    goods_amount_limit: u32,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum AmountLimitGoodsCodecError {
    #[error(transparent)]
    Goods(#[from] GoodsDecodeError),
    #[error("amount container обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("amount container получил {count} goods при limit {limit}")]
    ContainerLimitExceeded {
        count: u32,
        limit: u32,
    },
}

impl Default for CAmountLimitGoodsContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CAmountLimitGoodsContainer {
    pub fn new() -> Self {
        Self {
            base: CGoodsContainer::new(),
            goods: IndexMap::new(),
            locked_goods: Vec::new(),
            goods_amount_limit: 1,
        }
    }

    pub const fn base(&self) -> &CGoodsContainer {
        &self.base
    }

    pub const fn base_mut(&mut self) -> &mut CGoodsContainer {
        &mut self.base
    }

    pub fn traversing_goods(&self) -> impl ExactSizeIterator<Item = &CGoods> {
        self.goods.values()
    }

    pub fn traversing_goods_mut(
        &mut self,
    ) -> impl ExactSizeIterator<Item = &mut CGoods> {
        self.goods.values_mut()
    }

    pub const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.base.set_owner(owner_type, owner_id);
    }

    pub const fn set_goods_amount_limit(&mut self, limit: u32) {
        self.goods_amount_limit = limit;
    }

    pub const fn goods_amount_limit(&self) -> u32 {
        self.goods_amount_limit
    }

    pub fn goods_amount(&self, factory: &CGoodsFactory) -> u32 {
        self.goods.values().fold(0u32, |amount, goods| {
            amount.wrapping_add(u32::from(
                factory
                    .query_goods_base_properties(goods.base_properties_index())
                    .is_some(),
            ))
        })
    }

    pub fn is_full(&self, factory: &CGoodsFactory) -> bool {
        self.goods_amount_limit <= self.goods_amount(factory)
    }

    pub fn is_locked(&self, ex_id: CGuid) -> bool {
        self.locked_goods.contains(&ex_id)
    }

    pub fn lock(&mut self, ex_id: CGuid) -> bool {
        if self.locked_goods.contains(&ex_id) {
            return false;
        }
        self.locked_goods.push(ex_id);
        true
    }

    pub fn unlock(&mut self, ex_id: CGuid) -> bool {
        let Some(index) = self.locked_goods.iter().position(|locked| *locked == ex_id) else {
            return false;
        };
        self.locked_goods.remove(index);
        true
    }

    pub fn find(&self, ex_id: CGuid) -> Option<&CGoods> {
        if self.locked_goods.contains(&ex_id) {
            return None;
        }
        self.goods.get(&ex_id)
    }

    pub fn find_mut(&mut self, ex_id: CGuid) -> Option<&mut CGoods> {
        if self.locked_goods.contains(&ex_id) {
            return None;
        }
        self.goods.get_mut(&ex_id)
    }

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        if self.goods_amount_limit <= position {
            return None;
        }
        self.goods
            .get_index(position as usize)
            .map(|(_, goods)| goods)
            .filter(|goods| !self.is_locked(goods.identity().ex_id))
    }

    pub fn get_first_goods(&self, base_properties_index: u32) -> Option<&CGoods> {
        self.goods.values().find(|goods| {
            goods.base_properties_index() == base_properties_index
                && !self.is_locked(goods.identity().ex_id)
        })
    }

    pub fn get_goods_by_base_properties(&self, base_properties_index: u32) -> Vec<&CGoods> {
        self.goods
            .values()
            .filter(|goods| {
                goods.base_properties_index() == base_properties_index
                    && !self.is_locked(goods.identity().ex_id)
            })
            .collect()
    }

    pub fn query_goods_position(&self, ex_id: CGuid) -> Option<u32> {
        self.goods
            .get_index_of(&ex_id)
            .map(|position| position as u32)
    }

    pub fn is_goods_existed(&self, base_properties_index: u32) -> bool {
        self.goods
            .values()
            .any(|goods| goods.base_properties_index() == base_properties_index)
    }

    pub fn contents_weight(&self, factory: &CGoodsFactory) -> u32 {
        self.goods.values().fold(0u32, |weight, goods| {
            weight.wrapping_add(goods.weight(factory))
        })
    }

    /// Замыкает storage-часть exact `Add(CBaseObject*)`; player AI tree и
    /// ordered listener callbacks описаны отчётом для runtime dispatcher-а.
    pub fn add_goods(
        &mut self,
        goods: CGoods,
        factory: &CGoodsFactory,
    ) -> Result<AmountLimitGoodsAdded, CGoods> {
        if self.is_full(factory) {
            return Err(goods);
        }
        let identity = goods.identity();
        let amount = goods.amount();
        let (position, replaced) = self.goods.insert_full(identity.ex_id, goods);
        Ok(AmountLimitGoodsAdded {
            owner_type: self.base.owner_type(),
            owner_id: self.base.owner_id(),
            position: Some(position as u32),
            identity,
            amount,
            listeners: self.base.base().listener_snapshot(),
            replaced,
        })
    }

    /// Storage-часть exact `Remove(CGUID)`: locked object не извлекается;
    /// listener traversal и message assembly остаются в returned report.
    pub fn remove_goods(&mut self, ex_id: CGuid) -> Option<AmountLimitGoodsRemoved> {
        if self.is_locked(ex_id) {
            return None;
        }
        let (position, _, goods) = self.goods.shift_remove_full(&ex_id)?;
        let amount = goods.amount();
        Some(AmountLimitGoodsRemoved {
            owner_type: self.base.owner_type(),
            owner_id: self.base.owner_id(),
            position: Some(position as u32),
            amount,
            listeners: self.base.base().listener_snapshot(),
            goods,
        })
    }

    pub fn merge_goods_at(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> GoodsStackMergeOutcome {
        if self.goods_amount_limit <= position {
            return GoodsStackMergeOutcome::Incompatible;
        }
        let Some(ex_id) = self
            .goods
            .get_index(position as usize)
            .map(|(_, goods)| goods.identity().ex_id)
        else {
            return GoodsStackMergeOutcome::Incompatible;
        };
        self.merge_goods_by_ex_id(ex_id, incoming, factory, owner_progress_allows)
    }

    pub fn merge_goods_by_ex_id(
        &mut self,
        ex_id: CGuid,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> GoodsStackMergeOutcome {
        if self.is_locked(ex_id) {
            return GoodsStackMergeOutcome::Incompatible;
        }
        let Some(target) = self.goods.get_mut(&ex_id) else {
            return GoodsStackMergeOutcome::Incompatible;
        };
        self.base
            .merge_stack(target, incoming, factory, owner_progress_allows)
    }

    /// Exact `Remove(position, amount)` core: amount `0` ничего не делает,
    /// partial remove разрешён только stackable goods, full remove требует
    /// точного равенства. Player packet checks до/после принадлежат dispatcher-у
    /// и получают owner/base index из отчёта.
    pub fn take_goods<Create>(
        &mut self,
        position: u32,
        requested_amount: u32,
        factory: &CGoodsFactory,
        create_goods: Create,
    ) -> Option<AmountLimitGoodsTaken>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        if requested_amount == 0 {
            return None;
        }
        let ex_id = self.get_goods(position)?.identity().ex_id;
        self.take_goods_by_ex_id(ex_id, position, requested_amount, factory, create_goods)
    }

    pub fn take_goods_by_ex_id<Create>(
        &mut self,
        ex_id: CGuid,
        position: u32,
        requested_amount: u32,
        factory: &CGoodsFactory,
        mut create_goods: Create,
    ) -> Option<AmountLimitGoodsTaken>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        if requested_amount == 0 {
            return None;
        }
        let target = self.find(ex_id)?;
        let base_properties_index = target.base_properties_index();
        let current_amount = target.amount();
        if current_amount < requested_amount {
            return None;
        }
        if current_amount == requested_amount {
            let mut removed = self.remove_goods(ex_id)?;
            removed.position = Some(position);
            return Some(AmountLimitGoodsTaken::Removed(removed));
        }
        if target.max_stack_number(factory) <= 1 {
            return None;
        }

        let mut split = create_goods(base_properties_index)?;
        split.copy_addon_properties_core_from(target);
        split.set_amount(requested_amount);
        let source = target.identity();
        self.goods
            .get_mut(&ex_id)
            .expect("target GUID проверен до создания split goods")
            .set_amount(current_amount.wrapping_sub(requested_amount));
        Some(AmountLimitGoodsTaken::Split(AmountLimitGoodsSplit {
            owner_type: self.base.owner_type(),
            owner_id: self.base.owner_id(),
            position: Some(position),
            base_properties_index,
            source,
            amount: requested_amount,
            listeners: self.base.base().listener_snapshot(),
            goods: split,
        }))
    }

    /// `Clear` сохраняет insertion order listener-событий, отделяет все
    /// objects и очищает locks, не меняя owner, mode или limit.
    pub fn clear_goods(&mut self) -> AmountLimitGoodsCleared {
        let owner_type = self.base.owner_type();
        let owner_id = self.base.owner_id();
        let listeners = self.base.base().listener_snapshot();
        let goods = std::mem::take(&mut self.goods);
        self.locked_goods.clear();
        let removed = goods
            .into_values()
            .enumerate()
            .map(|(position, goods)| AmountLimitGoodsRemoved {
                owner_type,
                owner_id,
                position: Some(position as u32),
                amount: goods.amount(),
                listeners: listeners.clone(),
                goods,
            })
            .collect();
        AmountLimitGoodsCleared { removed }
    }

    /// `Release` не вызывает listener callbacks. Normal mode уничтожает
    /// owned goods, а test mode только отделяет их от container-а.
    pub fn release(&mut self) -> AmountLimitGoodsRelease {
        self.goods_amount_limit = 1;
        let mode = self.base.container_mode();
        let goods = std::mem::take(&mut self.goods)
            .into_values()
            .collect::<Vec<_>>();
        self.locked_goods.clear();
        self.base.release();
        match mode {
            GoodsContainerMode::Normal => AmountLimitGoodsRelease::Collected { count: goods.len() },
            GoodsContainerMode::Test => AmountLimitGoodsRelease::Detached { goods },
        }
    }

    /// Exact amount-container persistence wire: число только известных
    /// factory goods, затем каждый полный `CGoods` в insertion order.
    pub fn serialize(&self, destination: &mut Vec<u8>, factory: &CGoodsFactory) -> bool {
        let count = self.goods_amount(factory);
        LegacyWriter::new(destination).write_u32(count);
        for goods in self.goods.values() {
            if factory
                .query_goods_base_properties(goods.base_properties_index())
                .is_some()
                && !goods.serialize(destination, true)
            {
                return false;
            }
        }
        true
    }

    /// Exact `Unserialize`: прежние objects очищаются до чтения count;
    /// каждый goods восстанавливает derived fairy state из тех же setup
    /// owners, после чего проходит обычный amount-limit storage owner.
    pub fn unserialize<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        factory: &CGoodsFactory,
        mut ordinary_threshold: OrdinaryThreshold,
        mut battle_threshold: BattleThreshold,
    ) -> Result<(), AmountLimitGoodsCodecError>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        let _cleared = self.clear_goods();
        let count = read_amount_wire_u32(source, cursor, "goods count")?;
        if self.goods_amount_limit < count {
            return Err(AmountLimitGoodsCodecError::ContainerLimitExceeded {
                count,
                limit: self.goods_amount_limit,
            });
        }
        for _ in 0..count {
            let mut goods = CGoods::default();
            goods.unserialize(
                source,
                cursor,
                true,
                factory,
                &mut ordinary_threshold,
                &mut battle_threshold,
            )?;
            let _added = self.add_goods(goods, factory)
                .map_err(|_| AmountLimitGoodsCodecError::ContainerLimitExceeded {
                    count,
                    limit: self.goods_amount_limit,
                })?;
        }
        Ok(())
    }
}

fn read_amount_wire_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, AmountLimitGoodsCodecError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        AmountLimitGoodsCodecError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader.read_u32().map_err(|block| {
        AmountLimitGoodsCodecError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        }
    })?;
    *cursor = reader.position();
    Ok(value)
}
