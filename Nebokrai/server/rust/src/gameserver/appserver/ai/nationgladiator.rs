//! Поиск цели национального гладиатора `CNationGladiator`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает тип ИИ `18`: игрок своей страны остаётся допустим только как
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
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::CGame;

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

/// Выполняет достигнутый поиск AI18 по игрокам и питомцам, сохраняя разные
/// источники страны и правило преступника только для игрока.
pub(crate) fn select_nation_gladiator_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    owner_country: u32,
) -> Option<ShapeIdentity> {
    let mut selected = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.id) || player.is_dead() {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected = consider_nation_gladiator_target(
            selected,
            GladiatorTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            owner_country,
            u32::from(player.country()),
            player.is_badman(game.globe_setup().pk_count_per_kill()),
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some((candidate, country)) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                Some((pet.shape_view(property)?, pet.master_info().master_country_id))
            })
        else {
            continue;
        };
        selected = consider_nation_gladiator_target(
            selected,
            GladiatorTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            owner_country,
            country as u32,
            false,
        );
    }
    selected.map(|selected| selected.identity)
}
