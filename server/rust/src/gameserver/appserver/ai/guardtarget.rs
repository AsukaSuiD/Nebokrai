//! Общий узкий выбор цели для охранников с минимальной дистанцией навыка.
//!
//! Несколько подтверждённых AI-owner-ов используют одинаковый порядок:
//! внутри отдельной категории предпочитается ближайшая цель не ближе
//! минимальной дистанции, а при отсутствии такой цели остаётся последняя
//! слишком близкая запись. Фильтры страны, фракции и союза остаются у
//! конкретных владельцев. Проход ближайшего игрока/питомца владыки и синего
//! босса (`select_nearest_player_or_pet`) переехал в Zone `ai/lord.rs`
//! кластером E1; здесь остаются дистанционное ядро и групповой выбор
//! охранников до своей порции.

use crate::gameserver::appserver::shape::{ShapeAreaCoordinates, ShapeIdentity, ShapeView};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GuardStationState {
    station: Option<ShapeAreaCoordinates>,
}

impl GuardStationState {
    pub(crate) const fn station(&self) -> Option<ShapeAreaCoordinates> {
        self.station
    }

    pub(crate) fn record_station(&mut self, owner: ShapeView) {
        self.station.get_or_insert(ShapeAreaCoordinates {
            x: owner.tile_x,
            y: owner.tile_y,
        });
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
