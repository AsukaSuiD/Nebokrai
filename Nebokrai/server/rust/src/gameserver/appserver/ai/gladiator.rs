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

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::CGame;
use crate::setup::monsterlist::MonsterProperties;

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

/// Выполняет общий поиск AI0/AI3 по игрокам, питомцам и совместимым повозкам.
/// Поздняя категория заменяет прежнюю цель при равной дистанции.
#[allow(clippy::too_many_arguments, reason = "граница сохраняет ownership повозки и свойства нападающего")]
pub(crate) fn select_gladiator_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    property: &MonsterProperties,
    tamed: bool,
    attacker_master: MasterInfo,
) -> Option<ShapeIdentity> {
    let guard_range = property.guard_range as i32;
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
        selected = consider_gladiator_target(
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
        );
    }
    if property.kind != 5 {
        for pet_id in region.pet_ids_around_area(area_index) {
            let Some(candidate) = region
                .find_monster_by_id(pet_id)
                .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
                .and_then(|pet| {
                    let property =
                        game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                    pet.shape_view(property)
                })
            else {
                continue;
            };
            selected = consider_gladiator_target(
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
            );
        }
    }
    for carriage_id in region.carriage_ids_around_area(area_index) {
        let Some((candidate, target_master)) =
            region.find_monster_by_id(carriage_id).and_then(|carriage| {
                let carriage_property =
                    game.find_monster_property_by_origin_name(carriage.base_property_key()?)?;
                if !carriage.is_carriage(carriage_property)
                    || CMoveShape::is_died(carriage.hit_points())
                {
                    return None;
                }
                Some((carriage.shape_view(carriage_property)?, carriage.master_info()))
            })
        else {
            continue;
        };
        if !game.carriage_attackable_by_monster(
            property,
            tamed,
            attacker_master,
            target_master,
            region.id,
        ) {
            continue;
        }
        selected = consider_gladiator_target(
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
        );
    }
    selected.map(|selected| selected.identity)
}
