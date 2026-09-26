//! Count-limit слой goods shadow container исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/camountlimitgoodsshadowcontainer.cpp`.
//! Maximum начинается с нуля, поэтому новый container full до явного setup.
//! Успешная запись metadata сразу формирует `AddShadow` report; фактический GUID
//! placement заменяет legacy повторное чтение потенциально уже уничтоженного
//! incoming pointer-а после stack merge.
//!
//! `BTreeMap` и resolver mechanics принадлежат base owner-у. Clone target и
//! packet assembly ещё требуют реконструкции; полный декомпилят хранится локально до их concrete caller-ов.

use super::ccontainer::PreviousContainer;
use super::cgoodsshadowcontainer::{
    CGoodsShadowContainer, PlacedShadowGoods, ShadowPresenceReport, ShadowRecordBlock,
    ShadowRecorded,
};

#[must_use = "успешный add содержит metadata и обязательный AddShadow report"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AmountShadowAdded {
    pub(crate) recorded: ShadowRecorded,
    pub(crate) presence: ShadowPresenceReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CAmountLimitGoodsShadowContainer {
    base: CGoodsShadowContainer,
    max_goods_amount: u32,
}

impl Default for CAmountLimitGoodsShadowContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CAmountLimitGoodsShadowContainer {
    pub(crate) const fn new() -> Self {
        Self {
            base: CGoodsShadowContainer::new(),
            max_goods_amount: 0,
        }
    }

    pub(crate) const fn base(&self) -> &CGoodsShadowContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CGoodsShadowContainer {
        &mut self.base
    }

    pub(crate) const fn set_goods_amount_limit(&mut self, limit: u32) {
        self.max_goods_amount = limit;
    }

    pub(crate) const fn goods_amount_limit(&self) -> u32 {
        self.max_goods_amount
    }

    pub(crate) fn is_full(&self) -> bool {
        self.max_goods_amount <= self.base.goods_amount()
    }

    pub(crate) fn record_placed_goods(
        &mut self,
        previous: PreviousContainer,
        placed: PlacedShadowGoods,
    ) -> Result<AmountShadowAdded, ShadowRecordBlock> {
        let recorded = self
            .base
            .record_placed_goods(previous, placed, !self.is_full())?;
        let presence = self
            .base
            .add_shadow_report(recorded.record.goods_id)
            .expect("записанный amount shadow обязан существовать");
        Ok(AmountShadowAdded { recorded, presence })
    }

    pub(crate) fn clear(&mut self) -> usize {
        self.base.clear()
    }

    pub(crate) fn release(&mut self) -> usize {
        let count = self.base.release();
        self.max_goods_amount = 0;
        count
    }
}
