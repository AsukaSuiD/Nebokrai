//! Однослотовое currency/storage core `CWallet` исторического GameServer.
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
//! GameSave owner-ом и одинаково обслуживает wallet/YuanBao/JiFen.

use std::marker::PhantomData;

use super::ccontainer::ContainerListenerHandle;
use super::cgoodscontainer::{CGoodsContainer, GoodsContainerMode, GoodsStackMergeOutcome};
use crate::gameserver::appserver::goods::cgoods::{CGoods, GoodsDecodeError};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_GOODS_STACKING_LIMIT;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::public::guid::CGuid;
use thiserror::Error;

pub(crate) trait CurrencyKind {
    const VALIDATE_EMPTY_GOODS: bool;

    fn goods_index(factory: &CGoodsFactory) -> u32;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoldCoinCurrency;

impl CurrencyKind for GoldCoinCurrency {
    const VALIDATE_EMPTY_GOODS: bool = true;

    fn goods_index(factory: &CGoodsFactory) -> u32 {
        factory.get_gold_coin_index()
    }
}

#[must_use = "report содержит обязательные listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CurrencyGoodsAdded {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: u32,
    pub(crate) identity: ShapeIdentity,
    pub(crate) amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "report содержит owned goods и обязательные listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CurrencyGoodsRemoved {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: u32,
    pub(crate) amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
    pub(crate) goods: CGoods,
}

#[must_use = "report сохраняет split ownership и listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CurrencyGoodsSplit {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: u32,
    pub(crate) source: ShapeIdentity,
    pub(crate) amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
    pub(crate) goods: CGoods,
}

#[must_use = "результат take владеет отделённой currency goods"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CurrencyGoodsTaken {
    Removed(CurrencyGoodsRemoved),
    Split(CurrencyGoodsSplit),
}

