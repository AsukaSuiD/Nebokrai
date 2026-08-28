//! Поиск цели атакующего монстра войны стран `WarAttackMonster`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает поиск ближайшего живого противника по лагерю `CountryWarSys`.
//! Реальный тип ИИ `18` сохраняет порядок игроков перед питомцами и замену
//! предыдущей цели при равной `RealDistance`.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\warattackmonster.cpp
// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::ai::gladiator::{
    GladiatorTarget, consider_gladiator_target,
};

/// Отбрасывает собственный лагерь и сохраняет общий выбор ближайшей цели.
pub(crate) fn consider_country_war_target(
    selected: Option<GladiatorTarget>,
    candidate: GladiatorTarget,
    guard_range: i32,
    own_camp: i32,
    candidate_camp: i32,
) -> Option<GladiatorTarget> {
    if candidate_camp == own_camp {
        selected
    } else {
        consider_gladiator_target(selected, candidate, guard_range)
    }
}
