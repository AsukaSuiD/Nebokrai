//! Локальная идентичность фигуры для игровых ссылок и маршрутов.
//! Источник сопоставления базовых type/ID: gameserver.exe + GameServer.pdb,
//! appserver/shape.h и appserver/states/state.cpp/.h.

use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShapeIdentity {
    pub object_type: i32,
    pub id: i32,
    pub ex_id: CGuid,
}
