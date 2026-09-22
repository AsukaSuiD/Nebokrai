//! Rust-адаптер генерации идентификаторов предметов поверх Shared CGuid.
//! Отказ источника не превращается в предмет с нулевым GUID; граница описана
//! в docs/gameplay/items.md, раздел «Создание идентификатора предмета».

use nebokrai_shared::values::CGuid;

use super::cgoods::CGoods;

pub(crate) fn create_goods_guid() -> Option<CGuid> {
    match CGuid::create() {
        Ok(guid) => Some(guid),
        Err(error) => {
            tracing::error!(%error, "Не удалось создать GUID предмета");
            None
        }
    }
}

pub(crate) fn clone_goods_with_new_guid(source: &CGoods) -> Option<CGoods> {
    let guid = create_goods_guid()?;
    let mut goods = source.clone();
    goods.set_ex_id(guid);
    Some(goods)
}
