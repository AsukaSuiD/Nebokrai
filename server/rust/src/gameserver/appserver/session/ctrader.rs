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

const SESSION_OWNER_TYPE: i32 = 10;
const TRADE_GOODS_CELLS: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TraderContainerKind {
    Goods,
    Gold,
    YuanBao,
}

impl TraderContainerKind {
    pub(crate) const fn from_index(index: i32) -> Option<Self> {
        match index {
            0 => Some(Self::Goods),
            1 => Some(Self::Gold),
            2 => Some(Self::YuanBao),
            _ => None,
        }
    }

    pub(crate) const fn index(self) -> i32 {
        match self {
            Self::Goods => 0,
            Self::Gold => 1,
            Self::YuanBao => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TraderOfferBlock {
    InvalidContainer,
    InvalidPosition,
    MissingGoods,
    CurrencyInGoodsContainer,
    InvalidCurrency,
    NoTrade,
    Occupied,
    ShadowRejected,
}

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
            .set_container_extend_id(plug_id.wrapping_shl(8));

        let mut gold = CShadowWallet::new();
        gold.set_owner(SESSION_OWNER_TYPE, session_id);
        gold.set_container_extend_id(plug_id.wrapping_shl(8) | 1);
        let mut yuan_bao = CShadowYuanBao::new();
        yuan_bao.set_owner(SESSION_OWNER_TYPE, session_id);
        yuan_bao.set_container_extend_id(plug_id.wrapping_shl(8) | 2);
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
        if amount == 0 || amount > goods.amount() {
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
                if base_index == expected_gold || base_index == expected_yuan_bao {
                    return Err(TraderOfferBlock::CurrencyInGoodsContainer);
                }
                if goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) & 0x20 != 0 {
                    return Err(TraderOfferBlock::NoTrade);
                }
                if position >= self.goods.size() {
                    return Err(TraderOfferBlock::InvalidPosition);
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
                let expected = if kind == TraderContainerKind::Gold {
                    expected_gold
                } else {
                    expected_yuan_bao
                };
                if base_index != expected {
                    return Err(TraderOfferBlock::InvalidCurrency);
                }
                if position != 0 {
                    return Err(TraderOfferBlock::InvalidPosition);
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
