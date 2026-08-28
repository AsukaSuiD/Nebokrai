//! Поиск цели национального гладиатора `CNationGladiator`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает тип ИИ `21`: игрок своей страны остаётся допустим только как
//! преступник, питомец своей страны исключается, а ближайшая цель выбирается
//! по `RealDistance` с заменой при равенстве.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\nationgladiator.cpp
// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::ai::gladiator::{
    GladiatorTarget, consider_gladiator_target,
};

/// Применяет национальный фильтр перед общим выбором ближайшей цели.
pub(crate) fn consider_nation_gladiator_target(
    selected: Option<GladiatorTarget>,
    candidate: GladiatorTarget,
    guard_range: i32,
    owner_country: u32,
    candidate_country: u32,
    same_country_criminal: bool,
) -> Option<GladiatorTarget> {
    if candidate_country == owner_country && !same_country_criminal {
        selected
    } else {
        consider_gladiator_target(selected, candidate, guard_range)
    }
}
