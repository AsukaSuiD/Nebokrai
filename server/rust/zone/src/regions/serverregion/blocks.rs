//! Динамическая block-разметка клеток региона и spatial shape-lookup
//! девяти-area окружения `CServerRegion` исторического GameServer (порция 2).
//! Исходный владелец — `appserver/serverregion.h/.cpp`; обе `RefeashBlock`
//! `0x0007FAD0/0x00081680` и обе `GetShape` `0x0007F390/0x00081300` имеют
//! статус `IMPLEMENTED, VERIFIED_DISASSEMBLY` исследовательского корпуса
//! старого файла; точная пара `GameServer/gameserver.exe +
//! GameServer/GameServer.pdb`.
//!
//! Block `3` — переоцениваемая пометка занятости клетки боевой фигурой:
//! refresh сначала снимает все блоки `3`, затем возвращает их живым
//! `CMoveShape` и NPC. Lookup обходит `NEIGHBOR_AREAS`, внутри области —
//! фиксированный порядок `get_all_shapes` (player, monster active/sleeping/
//! pets/carriages, goods, npc, other): `get_shape` выходит при первой
//! covering-фигуре, `get_shapes` собирает все. RTTI alive-факт concrete
//! `CMoveShape` остаётся у переходного владельца старого пакета и приходит
//! typed-access замыканием: `None` (неудачный downcast) пропускает фигуру.

use super::areagrid::get_area;
use super::geometry::{NEIGHBOR_AREAS, NPC_TYPE, shape_covers_tile};
use super::membership::{RegionMembershipBlock, validate_area_span};
use super::registry::{RegisteredShapeResolver, ServerRegionRegistry};
use crate::regions::ShapeIdentity;
use crate::regions::area::CArea;
use crate::regions::region::CRegion;
use crate::regions::shape::{ShapeResolver, ShapeView};

/// Тело exact `RefeashBlock()` (`0x0007FAD0`): полный проход сетки снимает
/// все динамические блоки `3`, затем row-major обход area-grid восстанавливает
/// их живым `CMoveShape` и NPC.
pub fn refresh_blocks<Resolver: ShapeResolver>(
    region: &mut CRegion,
    areas: &[CArea],
    registry: &ServerRegionRegistry,
    resolver: &Resolver,
    is_alive: &impl Fn(ShapeIdentity) -> Option<bool>,
) -> Result<(), RegionMembershipBlock> {
    let mut x = 0;
    while x < region.width {
        let mut y = 0;
        while y < region.height {
            if region
                .get_block(x, y)
                .map_err(RegionMembershipBlock::RegionCell)?
                == 3
            {
                region
                    .set_block(x, y, 0)
                    .map_err(RegionMembershipBlock::RegionCell)?;
            }
            y += 1;
        }
        x += 1;
    }

    let registered = RegisteredShapeResolver { registry, resolver };
    for area in areas {
        let mut shapes = Vec::new();
        area.get_all_shapes(&registered, &mut shapes);
        for shape in shapes {
            let Some(alive) = is_alive(shape.identity) else {
                continue;
            };
            if shape.identity.object_type != NPC_TYPE && !alive {
                continue;
            }
            region
                .set_block(shape.tile_x, shape.tile_y, 3)
                .map_err(RegionMembershipBlock::RegionCell)?;
        }
    }
    Ok(())
}

