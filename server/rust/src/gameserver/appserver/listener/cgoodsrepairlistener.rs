//! Visitor ремонта всех предметов GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/cgoodsrepairlistener.cpp`. `OnTraversingContainer`
//! пропускает не-`CGoods`, а для каждого ремонтопригодного товара вызывает
//! `CGoodsFactory::RepairEquipment` и всегда продолжает traversal.
//!
//! RTTI и polymorphic listener заменены прямым типизированным вызовом из
//! shop-owner-а; predicate и mutation остаются у `CGoods`/`CGoodsFactory`.

use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;

/// Точное полезное действие `CGoodsRepairListener::OnTraversingContainer`.
/// Возврат `true` сохраняет безусловное продолжение исходного traversal.
pub(crate) fn repair_visited_goods(factory: &CGoodsFactory, goods: &mut CGoods) -> bool {
    if goods.can_repair(factory) {
        let _ = factory.repair_equipment(goods);
    }
    true
}