#[must_use = "report подтверждает уничтожение прежнего currency goods"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CurrencyGoodsCollected {
    pub(crate) count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CurrencyAmountChanged {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: u32,
    pub(crate) identity: ShapeIdentity,
    pub(crate) amount: u32,
    pub(crate) new_amount: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CurrencyGoodsAddBlock {
    MissingGoods,
    InvalidCurrency {
        expected: u32,
        actual: u32,
        position: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum CurrencyCodecError {
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
pub(crate) enum CurrencyGoodsAddOutcome {
    Added(CurrencyGoodsAdded),
    Stack(GoodsStackMergeOutcome),
    Rejected(CurrencyGoodsAddBlock),
}

#[must_use = "результат decrease определяет ownership и message-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CurrencyDecreaseOutcome {
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
pub(crate) enum CurrencyIncreaseOutcome {
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
pub(crate) struct CSingleCurrencyContainer<K> {
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
    pub(crate) const fn new() -> Self {
        Self {
            base: CGoodsContainer::new(),
            goods: None,
            kind: PhantomData,
        }
    }

    pub(crate) const fn base(&self) -> &CGoodsContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CGoodsContainer {
        &mut self.base
    }

    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.base.set_owner(owner_type, owner_id);
    }

    pub(crate) const fn set_container_mode(&mut self, mode: GoodsContainerMode) {
        self.base.set_container_mode(mode);
    }

    pub(crate) const fn goods(&self) -> Option<&CGoods> {
        self.goods.as_ref()
    }
}

impl<K: CurrencyKind> CSingleCurrencyContainer<K> {
    pub(crate) fn currency_goods_index(&self, factory: &CGoodsFactory) -> u32 {
        K::goods_index(factory)
    }

    pub(crate) fn is_goods_existed(
        &self,
        base_properties_index: u32,
        factory: &CGoodsFactory,
    ) -> bool {
        self.goods.is_some() && base_properties_index == K::goods_index(factory)
    }

    pub(crate) fn get_first_goods(
        &self,
        base_properties_index: u32,
        factory: &CGoodsFactory,
    ) -> Option<&CGoods> {
        (base_properties_index == K::goods_index(factory))
            .then_some(self.goods.as_ref())
            .flatten()
    }

    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        (position == 0).then_some(self.goods.as_ref()).flatten()
    }

    pub(crate) fn query_goods_position(&self, ex_id: CGuid) -> Option<u32> {
        self.goods
            .as_ref()
            .filter(|goods| goods.identity().ex_id == ex_id)
            .map(|_| 0)
    }

    pub(crate) fn find(&self, ex_id: CGuid) -> Option<&CGoods> {
        self.goods
            .as_ref()
            .filter(|goods| goods.identity().ex_id == ex_id)
    }

    pub(crate) fn goods_amount(&self) -> u32 {
        u32::from(self.goods.is_some())
    }

    pub(crate) fn currency_amount(&self) -> u32 {
        self.goods.as_ref().map_or(0, CGoods::amount)
    }

    pub(crate) fn max_stack_number(&self, factory: &CGoodsFactory) -> u32 {
        if let Some(goods) = &self.goods {
            return goods.max_stack_number(factory);
        }
        factory
            .query_goods_base_properties(K::goods_index(factory))
            .map_or(0, |properties| {
                properties.get_addon_property_value(GAP_GOODS_STACKING_LIMIT, true) as u32
            })
    }

    pub(crate) fn is_full(&self, factory: &CGoodsFactory) -> bool {
        self.goods
            .as_ref()
            .is_some_and(|goods| goods.max_stack_number(factory) <= goods.amount())
    }

    pub(crate) fn add_goods(
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
            listeners: self.base.base().listeners().to_vec(),
        })
    }

    pub(crate) fn remove_goods(&mut self, ex_id: CGuid) -> Option<CurrencyGoodsRemoved> {
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
            listeners: self.base.base().listeners().to_vec(),
            goods,
        })
    }

    /// Exact positional `Remove(0, amount)` для container-move: full remove
    /// передаёт исходный object, partial создаёт новый GUID/owner и уменьшает
    /// stored currency только после успешного создания split.
    pub(crate) fn take_goods<Create>(
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
            listeners: self.base.base().listeners().to_vec(),
            goods: split,
        }))
    }

    /// Exact owner не проверяет `requested <= current`: unsigned subtraction
    /// в partial-ветке намеренно wrapping.
    pub(crate) fn decrease_currency(
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

    pub(crate) fn increase_currency<Create>(
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

    pub(crate) fn clear_goods(&mut self) -> CurrencyGoodsCollected {
        CurrencyGoodsCollected {
            count: usize::from(self.goods.take().is_some()),
        }
    }

    pub(crate) fn release(&mut self) -> CurrencyGoodsCollected {
        self.base.release();
        CurrencyGoodsCollected {
            count: usize::from(self.goods.take().is_some()),
        }
    }

    /// Wallet/YuanBao/JiFen persistence marker: `0` либо `1` и полный goods.
    pub(crate) fn serialize(&self, destination: &mut Vec<u8>) -> bool {
        let Some(goods) = &self.goods else {
            destination.push(0);
            return true;
        };
        destination.push(1);
        goods.serialize(destination, true)
    }

    pub(crate) fn unserialize<OrdinaryThreshold, BattleThreshold>(
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
        let _released = self.release();
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

pub(crate) type CWallet = CSingleCurrencyContainer<GoldCoinCurrency>;

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cwallet.cpp

// ============================================================================
// FUNCTION: CWallet::IsGoodsExisted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cwallet.cpp:186
// RVA: 0x000D5C80
// ADDRESS: 004d5c80
// PROTOTYPE: int __thiscall IsGoodsExisted(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::GetTheFirstGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cwallet.cpp:199
// RVA: 0x000D5CA0
// ADDRESS: 004d5ca0
// PROTOTYPE: CGoods * __thiscall GetTheFirstGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::CWallet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cwallet.cpp:19
// RVA: 0x000D5D60
// ADDRESS: 004d5d60
// PROTOTYPE: undefined __thiscall CWallet(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::~CWallet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cwallet.cpp:35
// RVA: 0x000D5DC0
// ADDRESS: 004d5dc0
// PROTOTYPE: void __thiscall ~CWallet(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// `DecreaseGoldCoins` материализован как `decrease_currency`: exact delete при
// равном amount, wrapping partial subtraction, identity/owner/position и
// outcome для `0xC0101/OT_DELETE_OBJECT` либо `OT_MOVE_OBJECT` сохранены.

// ============================================================================
// FUNCTION: CWallet::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cwallet.cpp:56
// RVA: 0x000D5FC0
// ADDRESS: 004d5fc0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, tagPreviousContainer * param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cwallet.cpp:42
// RVA: 0x000D60E0
// ADDRESS: 004d60e0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, tagPreviousContainer * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::AddGoldCoins
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cwallet.cpp:371
// RVA: 0x000D6130
// ADDRESS: 004d6130
// PROTOTYPE: int __thiscall AddGoldCoins(ulong param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\container\cwallet.cpp:222
// RVA: 0x000D6350
// ADDRESS: 004d6350
// PROTOTYPE: void __thiscall GetGoods(ulong param_1, vector<CGoods*,std::allocator<CGoods*>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
