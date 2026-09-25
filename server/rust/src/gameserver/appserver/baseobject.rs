//! Базовый `CBaseObject` старого GameServer перенесён в Zone regions.
//! Здесь фабрика отдельных ветвей, принадлежащая доменным классам.

use super::monster::CMonster;
use super::npc::CNpc;

/// Материализует точную ветвь `CreateObject(500, id)`: сначала constructor
/// `CNpc`, затем общая factory-tail запись type/ID.
pub(crate) fn create_npc(id: i32) -> CNpc {
    let mut npc = CNpc::with_constructor_defaults();
    npc.move_shape_mut()
        .shape_mut()
        .base_object_mut()
        .set_id(id);
    npc
}

/// Материализует ветвь `CreateObject(600, id)` до derived skills/AI Init.
pub(crate) fn create_monster(id: i32) -> CMonster {
    let mut monster = CMonster::with_constructor_defaults();
    monster
        .move_shape_mut()
        .shape_mut()
        .base_object_mut()
        .set_id(id);
    monster
}
