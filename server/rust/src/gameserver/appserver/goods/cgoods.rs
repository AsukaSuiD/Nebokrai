//! Достигнутый object/addon core `CGoods` старого GameServer перенесён в Zone items.
//! Здесь его реэкспорт для старого пакета и привязка шва lookup реестра базовых
//! свойств к прежнему `CGoodsFactory` до его собственной волны.

pub(crate) use nebokrai_zone::items::cgoods::*;

use nebokrai_zone::content::goods::CGoodsBaseProperties;

use super::cgoodsfactory::CGoodsFactory;

/// Реализация zone-шва прежним владельцем реестра; тело делегирует прежнему
/// inherent-поиску, имя операции сохранено.
impl GoodsBasePropertiesLookup for CGoodsFactory {
    fn query_goods_base_properties(&self, goods_id: u32) -> Option<&CGoodsBaseProperties> {
        self.query_goods_base_properties(goods_id)
    }
}
