//! Общий узкий выбор цели охранников с минимальной дистанцией текущего навыка.
//!
//! Несколько подтверждённых AI-owner-ов используют одинаковый порядок: внутри
//! отдельной категории предпочитается ближайшая цель не ближе минимальной
//! дистанции, а при отсутствии такой цели остаётся последняя слишком близкая
//! запись. Фильтры страны, фракции и союза остаются у конкретных владельцев.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Дистанционное ядро разобрано построчно по телам
//! `CCityGuardWithSword::SearchEnemyGuildMember` (VA `0x0060E350`, ветки
//! замены при equal/compare и минимальной дистанции навыка vt `+0x70`),
//! `SearchEnemyGuildPet` (VA `0x0060E510`) и общему выбору
//! `CCityGuardWithSword::OnSearchEnemy` (VA `0x0060E290`):
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | категорийный выбор: ближайший не ближе минимальной дистанции навыка; слишком близкая сохранённая заменяется следующей записью | ветка compare в VA `0x0060E350` | [`consider_guard_distance_target`] | `MATCH` |
//! | первый категорийный список выигрывает при равной дистанции (игрок перед питомцем) | VA `0x0060E290`, также `0x0060DA90`/`0x0060DB10` | [`select_guard_target_groups`] | `MATCH` |
//! | точка поста охранника фиксируется один раз и далее только читается | `m_lX`/`m_lY` владельца семьи VA `0x0060E290` | [`GuardStationState`] | `MATCH` (по прежнему владельцу) |
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
