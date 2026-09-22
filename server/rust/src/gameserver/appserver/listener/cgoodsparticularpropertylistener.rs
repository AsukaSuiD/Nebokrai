//! Visitor товаров с заданным addon-property GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/cgoodsparticularpropertylistener.cpp`. Новый listener
//! начинает с пустого ordered vector и `GAP_UNKNOW` (`0`); traversal добавляет
//! GUID каждого `CGoods`, у которого value-id `1` выбранного property ненулевой,
//! и всегда продолжается. RTTI/vtable/vector lifecycle заменены типами Rust.

use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsParticularPropertyListener {
    goods_ids: Vec<CGuid>,
    property: i32,
}

impl GoodsParticularPropertyListener {
    pub(crate) const fn new(property: i32) -> Self {
        Self {
            goods_ids: Vec::new(),
            property,
        }
    }

    pub(crate) fn goods_ids(&self) -> &[CGuid] {
        &self.goods_ids
    }

    pub(crate) fn visit(&mut self, factory: &CGoodsFactory, goods: &CGoods) -> bool {
        if goods.addon_property_value(factory, self.property, 1) != 0 {
            self.goods_ids.push(goods.identity().ex_id);
        }
        true
    }
}
