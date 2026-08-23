//! Базовый listener обхода контейнера исторического `WorldServer`.
//!
//! действует. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! безусловно возвращает `1` и не читает ни
//! `CContainer*`, ни `CBaseObject*`. Rust сохраняет точный `int` callback-а,
//! но не переносит C++-иерархию и RTTI: объект представлен узким enum только
//! с различием, которое уже наблюдает `CSeekGoodsListener` — товар либо иной
//! `CBaseObject`. Неиспользуемый container-параметр не вводится до owner-а,
//! которому он действительно понадобится.
//!
//! `Catch@004D6857`, `Catch@004D68E6` и `Unwind@00535410`, приписанные этому
//! translation unit, являются STL/compiler cleanup и не получают
//! отдельных Rust-тел; владение заменено `Vec` и обычным `Drop`.

use crate::worldserver::appworld::goods::cgoods::CGoods;

/// Действующее различие типов в общем `CBaseObject*` callback-а.
pub(crate) enum TraversedContainerObject<'object> {
    Goods(&'object CGoods),
    Other,
}

/// Safe Rust-форма виртуального контракта `CContainerListener`.
pub(crate) trait CContainerListener: Send {
 /// Базовый owner всегда разрешает продолжить traversal точным значением `1`.
    fn on_traversing_container(&mut self, _object: TraversedContainerObject<'_>) -> i32 {
        1
    }
}
