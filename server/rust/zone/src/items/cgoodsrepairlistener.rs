//! Visitor ремонта всех предметов GameServer, перенесённый в Zone `items/` —
//! владельца типов контейнеров и операций над ними.
//!
//! Тело перенесено буквально из прежнего
//! `src/gameserver/appserver/listener/cgoodsrepairlistener.rs` (волна Z-C2c);
//! отличия — нормализация `pub(crate)`→`pub` на границе crate и швы переноса
//! (не расхождения): `CGoods` — Zone `items/cgoods.rs`, реестр
//! `CGoodsFactory` — Zone `content/goodsfactory.rs` (волна Z-G0b).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/listener/cgoodsrepairlistener.cpp`. `OnTraversingContainer`
//! пропускает не-`CGoods`, а для каждого ремонтопригодного товара вызывает
//! `CGoodsFactory::RepairEquipment` и всегда продолжает traversal.
//!
//! RTTI и polymorphic listener заменены прямым типизированным вызовом из
//! shop-owner-а; predicate и mutation остаются у `CGoods`/`CGoodsFactory`.

use super::cgoods::CGoods;
use crate::content::goodsfactory::CGoodsFactory;

/// Точное полезное действие `CGoodsRepairListener::OnTraversingContainer`.
/// Возврат `true` сохраняет безусловное продолжение исходного traversal.
pub fn repair_visited_goods(factory: &CGoodsFactory, goods: &mut CGoods) -> bool {
    if goods.can_repair(factory) {
        let _ = factory.repair_equipment(goods);
    }
    true
}
