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
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::CGame;

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

/// Выполняет достигнутый поиск AI104 по игрокам, питомцам и всем монстрам,
/// сохраняя исходную категорийную последовательность и faction-поле каждой
/// категории.
pub(crate) fn select_gods_battle_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    owner_faction: u32,
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
        selected = consider_gods_battle_target(
            selected,
            GladiatorTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            owner_faction,
            player.gods_battle_faction() as u32,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some((candidate, faction)) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                Some((pet.shape_view(property)?, property.race))
            })
        else {
            continue;
        };
        selected = consider_gods_battle_target(
            selected,
            GladiatorTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            owner_faction,
            faction,
        );
    }
    for monster_id in region.monster_ids_around_area(area_index) {
        let Some((candidate, faction)) = region
            .find_monster_by_id(monster_id)
            .filter(|monster| !CMoveShape::is_died(monster.hit_points()))
            .and_then(|monster| {
                let property =
                    game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
                Some((monster.shape_view(property)?, property.race))
            })
        else {
            continue;
        };
        selected = consider_gods_battle_target(
            selected,
            GladiatorTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            owner_faction,
            faction,
        );
    }
    selected.map(|selected| selected.identity)
}
