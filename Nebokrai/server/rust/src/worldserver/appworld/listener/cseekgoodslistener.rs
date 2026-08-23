//! Listener поиска товаров `CSeekGoodsListener` из WorldServer, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! `SetTarget(null)` сохраняет прежний target; непустое имя разрешается через
//! goods factory и не очищает уже найденные GUID. Traversal добавляет GUID
//! каждого Goods с совпавшим base-properties index и всегда возвращает 1.
//!
//! `Vec<CGuid>` заменяет MSVC vector, а typed base-object variant — RTTI.
//! Неназначенный goods index не совпадает ни с каким target.

use std::ffi::CStr;

use crate::public::guid::CGuid;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsOriginalNameIndex, query_goods_id_by_original_name,
};
use crate::worldserver::appworld::listener::ccontainerlistener::{
    CContainerListener, TraversedContainerObject,
};

#[derive(Default)]
pub(crate) struct CSeekGoodsListener {
    target_goods_index: u32,
    goods_ids: Vec<CGuid>,
}

impl CSeekGoodsListener {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_target(
        &mut self,
        original_name: Option<&CStr>,
        original_name_index: &GoodsOriginalNameIndex,
    ) {
        let Some(original_name) = original_name else {
            return;
        };

        self.target_goods_index =
            query_goods_id_by_original_name(original_name_index, Some(original_name));
    }

    pub(crate) fn goods_ids(&self) -> &[CGuid] {
        &self.goods_ids
    }
}

impl CContainerListener for CSeekGoodsListener {
    fn on_traversing_container(&mut self, object: TraversedContainerObject<'_>) -> i32 {
        if let TraversedContainerObject::Goods(goods) = object
            && goods.get_base_properties_index() == Some(self.target_goods_index)
        {
            self.goods_ids.push(*goods.get_ex_id());
        }

        1
    }
}
