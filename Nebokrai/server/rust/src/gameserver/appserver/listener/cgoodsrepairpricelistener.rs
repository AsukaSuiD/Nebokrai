//! Visitor расчёта полной стоимости ремонта GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/cgoodsrepairpricelistener.cpp`. Конструктор начинает с
//! нулевых `price/count`; traversal учитывает только ремонтопригодный товар с
//! положительной максимальной и меньшей текущей прочностью, увеличивает count
//! и wrapping-сумму `CGoodsFactory::CalculateRepairPrice`, затем всегда
//! продолжает обход. Деструктор не имел наблюдаемого эффекта кроме очистки
//! служебных полей.
//!
//! RTTI/vtable заменены обычным Rust accumulator-ом. Переданный repair factor
//! является уже восстановленной конфигурационной зависимостью factory-owner-а.

use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_GOODS_MAXIMUM_DURABILITY;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsRepairPrice {
    price: u32,
    equipment_count: u32,
}

impl GoodsRepairPrice {
    pub(crate) const fn price(self) -> u32 {
        self.price
    }

    pub(crate) const fn equipment_count(self) -> u32 {
        self.equipment_count
    }

    /// Точное полезное действие `OnTraversingContainer`; `true` означает
    /// безусловное продолжение обхода контейнера.
    pub(crate) fn visit(
        &mut self,
        factory: &CGoodsFactory,
        goods: &CGoods,
        repair_factor: f32,
    ) -> bool {
        if goods.can_repair(factory) {
            let current = goods.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2);
            let maximum = goods.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 1);
            if 0 < maximum && current < maximum {
                self.equipment_count = self.equipment_count.wrapping_add(1);
                self.price = self
                    .price
                    .wrapping_add(factory.calculate_repair_price(goods, repair_factor));
            }
        }
        true
    }
}
