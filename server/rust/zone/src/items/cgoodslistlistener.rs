//! Visitor списка GUID товаров GameServer, перенесённый в Zone `items/` —
//! владельца типов контейнеров и операций над ними.
//!
//! Тело перенесено буквально из прежнего
//! `src/gameserver/appserver/listener/cgoodslistlistener.rs` (волна Z-C2c);
//! отличия — нормализация `pub(crate)`→`pub` на границе crate и швы переноса
//! (не расхождения): `CGoods` — Zone `items/cgoods.rs`, `CGuid` — Shared.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/cgoodslistlistener.cpp`. Новый listener имеет пустой
//! ordered vector и `is_all_goods_exist = true`; каждый встреченный `CGoods`
//! добавляет свой GUID и всегда продолжает traversal. MSVC vector/vtable и
//! явный destructor заменены `Vec`/`Drop`.

use super::cgoods::CGoods;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoodsListListener {
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
    pub fn goods_ids(&self) -> &[CGuid] {
        &self.goods_ids
    }

    pub const fn is_all_goods_exist(&self) -> bool {
        self.is_all_goods_exist
    }

    pub const fn set_all_goods_exist(&mut self, value: bool) {
        self.is_all_goods_exist = value;
    }

    pub fn visit(&mut self, goods: &CGoods) -> bool {
        self.goods_ids.push(goods.identity().ex_id);
        true
    }
}
