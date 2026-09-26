//! Общий узкий выбор цели охранников с минимальной дистанцией текущего навыка.
//!
//! Несколько подтверждённых AI-owner-ов используют одинаковый порядок: внутри
//! отдельной категории предпочитается ближайшая цель не ближе минимальной
//! дистанции, а при отсутствии такой цели остаётся последняя слишком близкая
//! запись. Фильтры страны, фракции и союза остаются у конкретных владельцев.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (идентификаторы —
//! docs/reconstruction/gameserver-npc-and-regions.md#идентификаторы-сборки).
//! Дистанционное ядро (выбор ближайшего не ближе min-distance навыка,
//! замена слишком близкой сохранённой записью следующей, победа первого
//! категорийного списка при равной дистанции, однократная фиксация точки
//! поста) разобрано построчно по телам городских selector-ов — MATCH;
//! опорные адреса и статусы — раздел «AI расписаний и поведение» там же.
//!
//! Слои владельцев (фильтры guild/faction/union, перечисление категорий и
//! применение цели) — свои zone-файлы конкретных AI.

use crate::regions::ShapeIdentity;
use crate::regions::shape::{ShapeAreaCoordinates, ShapeView};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GuardStationState {
    station: Option<ShapeAreaCoordinates>,
}

impl GuardStationState {
    pub const fn station(&self) -> Option<ShapeAreaCoordinates> {
        self.station
    }

    pub fn record_station(&mut self, owner: ShapeView) {
        self.station.get_or_insert(ShapeAreaCoordinates {
            x: owner.tile_x,
            y: owner.tile_y,
        });
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuardDistanceTarget {
    pub identity: ShapeIdentity,
    pub distance: i32,
}

pub fn consider_guard_distance_target(
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
pub fn select_guard_target_groups(
    first: Option<GuardDistanceTarget>,
    second: Option<GuardDistanceTarget>,
) -> Option<GuardDistanceTarget> {
    match (first, second) {
        (Some(first), Some(second)) if second.distance < first.distance => Some(second),
        (Some(first), _) => Some(first),
        (None, second) => second,
    }
}
