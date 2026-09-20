//! Однослотовая currency-проекция `CShadowWallet` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cshadowwallet.cpp`. Shadow не
//! владеет Gold: он перемещает incoming goods в канонический
//! wallet player-а type `400`, extend `4`, а затем хранит только
//! source metadata и amount до merge.
//!
//! Порядок частичных эффектов сохранён буквально: null/неверный
//! catalog не меняет старую запись; корректный catalog сначала
//! публикует `RemoveShadow` для первой записи и очищает map, и только
//! потом проверяет previous/source. Поэтом отказ после этой точки
//! оставляет projection пустой. Успешный derived add пишет map
//! напрямую и не перепроверяет count-limit.
//!
//! `Option`, RAII и caller-supplied source wallet заменяют nullable pointer,
//! `CGame::FindPlayer` и MSVC map. Packet/log dispatch остаётся внешней
//! границей typed outcome до materialization соответствующих caller-ов.

use std::marker::PhantomData;

use super::camountlimitgoodsshadowcontainer::{
    AmountShadowAdded, CAmountLimitGoodsShadowContainer,
};
use super::ccontainer::PreviousContainer;
use super::cgoodscontainer::GoodsStackMergeOutcome;
use super::cgoodsshadowcontainer::{PlacedShadowGoods, ShadowRemovedReport};
use super::cwallet::{
    CSingleCurrencyContainer, CurrencyGoodsAddOutcome, CurrencyKind, GoldCoinCurrency,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use crate::public::guid::CGuid;

const PLAYER_CONTAINER_OWNER_TYPE: i32 = 400;

pub(crate) trait ShadowCurrencyKind: CurrencyKind {
    const SOURCE_CONTAINER_EXTEND_ID: i32;
}

impl ShadowCurrencyKind for GoldCoinCurrency {
    const SOURCE_CONTAINER_EXTEND_ID: i32 = 4;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShadowCurrencyAddBlock {
    MissingGoods,
    InvalidCurrency {
        expected: u32,
        actual: u32,
        position: u32,
    },
    MissingPreviousContainer,
    InvalidSourceOwner {
        actual: i32,
    },
    InvalidSourceContainer {
        expected: i32,
        actual: i32,
    },
    SourcePlayerNotFound {
        player_id: i32,
    },
    SourceAddRejected,
}

#[must_use = "outcome содержит необратимые source/listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ShadowCurrencyAddOutcome {
    Added {
        removed_shadow: Option<ShadowRemovedReport>,
        source_add: CurrencyGoodsAddOutcome,
        shadow: AmountShadowAdded,
    },
    Rejected {
        block: ShadowCurrencyAddBlock,
        removed_shadow: Option<ShadowRemovedReport>,
        source_add: Option<CurrencyGoodsAddOutcome>,
    },
}

#[must_use = "callback всегда accepted, но report может отсутствовать"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShadowCurrencyRemovedCallback {
    pub(crate) accepted: bool,
    pub(crate) removed: Option<ShadowRemovedReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CShadowCurrencyContainer<K> {
    base: CAmountLimitGoodsShadowContainer,
    kind: PhantomData<fn() -> K>,
}

impl<K: ShadowCurrencyKind> Default for CShadowCurrencyContainer<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: ShadowCurrencyKind> CShadowCurrencyContainer<K> {
    pub(crate) const fn new() -> Self {
        let mut base = CAmountLimitGoodsShadowContainer::new();
        base.set_goods_amount_limit(1);
        Self {
            base,
            kind: PhantomData,
        }
    }

    pub(crate) const fn base(&self) -> &CAmountLimitGoodsShadowContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CAmountLimitGoodsShadowContainer {
        &mut self.base
    }

    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.base
            .base_mut()
            .base_mut()
            .set_owner(owner_type, owner_id);
    }

    pub(crate) const fn set_container_extend_id(&mut self, extend_id: i32) {
        self.base.base_mut().set_container_extend_id(extend_id);
    }

    /// Exact `GetGoldCoinsAmount` читает snapshot amount первой
    /// GUID-ordered shadow-записи, а не текущий source balance.
    pub(crate) fn currency_amount(&self) -> u32 {
        self.base
            .base()
            .shadows()
            .values()
            .next()
            .map_or(0, |shadow| shadow.goods_amount)
    }

    /// `CShadowWallet::OnObjectAdded` намеренно не обновляет amount.
    pub(crate) const fn on_source_added(&mut self) -> bool {
        false
    }

    /// Exact override удаляет shadow целиком при любом remove
    /// source-object и возвращает true даже для null/missing.
    pub(crate) fn on_source_removed(
        &mut self,
        goods_id: Option<CGuid>,
    ) -> ShadowCurrencyRemovedCallback {
        ShadowCurrencyRemovedCallback {
            accepted: true,
            removed: goods_id.and_then(|goods_id| self.base.base_mut().remove_shadow(goods_id)),
        }
    }

    pub(crate) fn add_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        previous: Option<PreviousContainer>,
        source_player: Option<&mut CSingleCurrencyContainer<K>>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> ShadowCurrencyAddOutcome {
        let Some(goods) = incoming.as_ref() else {
            return Self::rejected(ShadowCurrencyAddBlock::MissingGoods, None, None);
        };
        let expected = K::goods_index(factory);
        let actual = goods.base_properties_index();
        if actual != expected {
            return Self::rejected(
                ShadowCurrencyAddBlock::InvalidCurrency {
                    expected,
                    actual,
                    position,
                },
                None,
                None,
            );
        }

        let amount = goods.amount();
        let removed_shadow = self.remove_first_shadow_then_clear();
        let Some(previous) = previous else {
            return Self::rejected(
                ShadowCurrencyAddBlock::MissingPreviousContainer,
                removed_shadow,
                None,
            );
        };
        if previous.container_type != PLAYER_CONTAINER_OWNER_TYPE {
            return Self::rejected(
                ShadowCurrencyAddBlock::InvalidSourceOwner {
                    actual: previous.container_type,
                },
                removed_shadow,
                None,
            );
        }
        if previous.container_extend_id != K::SOURCE_CONTAINER_EXTEND_ID {
            return Self::rejected(
                ShadowCurrencyAddBlock::InvalidSourceContainer {
                    expected: K::SOURCE_CONTAINER_EXTEND_ID,
                    actual: previous.container_extend_id,
                },
                removed_shadow,
                None,
            );
        }
        // Caller уже выполнил exact `FindPlayer(previous.container_id)`;
        // `None` означает тот же lookup miss без хранения глобального `CGame`.
        let Some(source) = source_player else {
            return Self::rejected(
                ShadowCurrencyAddBlock::SourcePlayerNotFound {
                    player_id: previous.container_id,
                },
                removed_shadow,
                None,
            );
        };

        let source_position = if previous.goods_position == u32::MAX {
            0
        } else {
            previous.goods_position
        };
        let source_add =
            source.add_goods(source_position, incoming, factory, owner_progress_allows);
        let placed_identity = match &source_add {
            CurrencyGoodsAddOutcome::Added(added) => added.identity.ex_id,
            CurrencyGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged { target, .. }) => {
                target.ex_id
            }
            CurrencyGoodsAddOutcome::Stack(
                GoodsStackMergeOutcome::BlockedByOwnerProgress
                | GoodsStackMergeOutcome::Incompatible,
            )
            | CurrencyGoodsAddOutcome::Rejected(_) => {
                return Self::rejected(
                    ShadowCurrencyAddBlock::SourceAddRejected,
                    removed_shadow,
                    Some(source_add),
                );
            }
        };

        let placed = PlacedShadowGoods {
            identity: placed_identity,
            position: 0,
            base_properties_index: actual,
            amount,
        };
        // Derived owner писал `map[guid]` напрямую: этот add не
        // зависит от max=1, который `Release` сбрасывает в 0.
        let recorded = self
            .base
            .base_mut()
            .record_placed_goods(previous, placed, true)
            .expect("currency previous проверен до shadow record");
        let presence = self
            .base
            .base()
            .add_shadow_report(recorded.record.goods_id)
            .expect("записанный currency shadow обязан существовать");
        ShadowCurrencyAddOutcome::Added {
            removed_shadow,
            source_add,
            shadow: AmountShadowAdded { recorded, presence },
        }
    }

    pub(crate) fn clear(&mut self) -> usize {
        self.base.clear()
    }

    pub(crate) fn release(&mut self) -> usize {
        self.base.release()
    }

    fn remove_first_shadow_then_clear(&mut self) -> Option<ShadowRemovedReport> {
        let first = self.base.base().shadows().keys().next().copied();
        let removed = first.and_then(|goods_id| self.base.base_mut().remove_shadow(goods_id));
        self.base.clear();
        removed
    }

    fn rejected(
        block: ShadowCurrencyAddBlock,
        removed_shadow: Option<ShadowRemovedReport>,
        source_add: Option<CurrencyGoodsAddOutcome>,
    ) -> ShadowCurrencyAddOutcome {
        ShadowCurrencyAddOutcome::Rejected {
            block,
            removed_shadow,
            source_add,
        }
    }
}

pub(crate) type CShadowWallet = CShadowCurrencyContainer<GoldCoinCurrency>;
