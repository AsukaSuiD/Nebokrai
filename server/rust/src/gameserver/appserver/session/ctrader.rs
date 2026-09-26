//! Достигнутый owner двусторонней торговли GameServer `CTrader`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/session/ctrader.cpp`. Материализованы три trade-shadow
//! container-а `(goods, Gold, YuanBao)`, ready-state, source metadata и
//! terminal clear. `CSessionFactory` владеет plug-ами, а `CGame` выполняет
//! достигнутую двухфазную проверку и ownership transaction: исходные goods
//! остаются у player до commit, затем переходят в packet второго участника;
//! при частичном отказе они удаляются у получателя и возвращаются в packet
//! владельца, как исходный `RollBack`. Универсальный registry выражен
//! существующими owned maps/containers без отдельного transaction framework.
//!
//! Подтверждённые goods/increment audit и container-listener следствия входят
//! в тот же проход; замещённый RAW в owner-файле не дублируется.
//!
//! Скалярные правила рамки (kind индексов extend-id, приёмка предложения до
//! занятия ячейки, таблица player-контейнеров источника, сверка количества и
//! обратимость stack-merge rollback) перенесены в Zone
//! `trade/ctrader.rs` порцией T и импортируются отсюда как прежние имена.
//! Контейнерные операции `record_offer`/`remove_offer`/`clear` и отчёты
//! `ShadowPresenceReport/ShadowRemovedReport` остаются здесь как
//! container-owner (прецедент `CPersonalShopSeller`).

use crate::gameserver::appserver::container::ccontainer::PreviousContainer;
use crate::gameserver::appserver::container::cgoodsshadowcontainer::{
    GoodsShadow, PlacedShadowGoods, ShadowPresenceReport, ShadowRemovedReport,
};
use crate::gameserver::appserver::container::cshadowwallet::CShadowWallet;
use crate::gameserver::appserver::container::cshadowyuanbao::CShadowYuanBao;
use crate::gameserver::appserver::container::cvolumelimitgoodsshadowcontainer::CVolumeLimitGoodsShadowContainer;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_PARTICULAR_ATTRIBUTE;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use nebokrai_shared::values::CGuid;
use nebokrai_zone::trade::ctrader::{
    SESSION_OWNER_TYPE, TRADE_GOODS_CELLS, trade_container_extend_id,
    trade_currency_offer_block, trade_goods_offer_block, trade_offer_missing_goods,
};

