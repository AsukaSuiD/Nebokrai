//! Поиск цели боевого ИИ `CGladiator` и `CStupidGladiator`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! владельцы `appserver/ai/gladiator.cpp` и `stupidgladiator.cpp` подтверждают
//! одинаковый поиск ближайшей живой цели по `RealDistance`. Дальность берётся
//! из `CMonster::GetGuardRange`, равенство заменяет предыдущую запись, а общий
//! цикл сохраняет порядок игроков, питомцев и совместимых повозок.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\gladiator.cpp

// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::shape::ShapeIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GladiatorTarget {
    pub(crate) identity: ShapeIdentity,
    pub(crate) distance: i32,
}

/// Общий для `CGladiator` и `CStupidGladiator` выбор ближайшей живой цели:
/// дальность берётся из `CMonster::GetGuardRange`, равенство заменяет прежнюю
/// запись и тем самым сохраняет порядок игроков, питомцев и повозок.
pub(crate) fn consider_gladiator_target(
    selected: Option<GladiatorTarget>,
    candidate: GladiatorTarget,
    guard_range: i32,
) -> Option<GladiatorTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    match selected {
        Some(current) if current.distance < candidate.distance => Some(current),
        _ => Some(candidate),
    }
}
