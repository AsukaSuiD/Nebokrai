//! Общий узкий выбор цели для охранников с минимальной дистанцией навыка.
//!
//! Несколько подтверждённых AI-owner-ов используют одинаковый порядок:
//! внутри отдельной категории предпочитается ближайшая цель не ближе
//! минимальной дистанции, а при отсутствии такой цели остаётся последняя
//! слишком близкая запись. Фильтры страны, фракции и союза остаются у
//! конкретных владельцев.

use crate::gameserver::appserver::shape::{ShapeAreaCoordinates, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::real_distance;

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
