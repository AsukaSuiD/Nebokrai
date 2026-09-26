//! Visitor суммарной цены equipment-upgrade исторического GameServer:
//! wrapping-accumulator value-id `1` свойства `GAP_GOODS_UPGRADE_PRICE`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/cupgradepricelistener.cpp`. Accumulator начинается с
//! нуля, для каждого `CGoods` прибавляет value-id `1` свойства
//! `GAP_GOODS_UPGRADE_PRICE` с 32-битным wrapping и всегда продолжает обход.
//! RTTI/vtable и служебный destructor заменены обычным Rust-значением.

use super::cgoods::CGoods;
use crate::content::goods::GAP_GOODS_UPGRADE_PRICE;
use crate::content::goodsfactory::CGoodsFactory;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UpgradePriceListener {
    price: u32,
}

impl UpgradePriceListener {
    pub const fn price(self) -> u32 {
        self.price
    }

    pub fn visit(&mut self, factory: &CGoodsFactory, goods: &CGoods) -> bool {
        self.add_property_value(goods.addon_property_value(factory, GAP_GOODS_UPGRADE_PRICE, 1));
        true
    }

    pub const fn add_property_value(&mut self, value: i32) {
        self.price = self.price.wrapping_add(value as u32);
    }
}
