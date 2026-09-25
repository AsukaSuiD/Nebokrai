//! Transition-семья смены области `CServerRegion` исторического GameServer:
//! immutable plan (порция 1), staging-очереди area/region AI и ядро
//! plan/commit (порция 3). Исходный владелец — `appserver/serverregion.h/.cpp`;
//! `OnShapeChangeArea` `0x000802A0` имеет статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY` исследовательского корпуса старого файла; точная
//! пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`.
//!
//! Staging сохраняет pointer-unique append исходного region AI: marker
//! сбрасывается caller-ом только после добавления, ordered snapshot берётся
//! без очистки исходного list, cleanup очереди выполняет вызывающая сторона
//! после применения всех `OnShapeChangeArea`. Commit применяет исходный
//! порядок `RemoveObject -> AddObject -> m_pArea` (`0x0048076E/0x004807BC/
//! 0x004807C6`) до этого cleanup. Owned-monster/NPC обвязки plan/commit
//! остаются у переходного владельца generational hub-ов до своих волн.

use indexmap::IndexSet;

use super::areagrid::{area_index_by_coordinates, neighbor_area_indices};
use super::geometry::PLAYER_TYPE;
use super::registry::{RegisteredShapeResolver, ServerRegionRegistry};
use crate::regions::ShapeIdentity;
use crate::regions::area::CArea;
use crate::regions::shape::{
    CShape, ShapeAreaCoordinates, ShapeResolver, ShapeRuntimeFacts, ShapeView,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AreaTransitionBlock {
    StaleAreaIndex { index: usize, available: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AreaTransitionAudience {
    pub area_x: i32,
    pub area_y: i32,
    pub shapes_for_moving_player: Vec<ShapeView>,
}

/// Immutable effect-plan исходного `OnShapeChangeArea`. Регион вычисляет
/// exclusive areas и их ordered shape snapshots до membership mutation;
/// `CGame` исполняет клиентский wire, затем регион применяет target index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AreaTransitionPlan {
    pub moving: ShapeIdentity,
    pub current_index: usize,
    pub target_index: Option<usize>,
    pub audience: Vec<AreaTransitionAudience>,
}

/// Pointer-unique append area AI scan; marker сбрасывает caller только
/// после `true`, потому что duplicate исходник оставлял неизменным.
pub fn stage_area_transition(
    queue: &mut IndexSet<ShapeIdentity>,
    identity: ShapeIdentity,
) -> bool {
    queue.insert(identity)
}

/// Возвращает ordered snapshot, не очищая исходный list до применения всех
/// `OnShapeChangeArea`, как в конце original region AI.
pub fn staged_area_transitions(queue: &IndexSet<ShapeIdentity>) -> Vec<ShapeIdentity> {
    queue.iter().copied().collect()
}

pub fn clear_staged_area_transitions(queue: &mut IndexSet<ShapeIdentity>) {
    queue.clear()
}

pub fn stage_region_transition(
    queue: &mut IndexSet<ShapeIdentity>,
    identity: ShapeIdentity,
) -> bool {
    queue.insert(identity)
}

/// Atomic take очереди смены региона: исходный owner забирает ordered
/// snapshot и очищает list одним действием.
pub fn take_staged_region_transitions(queue: &mut IndexSet<ShapeIdentity>) -> Vec<ShapeIdentity> {
    std::mem::take(queue).into_iter().collect()
}

/// Планирующая часть exact `OnShapeChangeArea` до membership mutation:
/// gate отсутствующего owner-link и совпавшей области, stale index как
/// typed-граница размера grid, затем exclusive areas нового девяти-area
/// окружения в исходном порядке `NEIGHBOR_AREAS` и их ordered shape
/// snapshots для moving player (без самой фигуры).
pub fn plan_area_transition<Resolver: ShapeResolver>(
    areas: &[CArea],
    area_x: i32,
    area_y: i32,
    registry: &ServerRegionRegistry,
    shape: &CShape,
    resolver: &Resolver,
) -> Result<Option<AreaTransitionPlan>, AreaTransitionBlock> {
    let Some(current_index) = shape.area_index() else {
        return Ok(None);
    };
    let available = areas.len();
    let current_area = areas
        .get(current_index)
        .ok_or(AreaTransitionBlock::StaleAreaIndex {
            index: current_index,
            available,
        })?;
    let current = ShapeAreaCoordinates {
        x: current_area.x(),
        y: current_area.y(),
    };
    let next = shape.next_area_coordinates();
    if current == next {
        return Ok(None);
    }

    let old_neighbors = neighbor_area_indices(area_x, area_y, current);
    let mut new_exclusive = neighbor_area_indices(area_x, area_y, next);
    new_exclusive.retain(|index| !old_neighbors.contains(index));

    let moving = shape.identity();
    let registered = RegisteredShapeResolver { registry, resolver };
    let mut audience = Vec::new();
    for area_index in new_exclusive {
        let area = &areas[area_index];
        if area.get_num_shapes() == 0 {
            continue;
        }
        let mut shapes_for_moving_player = Vec::new();
        if moving.object_type == PLAYER_TYPE {
            area.get_all_shapes(&registered, &mut shapes_for_moving_player);
            shapes_for_moving_player.retain(|shape| shape.identity != moving);
        }
        audience.push(AreaTransitionAudience {
            area_x: area.x(),
            area_y: area.y(),
            shapes_for_moving_player,
        });
    }

    Ok(Some(AreaTransitionPlan {
        moving,
        current_index,
        target_index: area_index_by_coordinates(area_x, area_y, next),
        audience,
    }))
}

/// Committing-часть `OnShapeChangeArea`: `false` без target area, иначе
/// исходный порядок `RemoveObject -> AddObject -> m_pArea` над текущей и
/// целевой областью plan-а. Очистка staged-очередей остаётся caller-у.
pub fn commit_area_transition(
    areas: &mut [CArea],
    shape: &mut CShape,
    facts: ShapeRuntimeFacts,
    now_ms: u32,
    plan: &AreaTransitionPlan,
) -> bool {
    let Some(target_index) = plan.target_index else {
        return false;
    };
    areas[plan.current_index].remove_object(plan.moving, facts);
    areas[target_index].add_object(plan.moving, facts, now_ms);
    shape.set_area_index(Some(target_index));
    true
}