/// Тело exact `RefeashBlock(long, long)` (`0x00081680`): gate отсекает только
/// `< 0` и `> width/height` (граничные координаты разрешает внутренний
/// fallback `CRegion`), снимает одиночный блок `3` и восстанавливает его
/// живым `CMoveShape` и NPC через тот же девяти-area lookup, что `get_shapes`.
#[allow(
    clippy::too_many_arguments,
    reason = "literal RefeashBlock сохраняет исходные аргументы поверх owner-хранилищ переходного агрегата"
)]
pub fn refresh_block<Resolver: ShapeResolver>(
    region: &mut CRegion,
    areas: &[CArea],
    area_x: i32,
    area_y: i32,
    registry: &ServerRegionRegistry,
    tile_x: i32,
    tile_y: i32,
    area_width: i32,
    area_height: i32,
    resolver: &Resolver,
    is_alive: &impl Fn(ShapeIdentity) -> Option<bool>,
) -> Result<(), RegionMembershipBlock> {
    if tile_x < 0 || tile_y < 0 || tile_x > region.width || tile_y > region.height {
        return Ok(());
    }
    if region
        .get_block(tile_x, tile_y)
        .map_err(RegionMembershipBlock::RegionCell)?
        == 3
    {
        region
            .set_block(tile_x, tile_y, 0)
            .map_err(RegionMembershipBlock::RegionCell)?;
    }

    let mut shapes = Vec::new();
    get_shapes(
        areas,
        area_x,
        area_y,
        registry,
        tile_x,
        tile_y,
        area_width,
        area_height,
        resolver,
        &mut shapes,
    )?;
    for shape in shapes {
        let Some(alive) = is_alive(shape.identity) else {
            continue;
        };
        if shape.identity.object_type == NPC_TYPE || alive {
            region
                .set_block(shape.tile_x, shape.tile_y, 3)
                .map_err(RegionMembershipBlock::RegionCell)?;
        }
    }
    Ok(())
}

/// Тело exact `GetShape(long, long)` (`0x0007F390`): возвращает первую
/// covering-фигуру в порядке обхода `NEIGHBOR_AREAS` и `get_all_shapes`.
#[allow(
    clippy::too_many_arguments,
    reason = "literal GetShape сохраняет исходные аргументы поверх owner-хранилищ переходного агрегата"
)]
pub fn get_shape<Resolver: ShapeResolver>(
    areas: &[CArea],
    area_x: i32,
    area_y: i32,
    registry: &ServerRegionRegistry,
    tile_x: i32,
    tile_y: i32,
    area_width: i32,
    area_height: i32,
    resolver: &Resolver,
) -> Result<Option<ShapeView>, RegionMembershipBlock> {
    validate_area_span(area_width, area_height)?;
    let registered = RegisteredShapeResolver { registry, resolver };

    for (offset_x, offset_y) in NEIGHBOR_AREAS {
        let neighbor_x = (tile_x / area_width).wrapping_add(offset_x);
        let neighbor_y = (tile_y / area_height).wrapping_add(offset_y);
        let Some(area) = get_area(areas, area_x, area_y, neighbor_x, neighbor_y) else {
            continue;
        };
        let mut area_shapes = Vec::new();
        area.get_all_shapes(&registered, &mut area_shapes);
        for shape in area_shapes {
            if shape_covers_tile(shape, tile_x, tile_y) {
                return Ok(Some(shape));
            }
        }
    }
    Ok(None)
}

/// Тело vector-overload `GetShape` (`0x00081300`): собирает все covering
/// фигуры того же обхода в том же порядке.
#[allow(
    clippy::too_many_arguments,
    reason = "literal GetShape сохраняет исходные аргументы поверх owner-хранилищ переходного агрегата"
)]
pub fn get_shapes<Resolver: ShapeResolver>(
    areas: &[CArea],
    area_x: i32,
    area_y: i32,
    registry: &ServerRegionRegistry,
    tile_x: i32,
    tile_y: i32,
    area_width: i32,
    area_height: i32,
    resolver: &Resolver,
    destination: &mut Vec<ShapeView>,
) -> Result<(), RegionMembershipBlock> {
    validate_area_span(area_width, area_height)?;
    let registered = RegisteredShapeResolver { registry, resolver };

    for (offset_x, offset_y) in NEIGHBOR_AREAS {
        let neighbor_x = (tile_x / area_width).wrapping_add(offset_x);
        let neighbor_y = (tile_y / area_height).wrapping_add(offset_y);
        let Some(area) = get_area(areas, area_x, area_y, neighbor_x, neighbor_y) else {
            continue;
        };
        let mut area_shapes = Vec::new();
        area.get_all_shapes(&registered, &mut area_shapes);
        for shape in area_shapes {
            if shape_covers_tile(shape, tile_x, tile_y) {
                destination.push(shape);
            }
        }
    }
    Ok(())
}
