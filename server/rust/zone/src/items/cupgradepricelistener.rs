//! Visitor суммарной цены equipment-upgrade GameServer, перенесённый в Zone
//! `items/` — владельца типов контейнеров и операций над ними.
//!
//! Тело перенесено буквально из прежнего
//! `src/gameserver/appserver/listener/cupgradepricelistener.rs` (волна Z-C2c);
//! отличия — нормализация `pub(crate)`→`pub` на границе crate и швы переноса
//! (не расхождения): `CGoods` — Zone `items/cgoods.rs`, GAP-константа цены
//! upgrade — Zone `content/goods.rs`, реестр `CGoodsFactory` — Zone
//! `content/goodsfactory.rs` (волна Z-G0b).
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
