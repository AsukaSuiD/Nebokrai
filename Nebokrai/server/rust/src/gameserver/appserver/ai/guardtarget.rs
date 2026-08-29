//! Общий узкий выбор цели для охранников с минимальной дистанцией навыка.
//!
//! Несколько подтверждённых AI-owner-ов используют одинаковый порядок:
//! внутри отдельной категории предпочитается ближайшая цель не ближе
//! минимальной дистанции, а при отсутствии такой цели остаётся последняя
//! слишком близкая запись. Фильтры страны, фракции и союза остаются у
//! конкретных владельцев.

use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeAreaCoordinates, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::CGame;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GuardStationState {
    station: Option<ShapeAreaCoordinates>,
}

impl GuardStationState {
    pub(crate) fn record_station(&mut self, owner: ShapeView) {
        self.station.get_or_insert(ShapeAreaCoordinates {
            x: owner.tile_x,
            y: owner.tile_y,
        });
    }

    pub(crate) fn left_chase_range(&self, owner: ShapeView, chase_range: i32) -> bool {
        self.station.is_some_and(|station| {
            real_distance(owner.tile_x, owner.tile_y, station.x, station.y) > chase_range
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GuardDistanceTarget {
    pub(crate) identity: ShapeIdentity,
    pub(crate) distance: i32,
}

pub(crate) fn consider_guard_distance_target(
    selected: Option<GuardDistanceTarget>,
    candidate: GuardDistanceTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<GuardDistanceTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    let Some(current) = selected else {
        return Some(candidate);
    };
    if current.distance <= candidate.distance {
        if current.distance < minimum_skill_distance {
            Some(candidate)
        } else {
            Some(current)
        }
    } else if candidate.distance < minimum_skill_distance {
        Some(current)
    } else {
        Some(candidate)
    }
}

/// Первый категорийный список выигрывает при равной дистанции.
pub(crate) fn select_guard_target_groups(
    first: Option<GuardDistanceTarget>,
    second: Option<GuardDistanceTarget>,
) -> Option<GuardDistanceTarget> {
    match (first, second) {
        (Some(first), Some(second)) if second.distance < first.distance => Some(second),
        (Some(first), _) => Some(first),
        (None, second) => second,
    }
}

/// Выбирает ближайшую живую цель общим проходом игроков, затем питомцев.
/// Равная дистанция заменяет предыдущую запись, поэтому порядок индексов и
/// категорий остаётся частью результата.
pub(crate) fn select_nearest_player_or_pet(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<GuardDistanceTarget> {
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
        let candidate = GuardDistanceTarget {
            identity: candidate.identity,
            distance: real_distance(
                owner.tile_x,
                owner.tile_y,
                candidate.tile_x,
                candidate.tile_y,
            ),
        };
        if candidate.distance <= guard_range
            && selected.is_none_or(|current: GuardDistanceTarget| {
                candidate.distance <= current.distance
            })
        {
            selected = Some(candidate);
        }
    }
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
        let candidate = GuardDistanceTarget {
            identity: candidate.identity,
            distance: real_distance(
                owner.tile_x,
                owner.tile_y,
                candidate.tile_x,
                candidate.tile_y,
            ),
        };
        if candidate.distance <= guard_range
            && selected.is_none_or(|current| candidate.distance <= current.distance)
        {
            selected = Some(candidate);
        }
    }
    selected
}
