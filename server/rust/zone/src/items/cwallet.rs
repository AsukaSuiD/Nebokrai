//! Однослотовое currency/storage core `CWallet` исторического GameServer:
//! владеет одним `CGoods` catalog `MONEY`, а не числовым balance; generic
//! core также обслуживает `CYuanBao` и `CJiFen`.
//!
//! Player-владелец публикует этот контейнер под extend-id
//! [`PlayerContainerKind::Wallet`][crate::items::playercontainers::PlayerContainerKind]
//! единого каталога (дизайн D4).
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cwallet.cpp`. Wallet владеет не
//! числовым balance, а одним `CGoods` catalog `MONEY`; `Option<CGoods>` и RAII
//! заменяют nullable pointer и `GarbageCollect`. Общий generic core также
//! используется точным двойником `CYuanBao`, который отличается только
//! factory-index-ом.
//!
//! Query, add/stack, remove, positional full/partial take, direct
//! increase/decrease и lifecycle возвращают
//! typed reports для будущего listener/message dispatcher-а. Exact decrease
//! при запросе больше balance выполняет unsigned wrapping subtraction — это
//! наблюдаемый legacy-контракт, а не внутренний pointer-дефект. Достигнутый
//! battle-fairy и ground currency callers собирают из outcomes точный
//! `CS2CContainerObjectMove`; increase/create публикации остаются за своими
//! ещё отдельными сценариями.
//! Marker + optional full-goods persisted codec достигнут общим player
//! GameSave owner-ом и одинаково обслуживает wallet/YuanBao/JiFen. Restore
//! очищает только прежний goods: owner, mode и listener-set принадлежат
//! container lifecycle и не освобождаются при `Unserialize`.

use std::marker::PhantomData;

use super::ccontainer::ContainerListenerHandle;
use super::cgoods::{CGoods, GoodsDecodeError};
use super::cgoodscontainer::{CGoodsContainer, GoodsContainerMode, GoodsStackMergeOutcome};
use crate::content::goods::GAP_GOODS_STACKING_LIMIT;
use crate::content::goodsfactory::CGoodsFactory;
use crate::regions::ShapeIdentity;
use nebokrai_shared::values::CGuid;
use thiserror::Error;

pub trait CurrencyKind {
    const VALIDATE_EMPTY_GOODS: bool;

    fn goods_index(factory: &CGoodsFactory) -> u32;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoldCoinCurrency;

impl CurrencyKind for GoldCoinCurrency {
    const VALIDATE_EMPTY_GOODS: bool = true;

    fn goods_index(factory: &CGoodsFactory) -> u32 {
        factory.get_gold_coin_index()
    }
}

#[must_use = "report содержит обязательные listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyGoodsAdded {
    pub owner_type: i32,
    pub owner_id: i32,
    pub position: u32,
    pub identity: ShapeIdentity,
    pub amount: u32,
    pub listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "report содержит owned goods и обязательные listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyGoodsRemoved {
    pub owner_type: i32,
    pub owner_id: i32,
    pub position: u32,
    pub amount: u32,
    pub listeners: Vec<ContainerListenerHandle>,
    pub goods: CGoods,
}

#[must_use = "report сохраняет split ownership и listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyGoodsSplit {
    pub owner_type: i32,
    pub owner_id: i32,
    pub position: u32,
    pub source: ShapeIdentity,
    pub amount: u32,
    pub listeners: Vec<ContainerListenerHandle>,
    pub goods: CGoods,
}

#[must_use = "результат take владеет отделённой currency goods"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyGoodsTaken {
    Removed(CurrencyGoodsRemoved),
    Split(CurrencyGoodsSplit),
}

#[must_use = "report подтверждает уничтожение прежнего currency goods"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyGoodsCollected {
    pub count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyAmountChanged {
    pub owner_type: i32,
    pub owner_id: i32,
    pub position: u32,
    pub identity: ShapeIdentity,
    pub amount: u32,
    pub new_amount: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencyGoodsAddBlock {
    MissingGoods,
    InvalidCurrency {
        expected: u32,
        actual: u32,
        position: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CurrencyCodecError {
    #[error(transparent)]
    Goods(#[from] GoodsDecodeError),
    #[error("currency container обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[must_use = "результат add определяет ownership и listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyGoodsAddOutcome {
    Added(CurrencyGoodsAdded),
    Stack(GoodsStackMergeOutcome),
    Rejected(CurrencyGoodsAddBlock),
}

#[must_use = "результат decrease определяет ownership и message-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyDecreaseOutcome {
    NoChange,
    InvalidStoredCurrency {
        expected: u32,
        actual: u32,
        requested: u32,
    },
    Decreased(CurrencyAmountChanged),
    Removed(CurrencyGoodsRemoved),
}

#[must_use = "результат increase определяет ownership и message-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyIncreaseOutcome {
    NoChange,
    InvalidStoredCurrency {
        expected: u32,
        actual: u32,
        requested: u32,
    },
    Increased(CurrencyAmountChanged),
    Created(CurrencyGoodsAdded),
    CapacityExceeded {
        current: u32,
        requested: u32,
        maximum: u32,
    },
    CreationFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CSingleCurrencyContainer<K> {
    base: CGoodsContainer,
    goods: Option<CGoods>,
    kind: PhantomData<fn() -> K>,
}

impl<K> Default for CSingleCurrencyContainer<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> CSingleCurrencyContainer<K> {
    pub const fn new() -> Self {
        Self {
            base: CGoodsContainer::new(),
            goods: None,
            kind: PhantomData,
        }
    }

    pub const fn base(&self) -> &CGoodsContainer {
        &self.base
    }

    pub const fn base_mut(&mut self) -> &mut CGoodsContainer {
        &mut self.base
    }

    pub const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.base.set_owner(owner_type, owner_id);
    }

    pub const fn set_container_mode(&mut self, mode: GoodsContainerMode) {
        self.base.set_container_mode(mode);
    }

    pub const fn goods(&self) -> Option<&CGoods> {
        self.goods.as_ref()
    }
}

impl<K: CurrencyKind> CSingleCurrencyContainer<K> {
    pub fn currency_goods_index(&self, factory: &CGoodsFactory) -> u32 {
        K::goods_index(factory)
    }

    pub fn is_goods_existed(
        &self,
        base_properties_index: u32,
        factory: &CGoodsFactory,
    ) -> bool {
        self.goods.is_some() && base_properties_index == K::goods_index(factory)
    }

    pub fn get_first_goods(
        &self,
        base_properties_index: u32,
        factory: &CGoodsFactory,
    ) -> Option<&CGoods> {
        (base_properties_index == K::goods_index(factory))
            .then_some(self.goods.as_ref())
            .flatten()
    }

    pub fn get_goods(&self, position: u32) -> Option<&CGoods> {
        (position == 0).then_some(self.goods.as_ref()).flatten()
    }

    pub fn query_goods_position(&self, ex_id: CGuid) -> Option<u32> {
        self.goods
            .as_ref()
            .filter(|goods| goods.identity().ex_id == ex_id)
            .map(|_| 0)
    }

    pub fn find(&self, ex_id: CGuid) -> Option<&CGoods> {
        self.goods
            .as_ref()
            .filter(|goods| goods.identity().ex_id == ex_id)
    }

    pub fn goods_amount(&self) -> u32 {
        u32::from(self.goods.is_some())
    }

    pub fn currency_amount(&self) -> u32 {
        self.goods.as_ref().map_or(0, CGoods::amount)
    }

    pub fn max_stack_number(&self, factory: &CGoodsFactory) -> u32 {
        if let Some(goods) = &self.goods {
            return goods.max_stack_number(factory);
        }
        factory
            .query_goods_base_properties(K::goods_index(factory))
            .map_or(0, |properties| {
                properties.get_addon_property_value(GAP_GOODS_STACKING_LIMIT, true) as u32
            })
    }

    pub fn is_full(&self, factory: &CGoodsFactory) -> bool {
        self.goods
            .as_ref()
            .is_some_and(|goods| goods.max_stack_number(factory) <= goods.amount())
    }

    pub fn add_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> CurrencyGoodsAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return CurrencyGoodsAddOutcome::Rejected(CurrencyGoodsAddBlock::MissingGoods);
        };
        let expected = K::goods_index(factory);
        let actual = goods.base_properties_index();
        if actual != expected && (K::VALIDATE_EMPTY_GOODS || self.goods.is_some()) {
            return CurrencyGoodsAddOutcome::Rejected(CurrencyGoodsAddBlock::InvalidCurrency {
                expected,
                actual,
                position,
            });
        }
        if let Some(stored) = &mut self.goods {
            return CurrencyGoodsAddOutcome::Stack(self.base.merge_stack(
                stored,
                incoming,
                factory,
                owner_progress_allows,
            ));
        }

        let goods = incoming.take().expect("incoming проверен до wallet add");
        let identity = goods.identity();
        let amount = goods.amount();
        self.goods = Some(goods);
        CurrencyGoodsAddOutcome::Added(CurrencyGoodsAdded {
            owner_type: self.base.owner_type(),
            owner_id: self.base.owner_id(),
            position: 0,
            identity,
            amount,
            listeners: self.base.base().listener_snapshot(),
        })
    }

    pub fn remove_goods(&mut self, ex_id: CGuid) -> Option<CurrencyGoodsRemoved> {
        let stored = self.goods.as_ref()?;
        if stored.identity().ex_id != ex_id {
            return None;
        }
        let goods = self.goods.take().expect("wallet GUID проверен до remove");
        Some(CurrencyGoodsRemoved {
            owner_type: self.base.owner_type(),
            owner_id: self.base.owner_id(),
            position: 0,
            amount: goods.amount(),
            listeners: self.base.base().listener_snapshot(),
            goods,
        })
    }

    /// Exact positional `Remove(0, amount)` для container-move: full remove
    /// передаёт исходный object, partial создаёт новый GUID/owner и уменьшает
    /// stored currency только после успешного создания split.
    pub fn take_goods<Create>(
        &mut self,
        position: u32,
        requested: u32,
        factory: &CGoodsFactory,
        mut create_goods: Create,
    ) -> Option<CurrencyGoodsTaken>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        if position != 0 || requested == 0 {
            return None;
        }
        let stored = self.goods.as_ref()?;
        if stored.amount() < requested {
            return None;
        }
        if stored.amount() == requested {
            return self
                .remove_goods(stored.identity().ex_id)
                .map(CurrencyGoodsTaken::Removed);
        }
        if stored.max_stack_number(factory) <= 1 {
            return None;
        }
        let mut split = create_goods(stored.base_properties_index())?;
        split.copy_addon_properties_core_from(stored);
        split.set_amount(requested);
        let source = stored.identity();
        let remaining = stored.amount().wrapping_sub(requested);
        self.goods
            .as_mut()
            .expect("currency проверена до split")
            .set_amount(remaining);
        Some(CurrencyGoodsTaken::Split(CurrencyGoodsSplit {
            owner_type: self.base.owner_type(),
            owner_id: self.base.owner_id(),
            position,
            source,
            amount: requested,
            listeners: self.base.base().listener_snapshot(),
            goods: split,
        }))
    }

    /// Exact owner не проверяет `requested <= current`: unsigned subtraction
    /// в partial-ветке намеренно wrapping.
    pub fn decrease_currency(
        &mut self,
        requested: u32,
        factory: &CGoodsFactory,
    ) -> CurrencyDecreaseOutcome {
        let expected = K::goods_index(factory);
        let Some(stored) = self.goods.as_ref() else {
            return CurrencyDecreaseOutcome::NoChange;
        };
        let actual = stored.base_properties_index();
        if actual != expected {
            return CurrencyDecreaseOutcome::InvalidStoredCurrency {
                expected,
                actual,
                requested,
            };
        }
        if requested == 0 {
            return CurrencyDecreaseOutcome::NoChange;
        }
        if stored.amount() == requested {
            let ex_id = stored.identity().ex_id;
            return CurrencyDecreaseOutcome::Removed(
                self.remove_goods(ex_id)
                    .expect("wallet goods проверен перед exact decrease"),
            );
        }

        let stored = self.goods.as_mut().expect("wallet goods проверен");
        let new_amount = stored.amount().wrapping_sub(requested);
        stored.set_amount(new_amount);
        CurrencyDecreaseOutcome::Decreased(CurrencyAmountChanged {
            owner_type: self.base.owner_type(),
            owner_id: self.base.owner_id(),
            position: 0,
            identity: stored.identity(),
            amount: requested,
            new_amount,
        })
    }

    pub fn increase_currency<Create>(
        &mut self,
        requested: u32,
        factory: &CGoodsFactory,
        create_goods: Create,
    ) -> CurrencyIncreaseOutcome
    where
        Create: FnOnce(u32, u32) -> Vec<CGoods>,
    {
        let expected = K::goods_index(factory);
        if let Some(stored) = self.goods.as_ref() {
            let actual = stored.base_properties_index();
            if actual != expected {
                return CurrencyIncreaseOutcome::InvalidStoredCurrency {
                    expected,
                    actual,
                    requested,
                };
            }
        }
        if requested == 0 {
            return CurrencyIncreaseOutcome::NoChange;
        }
        if let Some(stored) = self.goods.as_mut() {
            let current = stored.amount();
            let maximum = stored.max_stack_number(factory);
            let new_amount = current.wrapping_add(requested);
            if maximum < new_amount {
                return CurrencyIncreaseOutcome::CapacityExceeded {
                    current,
                    requested,
                    maximum,
                };
            }
            stored.set_amount(new_amount);
            return CurrencyIncreaseOutcome::Increased(CurrencyAmountChanged {
                owner_type: self.base.owner_type(),
                owner_id: self.base.owner_id(),
                position: 0,
                identity: stored.identity(),
                amount: requested,
                new_amount,
            });
        }

        for goods in create_goods(expected, requested) {
            let mut incoming = Some(goods);
            if let CurrencyGoodsAddOutcome::Added(added) =
                self.add_goods(0, &mut incoming, factory, true)
            {
                return CurrencyIncreaseOutcome::Created(added);
            }
        }
        CurrencyIncreaseOutcome::CreationFailed
    }

    pub fn clear_goods(&mut self) -> CurrencyGoodsCollected {
        CurrencyGoodsCollected {
            count: usize::from(self.goods.take().is_some()),
        }
    }

    pub fn release(&mut self) -> CurrencyGoodsCollected {
        self.base.release();
        CurrencyGoodsCollected {
            count: usize::from(self.goods.take().is_some()),
        }
    }

    /// Wallet/YuanBao/JiFen persistence marker: `0` либо `1` и полный goods.
    pub fn serialize(&self, destination: &mut Vec<u8>) -> bool {
        let Some(goods) = &self.goods else {
            destination.push(0);
            return true;
        };
        destination.push(1);
        goods.serialize(destination, true)
    }

    pub fn unserialize<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        marker_field: &'static str,
        factory: &CGoodsFactory,
        ordinary_threshold: OrdinaryThreshold,
        battle_threshold: BattleThreshold,
    ) -> Result<(), CurrencyCodecError>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        let _cleared = self.clear_goods();
        let offset = *cursor;
        let Some(&marker) = source.get(offset) else {
            return Err(CurrencyCodecError::UnexpectedEnd {
                field: marker_field,
                offset,
                needed: 1,
                available: source.len().saturating_sub(offset),
            });
        };
        *cursor = offset + 1;
        if marker != 0 {
            let mut goods = CGoods::default();
            goods.unserialize(
                source,
                cursor,
                true,
                factory,
                ordinary_threshold,
                battle_threshold,
            )?;
            self.goods = Some(goods);
        }
        Ok(())
    }
}

pub type CWallet = CSingleCurrencyContainer<GoldCoinCurrency>;
