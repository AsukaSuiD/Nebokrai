//! Visitor поиска товаров по catalog index GameServer, перенесённый в Zone
//! `items/` — владельца типов контейнеров и операций над ними.
//!
//! Тело перенесено буквально из прежнего
//! `src/gameserver/appserver/listener/cseekgoodslistener.rs` (волна Z-C2c);
//! отличия — нормализация `pub(crate)`→`pub` на границе crate и швы переноса
//! (не расхождения): `CGoods` — Zone `items/cgoods.rs`, реестр
//! `CGoodsFactory` — Zone `content/goodsfactory.rs` (волна Z-G0b), `CGuid` —
//! Shared.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/cseekgoodslistener.cpp`. Новый listener имеет target `0`
//! и пустой ordered GUID-vector. Ненулевой numeric target заменяет прежний;
//! ненулевое имя разрешается через original-name index factory и записывается
//! даже при результате «не найдено». Traversal собирает GUID всех `CGoods` с
//! равным base-properties index и всегда продолжается. RTTI/vtable/vector
//! lifecycle заменены типами Rust.

use super::cgoods::CGoods;
use crate::content::goodsfactory::CGoodsFactory;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SeekGoodsListener {
    goods_ids: Vec<CGuid>,
    target_index: u32,
}

impl SeekGoodsListener {
    pub fn goods_ids(&self) -> &[CGuid] {
        &self.goods_ids
    }

    pub fn set_target_name(&mut self, factory: &CGoodsFactory, name: Option<&[u8]>) {
        if let Some(name) = name {
            self.target_index = factory.query_goods_id_by_original_name(Some(name));
        }
    }

    pub const fn set_target_index(&mut self, target_index: u32) {
        if target_index != 0 {
            self.target_index = target_index;
        }
    }

    pub fn visit(&mut self, goods: &CGoods) -> bool {
        if goods.base_properties_index() == self.target_index {
            self.goods_ids.push(goods.identity().ex_id);
        }
        true
    }
}
