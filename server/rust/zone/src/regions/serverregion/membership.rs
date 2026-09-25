//! Typed-отказ пространственного членства `CServerRegion` исторического
//! GameServer (порция 1) и ядра его membership-операций (порция 3): вход и
//! выход фигуры и owner-обвязки позиционной регистрации move-shape. Исходный
//! владелец — `appserver/serverregion.h/.cpp`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb` (SHA-256 EXE
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, RSDS
//! `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, совпадение подтверждено
//! оснасткой `.local/evidence/symbols.py identity`). Машинные статусы порций
//! (прямой дизассембл тел точной пары):
//!
//! | функция | RVA | статус |
//! |---|---|---|
//! | `add_object_with_area_entry` | `0x00083270` | `VERIFIED_DISASSEMBLY` |
//! | `remove_object` | `0x0007CE60` | `VERIFIED_DISASSEMBLY` |
//! | `set_move_shape_position` / `set_move_shape_tile_position` | — | glue-обвязка над ядром `CShape::SetTileXY`; сама обвязка сверена с местами вызова membership-тел |
//!
//! `add_object_with_area_entry` сверен по всему телу (`0x00483270-0x00483570`,
//! vtable `+0x38`): RTTI-downcast `0x004832A0`, owner-link `[+0x40]`
//! `0x004832B4`, чтение tile X (`0x004832B7`) до tile Y (`0x004832C2`),
//! границы по `[+0x6C]/[+0x70]` (`0x004832D3-0x004832DF`), random-fallback
//! только при ненулевом cell-array `[+0x84]` (`0x004832E1`) через
//! `GetRandomPos` `0x000F04D0` (`0x004832F6`), затем virtual `SetTileXY`
//! `+0x88` (`0x0048330A`), registry-add switch по типам `0x190/0x1F4/0x258/
//! 0x2BC` (`0x00483313-0x00483413`), индекс области `idiv` глобалями
//! area-span `15` из `0x69EFC8/0x69EFCC` (`0x00483448/0x00483456`),
//! `GetArea` `0x0007BB60` (`0x0048345D`), virtual `AddObject` области `+0x38`
//! (`0x0048346D`), запись `m_pArea` `[+0x60]` (`0x00483477`), player-only
//! `PlayerEnter` `0x00075580` (`0x00483482`), entry-hook virtual `+0x150`
//! (`0x004834A6`) либо fallback — тот же virtual `RemoveObject` `+0x34` на
//! себе (`0x004834B2`).
//!
//! `remove_object` сверен по всему телу (`0x0047CE60-0x0047D08C`, vtable
//! `+0x34`): area-remove virtual `+0x34` области (`0x0047CE9A`) и сброс
//! owner-link `m_pArea = 0` (`0x0047CE9D`) до стирания блока клетки;
//! player-ветка вызывает `SetBlock(x, y, 0)` virtual `+0x90` (`0x0047CEEC`)
//! и достигнутый player-leave virtual `0x00601A70` — точный `ret 4` без
//! наблюдаемого эффекта (`0x0047CEFC`); NPC/monster-ветка — тот же
//! `SetBlock(x, y, 0)` (`0x0047CECE`) без stub-вызова; прочие типы блок не
//! стирают (`0x0047CEB3/0x0047CEB6` → `0x0047CF01`); registry-erase по типам
//! в конце (`0x0047CF04-0x0047D050`).
//!
//! Ядра сохраняют исходный порядок частичных эффектов. Entry-effects входа
//! (`before entry`-hook и virtual `AfterEnteredArea`) остаются у переходной
//! обвязки старого пакета, которая по-прежнему получает весь агрегат
//! `CServerRegion`; ядро возвращает решение typed-outcome. Owner-обвязки
//! позиционной регистрации дополняют area facts и выбирают достигнутый
//! dispatch `CShape::SetTileXY`, не дублируя его ядро. Внутренности
//! `GetRandomPos` `0x000F04D0` этой волной не дизассемблировались: при
//! неудаче оригинал молча продолжает с записанными out-координатами, ядро
//! поднимает typed-границу — отличие зафиксировано.

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
