//! Поиск товаров при обходе контейнера исторического `WorldServer`.
//!
//! Статус конструктора RVA `0x000D6980`, деструктора `0x000D6610`,
//! `SetTarget` `0x000D6460` и `OnTraversingContainer` `0x000D6A10` —
//! `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\listener\cseekgoodslistener.cpp:13,18,23,31`.
//!
//! Exact constructor `0x004D6980..0x004D6996` задаёт target `0` и пустой
//! `std::vector<CGUID>`; `Vec<CGuid>` и `Drop` заменяют его storage/destructor.
//! `SetTarget` `0x004D6460..0x004D6478` при null не меняет прежний target, а
//! при non-null сохраняет результат `CGoodsFactory::QueryGoodsIDByOriginalName`.
//! В частности, он не очищает уже собранный список — это сохранено буквально.
//! Очищенный `Nebokrai/server/cpp` принимает готовый numeric id и очищает
//! результаты в setter-е; Rust намеренно не переносит эти два удобных, но не
//! подтверждённых EXE изменения. Старый Linux-донор здесь совпадает с exact.
//!
//! Exact traversal `0x004D6A10..0x004D6A54` делает RTTI cast к `CGoods`,
//! сравнивает `GetBasePropertiesIndex()` с target, копирует GUID из поля
//! товара `+0xC` в конец vector и при любом объекте возвращает `1`. Safe enum
//! из base owner-а заменяет только RTTI-механику и явно сохраняет non-goods
//! ветку; container-параметр не представлен, потому что exact тело его не
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
    /// Создаёт listener с exact target `0` и пустым списком результатов.
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
