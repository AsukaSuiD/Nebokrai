//! Поиск товаров при обходе контейнера исторического `WorldServer`.
//!
//! `SetTarget` и `OnTraversingContainer` —
//! действует. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! constructor задаёт target `0` и пустой
//! `std::vector<CGUID>`; `Vec<CGuid>` и `Drop` заменяют его storage/destructor.
//! `SetTarget` при null не меняет прежний target, а
//! при non-null сохраняет результат `CGoodsFactory::QueryGoodsIDByOriginalName`.
//! В частности, он не очищает уже собранный список — это сохранено буквально.
//! Очищенный `Nebokrai/server/cpp` принимает готовый numeric id и очищает
//! результаты в setter-е; Rust намеренно не переносит эти два удобных, но не
//!
//! traversal делает RTTI cast к `CGoods`,
//! сравнивает `GetBasePropertiesIndex()` с target, копирует GUID из поля
//! товара `+0xC` в конец vector и при любом объекте возвращает `1`. Safe enum
//! из base owner-а заменяет только RTTI-механику и явно сохраняет non-goods
//! ветку; container-параметр не представлен, потому что тело его не
//! читает. Неинициализированный в C++ товар не материализуется как случайный
//! `u32`: Rust `None` не совпадает ни с каким target.
//!
//! Приписанные translation unit тела `CAuctionLog::stLogNode`, ADO wrappers,
//! `Catch@...`, `Unwind@...` и внутренности `std::vector` относятся к другим
//! owner-ам либо к библиотечной/компиляторной форме и не получают Rust-копий.

use std::ffi::CStr;

use crate::public::guid::CGuid;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsOriginalNameIndex, query_goods_id_by_original_name,
};
use crate::worldserver::appworld::listener::ccontainerlistener::{
    CContainerListener, TraversedContainerObject,
};

/// Safe Rust-состояние исходного `CSeekGoodsListener`.
#[derive(Default)]
pub(crate) struct CSeekGoodsListener {
    target_goods_index: u32,
    goods_ids: Vec<CGuid>,
}

impl CSeekGoodsListener {
 /// Создаёт listener с target `0` и пустым списком результатов.
    pub(crate) fn new() -> Self {
        Self::default()
    }

 /// Назначает target по legacy original-name; `None` оставляет его прежним.
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

 /// Заимствует GUID в исходном порядке traversal-а, включая дубликаты.
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
