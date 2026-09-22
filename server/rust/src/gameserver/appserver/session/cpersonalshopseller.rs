//! Personal-shop seller plug исторического GameServer.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`, owner
//! `server/gameserver/appserver/session/cpersonalshopseller.cpp`. Seller хранит
//! имя, open-флаг, GUID-ordered цены и shadow-контейнер 8×6 с owner `(10, plug)`
//! и extend ID `plug << 8`. Выставление не переносит ownership товара:
//! контейнер сохраняет исходный packet/equipment slot, а покупка выполняет
//! отдельный remove/add pass с legacy rollback. Safe Rust storage заменяет
//! указатели/RTTI, сохраняя session, listener и wire-контракты owner-а.

use std::collections::BTreeMap;

use crate::gameserver::appserver::container::ccontainer::ContainerListenerHandle;
use crate::gameserver::appserver::container::cgoodsshadowcontainer::ShadowRemovedReport;
use crate::gameserver::appserver::container::cvolumelimitgoodsshadowcontainer::CVolumeLimitGoodsShadowContainer;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PersonalShopPrice {
    pub(crate) price_type: u32,
    pub(crate) price: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CPersonalShopSeller {
    shop_name: Vec<u8>,
    goods: CVolumeLimitGoodsShadowContainer,
    shop_opened: bool,
    prices: BTreeMap<CGuid, PersonalShopPrice>,
}

impl CPersonalShopSeller {
    pub(crate) fn inserted(plug_id: i32) -> Self {
        let mut goods = CVolumeLimitGoodsShadowContainer::new();
        goods.set_container_dimensions(8, 6);
        goods
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(10, plug_id);
        goods
            .base_mut()
            .base_mut()
            .set_container_extend_id(plug_id.wrapping_shl(8));
        let listener = usize::try_from(plug_id)
            .ok()
            .and_then(ContainerListenerHandle::from_legacy_identity);
        let _self_listener = goods
            .base_mut()
            .base_mut()
            .base_mut()
            .base_mut()
            .add_listener(listener);
        Self {
            shop_name: Vec::new(),
            goods,
            shop_opened: false,
            prices: BTreeMap::new(),
        }
    }

    pub(crate) const fn goods(&self) -> &CVolumeLimitGoodsShadowContainer {
        &self.goods
    }

    pub(crate) const fn goods_mut(&mut self) -> &mut CVolumeLimitGoodsShadowContainer {
        &mut self.goods
    }

    pub(crate) fn shop_name(&self) -> &[u8] {
        &self.shop_name
    }

    pub(crate) const fn shop_opened(&self) -> bool {
        self.shop_opened
    }

    pub(crate) const fn prices(&self) -> &BTreeMap<CGuid, PersonalShopPrice> {
        &self.prices
    }

    pub(crate) fn set_goods_price(&mut self, goods_id: CGuid, price_type: u32, price: u32) -> bool {
        if !self.goods.base().base().shadows().contains_key(&goods_id) {
            return false;
        }
        self.prices
            .insert(goods_id, PersonalShopPrice { price_type, price });
        true
    }

    pub(crate) fn goods_price(&self, goods_id: CGuid) -> Option<PersonalShopPrice> {
        self.prices.get(&goods_id).copied()
    }

    pub(crate) fn remove_goods(&mut self, goods_id: CGuid) -> Option<ShadowRemovedReport> {
        self.prices.remove(&goods_id);
        self.goods.remove_shadow(goods_id)
    }

    pub(crate) fn remove_goods_price(&mut self, goods_id: CGuid) {
        self.prices.remove(&goods_id);
    }

    pub(crate) fn retain_live_prices(&mut self) {
        let shadows = self.goods.base().base().shadows();
        self.prices
            .retain(|goods_id, _| shadows.contains_key(goods_id));
    }

    pub(crate) fn set_shop_name(&mut self, name: &[u8]) {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        self.shop_name.clear();
        self.shop_name.extend_from_slice(name);
    }

    pub(crate) fn open_for_business(&mut self) -> bool {
        self.shop_opened = true;
        true
    }

    pub(crate) fn close_down(&mut self) -> bool {
        self.shop_opened = false;
        true
    }
}
