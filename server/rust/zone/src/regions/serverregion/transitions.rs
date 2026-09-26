//! Transition-семья смены области `CServerRegion`: immutable plan,
//! staging-очереди area/region AI и ядро plan/commit. Исходный владелец —
//! `appserver/serverregion.h/.cpp`; точная пара `gameserver.exe` +
//! `GameServer/GameServer.pdb` (идентификаторы сборки —
//! `server/rust/src/manifest/_gameserver_export_manifest.toml`; совпадение
//! подтверждено оснасткой `.local/evidence/symbols.py identity`). Машинные
//! статусы (прямой дизассембл тел точной пары):
//!
//! | функция | RVA | статус |
//! |---|---|---|
//! | `plan_area_transition` | `0x000802A0` | `VERIFIED_DISASSEMBLY` |
//! | `commit_area_transition` | `0x000802A0` | `VERIFIED_DISASSEMBLY` |
//! | staging-очереди (`stage_*`, `staged_area_transitions`, `take_staged_region_transitions`) | — | `PARTIAL`: staging-сайт исходного region AI отдельным телом не дизассемблировался |
//!
//! `OnShapeChangeArea` сверен по всему телу (`0x004802A0-0x004808A2`): gate
//! null-объекта и отсутствующего owner-link `[+0x60]` (`0x004802C4-
//! 0x004802D7`), gate совпавшей области по `[+0x68]/[+0x6C]` против
//! `[+0x44]/[+0x48]` (`0x004802DD-0x004802F7`), обход девяти-area окружения
//! по таблице смещений `0x69FC38`/границей `0x69FC80` в исходном порядке
//! (center-first row-major, значения совпадают с `NEIGHBOR_AREAS`),
//! list-разность «новые без старых» через copy-ctor и `list::remove`
//! (`0x0048046E-0x00480497`), skip пустых областей по `GetNumShapes`
//! `0x00070B80` (`0x004805A5`), audience только для moving player
//! (`0x004805C3` по `[+0x4] == 0x190`) с исключением самой фигуры
//! (`0x00480623`), ordered per-area snapshots до membership mutation, затем
//! commit-порядок `RemoveObject -> GetArea(next) -> AddObject -> m_pArea`
//! (`0x0048076E/0x00480771-0x004807B5/0x004807BC/0x004807C6`) и player-only
//! `PlayerEnter` `0x00075580` (`0x004807D1`).
//!
//! Частичный эффект пути ненайденной цели, сверенный по телу: оригинал
//! выполняет `RemoveObject` прежней области
//! (`0x0048076E`) до bounds-check цели (`GetArea(next)`
//! `0x00480771-0x004807B5`) и завершается без `AddObject`
//! (`0x004807BC`) и без сброса `m_pArea` (запись `0x004807C6` только на
//! найденном пути; owner-link остаётся указывать на область, из которой
//! фигура удалена). Ядро `commit_area_transition` исполняет этот порядок
//! и докладывает частичный эффект вариантом
//! `AreaTransitionOutcome::RemovedWithoutTargetArea`; обвязка старого
//! пакета отображает его в прежнее булево «отказ», хуков и логирования на
//! этой ветке машинное тело не содержит.
//!
//! Staging сохраняет pointer-unique append исходного region AI: marker
//! сбрасывается caller-ом только после добавления, ordered snapshot берётся
//! без очистки исходного list, cleanup очереди выполняет вызывающая сторона
//! после применения всех `OnShapeChangeArea`. Owned-monster/NPC обвязки
//! plan/commit остаются у переходного владельца generational hub-ов.

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

/// Outcome committing-части `OnShapeChangeArea`. Машинный порядок
/// эффектов: `RemoveObject` прежней области (`0x0048076E`) стоит до
/// `GetArea(next)`/bounds-check цели (`0x00480771-0x004807B5`), а
/// `AddObject` (`0x004807BC`) и запись owner-link `m_pArea`
/// (`0x004807C6`) выполняются только на найденном пути. Хуков и
/// логирования на ветке ненайденной цели машинное тело не содержит.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AreaTransitionOutcome {
    /// Целевая область найдена: исходный порядок `RemoveObject ->
    /// AddObject -> m_pArea` выполнен целиком.
    Committed,
    /// Целевая область не найдена: прежняя область уже выполнила
    /// `RemoveObject` (`0x0048076E`), без `AddObject` и без сброса
    /// `m_pArea` (`0x004807C6` только на найденном пути) — owner-link
    /// остаётся указывать на область, из которой фигура удалена, как в
    /// оригинале.
    RemovedWithoutTargetArea,
}

/// Committing-часть `OnShapeChangeArea`: `RemoveObject` прежней области
/// до проверки target area в исходном порядке (`0x0048076E` до
/// `0x00480771-0x004807B5`); при ненайденной цели фигура остаётся снятой
/// с висячим owner-link (`RemovedWithoutTargetArea`), иначе `AddObject ->
/// m_pArea` (`0x004807BC/0x004807C6`). Очистка staged-очередей остаётся
/// caller-у.
pub fn commit_area_transition(
    areas: &mut [CArea],
    shape: &mut CShape,
    facts: ShapeRuntimeFacts,
    now_ms: u32,
    plan: &AreaTransitionPlan,
) -> AreaTransitionOutcome {
    areas[plan.current_index].remove_object(plan.moving, facts);
    let Some(target_index) = plan.target_index else {
        return AreaTransitionOutcome::RemovedWithoutTargetArea;
    };
    areas[target_index].add_object(plan.moving, facts, now_ms);
    shape.set_area_index(Some(target_index));
    AreaTransitionOutcome::Committed
}