pub(crate) use nebokrai_zone::trade::ctrader::{TraderContainerKind, TraderOfferBlock};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TraderOfferAdded {
    pub(crate) plug_id: i32,
    pub(crate) kind: TraderContainerKind,
    pub(crate) position: u32,
    pub(crate) record: GoodsShadow,
    pub(crate) presence: ShadowPresenceReport,
    pub(crate) replaced: Option<ShadowRemovedReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TraderOfferRemoved {
    pub(crate) plug_id: i32,
    pub(crate) kind: TraderContainerKind,
    pub(crate) position: u32,
    pub(crate) removed: ShadowRemovedReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CTrader {
    plug_id: i32,
    session_id: i32,
    owner_id: i32,
    goods: CVolumeLimitGoodsShadowContainer,
    gold: CShadowWallet,
    yuan_bao: CShadowYuanBao,
    ready: bool,
}

impl CTrader {
    pub(crate) fn inserted(plug_id: i32, session_id: i32, owner_id: i32) -> Self {
        let mut goods = CVolumeLimitGoodsShadowContainer::new();
        goods.set_container_volume(TRADE_GOODS_CELLS);
        goods
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(SESSION_OWNER_TYPE, session_id);
        goods
            .base_mut()
            .base_mut()
            .set_container_extend_id(trade_container_extend_id(plug_id, TraderContainerKind::Goods));

        let mut gold = CShadowWallet::new();
        gold.set_owner(SESSION_OWNER_TYPE, session_id);
        gold.set_container_extend_id(trade_container_extend_id(plug_id, TraderContainerKind::Gold));
        let mut yuan_bao = CShadowYuanBao::new();
        yuan_bao.set_owner(SESSION_OWNER_TYPE, session_id);
        yuan_bao.set_container_extend_id(trade_container_extend_id(
            plug_id,
            TraderContainerKind::YuanBao,
        ));
        Self {
            plug_id,
            session_id,
            owner_id,
            goods,
            gold,
            yuan_bao,
            ready: false,
        }
    }

    pub(crate) const fn plug_id(&self) -> i32 {
        self.plug_id
    }

    pub(crate) const fn session_id(&self) -> i32 {
        self.session_id
    }

    pub(crate) const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub(crate) const fn ready(&self) -> bool {
        self.ready
    }

    pub(crate) const fn set_trade_state(&mut self, ready: bool) {
        self.ready = ready;
    }

    pub(crate) fn goods_offers(&self) -> Vec<GoodsShadow> {
        self.goods
            .base()
            .base()
            .shadows()
            .values()
            .copied()
            .collect()
    }

    pub(crate) fn gold_amount(&self) -> u32 {
        self.gold.currency_amount()
    }

    pub(crate) fn yuan_bao_amount(&self) -> u32 {
        self.yuan_bao.currency_amount()
    }

    pub(crate) fn currency_offer(&self, kind: TraderContainerKind) -> Option<GoodsShadow> {
        let container = match kind {
            TraderContainerKind::Gold => self.gold.base(),
            TraderContainerKind::YuanBao => self.yuan_bao.base(),
            TraderContainerKind::Goods => return None,
        };
        container.base().shadows().values().next().copied()
    }

    pub(crate) fn record_offer(
        &mut self,
        kind: TraderContainerKind,
        position: u32,
        goods: &CGoods,
        amount: u32,
        previous: PreviousContainer,
        factory: &CGoodsFactory,
    ) -> Result<TraderOfferAdded, TraderOfferBlock> {
        if trade_offer_missing_goods(amount, goods.amount()) {
            return Err(TraderOfferBlock::MissingGoods);
        }
        let base_index = goods.base_properties_index();
        let expected_gold = factory.get_gold_coin_index();
        let expected_yuan_bao = factory.get_yuan_bao_index();
        let placed = PlacedShadowGoods {
            identity: goods.identity().ex_id,
            position: previous.goods_position,
            base_properties_index: base_index,
            amount,
        };
        let (record, presence, replaced) = match kind {
            TraderContainerKind::Goods => {
                if let Some(block) = trade_goods_offer_block(
                    base_index,
                    expected_gold,
                    expected_yuan_bao,
                    goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) as u32,
                    position,
                    self.goods.size(),
                ) {
                    return Err(block);
                }
                if !self.goods.is_space_enough(position) {
                    return Err(TraderOfferBlock::Occupied);
                }
                let added = self
                    .goods
                    .base_mut()
                    .record_placed_goods(previous, placed)
                    .map_err(|_| TraderOfferBlock::ShadowRejected)?;
                if !self
                    .goods
                    .occupy_cell(position, added.recorded.record.goods_id)
                {
                    let _ = self.goods.remove_shadow(added.recorded.record.goods_id);
                    return Err(TraderOfferBlock::Occupied);
                }
                (added.recorded.record, added.presence, None)
            }
            TraderContainerKind::Gold | TraderContainerKind::YuanBao => {
                if let Some(block) = trade_currency_offer_block(
                    kind,
                    base_index,
                    expected_gold,
                    expected_yuan_bao,
                    position,
                ) {
                    return Err(block);
                }
                let container = if kind == TraderContainerKind::Gold {
                    self.gold.base_mut()
                } else {
                    self.yuan_bao.base_mut()
                };
                let replaced_id = container.base().shadows().keys().next().copied();
                let replaced = replaced_id.and_then(|id| container.base_mut().remove_shadow(id));
                container.clear();
                container.set_goods_amount_limit(1);
                let added = container
                    .record_placed_goods(previous, placed)
                    .map_err(|_| TraderOfferBlock::ShadowRejected)?;
                (added.recorded.record, added.presence, replaced)
            }
        };
        self.ready = false;
        Ok(TraderOfferAdded {
            plug_id: self.plug_id,
            kind,
            position,
            record,
            presence,
            replaced,
        })
    }

    pub(crate) fn remove_offer(
        &mut self,
        kind: TraderContainerKind,
        position: u32,
        goods_id: CGuid,
    ) -> Option<TraderOfferRemoved> {
        let removed = match kind {
            TraderContainerKind::Goods => {
                if self.goods.query_goods_position(goods_id)? != position {
                    return None;
                }
                self.goods.remove_shadow(goods_id)?
            }
            TraderContainerKind::Gold => {
                if position != 0 {
                    return None;
                }
                self.gold.base_mut().base_mut().remove_shadow(goods_id)?
            }
            TraderContainerKind::YuanBao => {
                if position != 0 {
                    return None;
                }
                self.yuan_bao
                    .base_mut()
                    .base_mut()
                    .remove_shadow(goods_id)?
            }
        };
        self.ready = false;
        Some(TraderOfferRemoved {
            plug_id: self.plug_id,
            kind,
            position,
            removed,
        })
    }

    pub(crate) fn clear(&mut self) -> usize {
        self.ready = false;
        self.goods.clear() + self.gold.clear() + self.yuan_bao.clear()
    }
}

// Полный достигнутый CTrader lifecycle исполняется typed owner-ами выше;
// отдельной сохранённой RAW-копии замещённых функций в owner-файле нет.
