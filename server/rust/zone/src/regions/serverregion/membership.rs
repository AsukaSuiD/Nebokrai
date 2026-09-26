//! Typed-отказ пространственного членства `CServerRegion` и ядра его
//! membership-операций: вход и выход фигуры и owner-обвязки позиционной
//! регистрации move-shape. Исходный владелец — `appserver/serverregion.h/.cpp`;
//! сверка по точной паре `gameserver.exe` + `GameServer.pdb`.
//!
//! Ядра сохраняют исходный порядок частичных эффектов: owner-link `[+0x40]`,
//! random-fallback только при ненулевом cell-array, registry-add switch по
//! типам, запись `m_pArea` до entry-hook с fallback-удалением на себе; выход
//! сбрасывает `m_pArea` до стирания блока клетки, а прочие типы блок не стирают.
//! Entry-effects входа (`before entry`-hook и virtual `AfterEnteredArea`)
//! остаются у переходной обвязки старого пакета, получающей весь агрегат;
//! ядро возвращает решение typed-outcome. Внутренности `GetRandomPos` не
//! дизассемблировались: его молчаливое продолжение при неудаче здесь поднимается
//! typed-границей — отличие зафиксировано.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#региональное-пространство

use super::areagrid::area_index_for_tile;
use super::geometry::{MONSTER_TYPE, NPC_TYPE, PLAYER_TYPE};
use super::registry::ServerRegionRegistry;
use crate::regions::area::CArea;
use crate::regions::moveshape::{
    MoveShapePositionBlock, MoveShapePositionDispatch, MoveShapePositionFacts,
};
use crate::regions::region::{CRegion, RegionCellAccessBlock, RegionRandomContext};
use crate::regions::shape::{
    BaseShapePositionDispatch, CShape, ShapeAreaCoordinates, ShapeBlockError,
    ShapeCoordinateBlock, ShapePositionDispatch, ShapeRuntimeFacts,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegionMembershipBlock {
    InvalidAreaSpan { width: i32, height: i32 },
    StaleAreaIndex { index: usize, available: usize },
    ShapeCoordinate(ShapeCoordinateBlock),
    ShapeBlock(ShapeBlockError),
    RegionCell(RegionCellAccessBlock),
    MoveShape(MoveShapePositionBlock),
}

pub fn validate_area_span(width: i32, height: i32) -> Result<(), RegionMembershipBlock> {
    if width <= 0 || height <= 0 {
        // BLOCKED_MISSING_FACT: zero вызывает x86 `idiv` trap, negative
        // GlobeSetup span не имеет доказанного переносимого runtime contract.
        return Err(RegionMembershipBlock::InvalidAreaSpan { width, height });
    }
    Ok(())
}

/// Решение ядра `AddObject` без entry-effects: найденная целевая область
/// входа либо уже выполненный fallback исходного virtual `RemoveObject`.
/// Переходная обвязка по `EnteredArea` вызывает hook перед входом и virtual
/// `AfterEnteredArea` в исходном порядке над всем переходным агрегатом.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AreaEntryOutcome {
    EnteredArea { area_index: usize },
    RemovedWithoutArea,
}

/// Тело `CServerRegion::AddObject(CBaseObject*)` (`0x00083270`) над
/// хранилищами переходного владельца с иным концом: после установки owner
/// area-link вместо hook-ов ядро возвращает `AreaEntryOutcome`. RTTI-факт
/// derived owner-а (`member` старой сигнатуры) приходит готовым `CShape` и
/// `ShapeRuntimeFacts`; downcast остаётся у обвязки старого пакета.
#[allow(
    clippy::too_many_arguments,
    reason = "literal AddObject сохраняет исходные аргументы поверх owner-хранилищ переходного агрегата"
)]
pub fn add_object_with_area_entry<Context: RegionRandomContext>(
    region: &mut CRegion,
    areas: &mut [CArea],
    area_x: i32,
    area_y: i32,
    registry: &mut ServerRegionRegistry,
    region_id: i32,
    shape: &mut CShape,
    facts: ShapeRuntimeFacts,
    area_width: i32,
    area_height: i32,
    now_ms: u32,
    context: &mut Context,
) -> Result<AreaEntryOutcome, RegionMembershipBlock> {
    validate_area_span(area_width, area_height)?;
    let mut tile_x = shape
        .get_tile_x()
        .map_err(RegionMembershipBlock::ShapeCoordinate)?;
    let mut tile_y = shape
        .get_tile_y()
        .map_err(RegionMembershipBlock::ShapeCoordinate)?;
    shape.assign_to_server_region();
    shape.set_region_id(region_id);

    if (tile_x < 0 || tile_x >= region.width || tile_y < 0 || tile_y >= region.height)
        && !region.cells.is_empty()
    {
        let position = region
            .get_random_pos(context)
            .map_err(RegionMembershipBlock::RegionCell)?;
        tile_x = position.x;
        tile_y = position.y;
    }
    if facts.is_move_shape {
        let mut dispatch = MoveShapePositionDispatch {
            facts: MoveShapePositionFacts {
                current_hit_points: u32::from(facts.blocks_region_cell),
                figure: facts.figure,
                current_area: None,
                area_width,
                area_height,
            },
        };
        shape
            .set_tile_xy(region, tile_x, tile_y, &mut dispatch)
            .map_err(RegionMembershipBlock::MoveShape)?;
    } else {
        shape
            .set_tile_xy(region, tile_x, tile_y, &mut BaseShapePositionDispatch)
            .expect("базовая запись координат CShape не может завершиться ошибкой");
    }

    let identity = shape.identity();
    registry.add(identity, facts);
    let area_index = area_index_for_tile(area_x, area_y, tile_x, tile_y, area_width, area_height);
    if let Some(area_index) = area_index {
        areas[area_index].add_object(identity, facts, now_ms);
        shape.set_area_index(Some(area_index));
        Ok(AreaEntryOutcome::EnteredArea { area_index })
    } else {
        remove_object(region, areas, registry, shape, facts)?;
        Ok(AreaEntryOutcome::RemovedWithoutArea)
    }
}

