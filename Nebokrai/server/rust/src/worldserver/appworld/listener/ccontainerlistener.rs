//! Базовый listener обхода контейнера исторического `WorldServer`.
//!
//! Статус `CContainerListener::OnTraversingContainer` RVA `0x000DD750` —
//! `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\listener\ccontainerlistener.cpp:46`.
//!
//! Exact `0x004DD750..0x004DD755` безусловно возвращает `1` и не читает ни
//! `CContainer*`, ни `CBaseObject*`. Rust сохраняет точный `int` callback-а,
//! но не переносит C++-иерархию и RTTI: объект представлен узким enum только
//! с различием, которое уже наблюдает `CSeekGoodsListener` — товар либо иной
//! `CBaseObject`. Неиспользуемый container-параметр не вводится до owner-а,
//! которому он действительно понадобится.
//!
//! `Catch@004D6857`, `Catch@004D68E6` и `Unwind@00535410`, приписанные этому
//! translation unit экспортом, являются STL/compiler cleanup и не получают
//! отдельных Rust-тел; владение заменено `Vec` и обычным `Drop`.

use crate::worldserver::appworld::goods::cgoods::CGoods;

/// Достигнутое различие типов в общем `CBaseObject*` callback-а.
pub(crate) enum TraversedContainerObject<'object> {
    Goods(&'object CGoods),
    Other,
}

/// Safe Rust-форма виртуального контракта `CContainerListener`.
pub(crate) trait CContainerListener {
    /// Базовый owner всегда разрешает продолжить traversal точным значением `1`.
    fn on_traversing_container(&mut self, _object: TraversedContainerObject<'_>) -> i32 {
        1
    }
}
