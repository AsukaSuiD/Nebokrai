//! Visitor списка GUID товаров GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/cgoodslistlistener.cpp`. Новый listener имеет пустой
//! ordered vector и `is_all_goods_exist = true`; каждый встреченный `CGoods`
//! добавляет свой GUID и всегда продолжает traversal. MSVC vector/vtable и
//! явный destructor заменены `Vec`/`Drop`.

use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::public::guid::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsListListener {
    goods_ids: Vec<CGuid>,
    is_all_goods_exist: bool,
}

impl Default for GoodsListListener {
    fn default() -> Self {
        Self {
            goods_ids: Vec::new(),
            is_all_goods_exist: true,
        }
    }
}

impl GoodsListListener {
    pub(crate) fn goods_ids(&self) -> &[CGuid] {
        &self.goods_ids
    }

    pub(crate) const fn is_all_goods_exist(&self) -> bool {
        self.is_all_goods_exist
    }

    pub(crate) const fn set_all_goods_exist(&mut self, value: bool) {
        self.is_all_goods_exist = value;
    }

    pub(crate) fn visit(&mut self, goods: &CGoods) -> bool {
        self.goods_ids.push(goods.identity().ex_id);
        true
    }
}