/// Тело `CServerRegion::RemoveObject(CBaseObject*)` (`0x0007CE60`): сначала
/// area-remove и сброс owner-link, затем стирание блока клетки для
/// player/NPC/monster, в конце registry remove. Достигнутый player-leave
/// virtual — точный `ret 4` и наблюдаемого эффекта не имеет.
pub fn remove_object(
    region: &mut CRegion,
    areas: &mut [CArea],
    registry: &mut ServerRegionRegistry,
    shape: &mut CShape,
    facts: ShapeRuntimeFacts,
) -> Result<(), RegionMembershipBlock> {
    let identity = shape.identity();
    if let Some(area_index) = shape.area_index() {
        let available = areas.len();
        let area =
            areas
                .get_mut(area_index)
                .ok_or(RegionMembershipBlock::StaleAreaIndex {
                    index: area_index,
                    available,
                })?;
        area.remove_object(identity, facts);
        shape.set_area_index(None);

        if matches!(identity.object_type, PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE) {
            let tile_x = shape
                .get_tile_x()
                .map_err(RegionMembershipBlock::ShapeCoordinate)?;
            let tile_y = shape
                .get_tile_y()
                .map_err(RegionMembershipBlock::ShapeCoordinate)?;
            shape
                .set_block(region, tile_x, tile_y, 0, facts.figure)
                .map_err(RegionMembershipBlock::ShapeBlock)?;
        }
    }
    registry.remove(identity);
    Ok(())
}

/// Дополняет `current_area` из owner-link фигуры для dispatch
/// `CMoveShape::SetPosXY`; stale index — typed-граница размера area-grid.
fn complete_move_shape_position_facts(
    areas: &[CArea],
    shape: &CShape,
    mut facts: MoveShapePositionFacts,
) -> Result<MoveShapePositionFacts, RegionMembershipBlock> {
    facts.current_area = match shape.area_index() {
        Some(index) => {
            let available = areas.len();
            let area = areas
                .get(index)
                .ok_or(RegionMembershipBlock::StaleAreaIndex { index, available })?;
            Some(ShapeAreaCoordinates {
                x: area.x(),
                y: area.y(),
            })
        }
        None => None,
    };
    Ok(facts)
}

/// Owner-обвязка достигнутых movement commands: регион дополняет area facts
/// и достигает `CMoveShape::SetPosXY` через dispatch, не дублируя его ядро и
/// не перекладывая игровые spatial-факты в процессный runtime.
pub fn set_move_shape_position(
    region: &mut CRegion,
    areas: &[CArea],
    shape: &mut CShape,
    x: f32,
    y: f32,
    facts: MoveShapePositionFacts,
) -> Result<(), RegionMembershipBlock> {
    let facts = complete_move_shape_position_facts(areas, shape, facts)?;
    let mut dispatch = MoveShapePositionDispatch { facts };
    dispatch
        .set_pos_xy(region, shape, x, y)
        .map_err(RegionMembershipBlock::MoveShape)
}

/// Tile-вариант owner-обвязки: тот же дополненный facts и достигнутый
/// `CShape::SetTileXY` владельца с его tile-center `+0.5`.
pub fn set_move_shape_tile_position(
    region: &mut CRegion,
    areas: &[CArea],
    shape: &mut CShape,
    tile_x: i32,
    tile_y: i32,
    facts: MoveShapePositionFacts,
) -> Result<(), RegionMembershipBlock> {
    let facts = complete_move_shape_position_facts(areas, shape, facts)?;
    let mut dispatch = MoveShapePositionDispatch { facts };
    shape
        .set_tile_xy(region, tile_x, tile_y, &mut dispatch)
        .map_err(RegionMembershipBlock::MoveShape)
}
