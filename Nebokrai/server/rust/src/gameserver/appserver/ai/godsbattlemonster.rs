//! Выбор цели монстра битвы богов `CGodsBattleMonsterAI`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает три последовательных обхода: игроки, питомцы, обычные монстры.
//! Кандидат своей фракции исключается, дальность берётся из `GetGuardRange`,
//! а равная `RealDistance` заменяет предыдущую цель.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\godsbattlemonster.cpp
// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::ai::gladiator::{
    GladiatorTarget, consider_gladiator_target,
};

/// Применяет фракционный фильтр перед общим выбором ближайшей цели.
pub(crate) fn consider_gods_battle_target(
    selected: Option<GladiatorTarget>,
    candidate: GladiatorTarget,
    guard_range: i32,
    owner_faction: u32,
    candidate_faction: u32,
) -> Option<GladiatorTarget> {
    if candidate_faction == owner_faction {
        selected
    } else {
        consider_gladiator_target(selected, candidate, guard_range)
    }
}
