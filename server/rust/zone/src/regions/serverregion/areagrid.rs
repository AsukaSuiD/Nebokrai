//! Построение и доступ к area-grid `CServerRegion` вместе с картами боевых
//! духов. Исходный владелец — `appserver/serverregion.h/.cpp`; точная пара
//! `gameserver.exe` + `GameServer.pdb`. Статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`: обе `GetArea` `0x00001DB0/0x0007BB60` и
//! `CreateAreaArray` `0x0007BE10`, spatial tail `CPlayer::SetWarSoulXY/
//! DelWarSoul` `0x0042DF50/0x0042E0A0`.
//!
//! Для положительных размеров grid использует исходное ceiling-деление и
//! row-major `area_x * y + x`; старые zero/negative span и 32-bit allocation
//! overflow становятся локальной typed-границей, а не platform-dependent
//! trap/UB. Старый unbounded `GetArea(index)` заменён `Result`, coordinate
//! overload сохраняет доказанный `nullptr -> Option`. Переходный агрегат
//! `CServerRegion` старого пакета держит сами хранилища и делегирует им
//! каждую операцию ниже с прежними сигнатурами методов.

use std::collections::BTreeMap;

use super::geometry::{NEIGHBOR_AREAS, WAR_SOUL_AREA_SPAN, ceil_positive_division};
use crate::regions::area::{CArea, WarSoulPoint};
use crate::regions::region::CRegion;
use crate::regions::shape::ShapeAreaCoordinates;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AreaGridBlock {
    InvalidAreaSpan { width: i32, height: i32 },
    InvalidRegionDimensions { width: i32, height: i32 },
    GridSizeOverflow { area_x: i32, area_y: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AreaIndexBlock {
    pub index: i32,
    pub available: usize,
}

/// Тело точного `CServerRegion::CreateAreaArray` над хранилищами переходного
/// владельца: сначала пишутся новые `area_x/area_y`, затем массив областей
/// пересоздаётся; typed-отказ после записи dimensions сохраняет исходную
/// частичную мутацию.
#[allow(
    clippy::too_many_arguments,
    reason = "literal CreateAreaArray сохраняет исходные аргументы и исходный порядок частичной мутации owner-хранилищ"
)]
pub fn create_area_array(
    areas: &mut Vec<CArea>,
    area_x: &mut i32,
    area_y: &mut i32,
    region_width: i32,
    region_height: i32,
    area_width: i32,
    area_height: i32,
) -> Result<(), AreaGridBlock> {
    if area_width <= 0 || area_height <= 0 {
        // BLOCKED_MISSING_FACT: x86 `idiv` trap для нуля и последующая
        // signed allocation для отрицательного span не задают safe contract.
        return Err(AreaGridBlock::InvalidAreaSpan {
            width: area_width,
            height: area_height,
        });
    }
    if region_width < 0 || region_height < 0 {
        return Err(AreaGridBlock::InvalidRegionDimensions {
            width: region_width,
            height: region_height,
        });
    }

    *area_x = ceil_positive_division(region_width, area_width);
    *area_y = ceil_positive_division(region_height, area_height);
    areas.clear();

    let Some(area_count_i32) = (*area_x).checked_mul(*area_y) else {
        return Err(AreaGridBlock::GridSizeOverflow {
            area_x: *area_x,
            area_y: *area_y,
        });
    };
    let Ok(area_count) = usize::try_from(area_count_i32) else {
        return Err(AreaGridBlock::GridSizeOverflow {
            area_x: *area_x,
            area_y: *area_y,
        });
    };
    if area_count > (u32::MAX as usize - 4) / 0x118 {
        return Err(AreaGridBlock::GridSizeOverflow {
            area_x: *area_x,
            area_y: *area_y,
        });
    }

    areas.resize_with(area_count, CArea::with_storage_defaults);
    let mut x = 0;
    while x < *area_x {
        let mut y = 0;
        while y < *area_y {
            let index = usize::try_from(*area_x * y + x).expect("положительные grid dimensions");
            areas[index].assign_to_server_region(x, y);
            y += 1;
        }
        x += 1;
    }
    Ok(())
}

/// Safe-граница исходного unbounded `GetArea(long)`.
pub fn get_area_by_index(areas: &[CArea], index: i32) -> Result<&CArea, AreaIndexBlock> {
    let Ok(index_usize) = usize::try_from(index) else {
        return Err(AreaIndexBlock {
            index,
            available: areas.len(),
        });
    };
    areas.get(index_usize).ok_or(AreaIndexBlock {
        index,
        available: areas.len(),
    })
}

/// Сохраняет bounds-check и `nullptr` coordinate-overload-а.
pub fn get_area(areas: &[CArea], area_x: i32, area_y: i32, x: i32, y: i32) -> Option<&CArea> {
    if x < 0 || x >= area_x || y < 0 || y >= area_y {
        return None;
    }
    let index = usize::try_from(area_x * y + x).expect("положительный grid index");
    areas.get(index)
}

pub fn block_at(region: &CRegion, x: i32, y: i32) -> Option<u8> {
    region.get_block(x, y).ok()
}

/// Mutable counterpart точного coordinate-overload `GetArea`; нужен
/// только owner-у war-soul map, который уже владеет всем area-grid.
pub fn get_area_mut(
    areas: &mut [CArea],
    area_x: i32,
    area_y: i32,
    x: i32,
    y: i32,
) -> Option<&mut CArea> {
    if x < 0 || x >= area_x || y < 0 || y >= area_y {
        return None;
    }
    let index = usize::try_from(area_x * y + x).expect("положительный grid index");
    areas.get_mut(index)
}

pub fn has_war_soul_area(areas: &[CArea], area_x: i32, area_y: i32, point: WarSoulPoint) -> bool {
    get_area(
        areas,
        area_x,
        area_y,
        point.x / WAR_SOUL_AREA_SPAN,
        point.y / WAR_SOUL_AREA_SPAN,
    )
    .is_some()
}

/// Материализует spatial tail `CPlayer::SetWarSoulXY`: target area должна
/// существовать; old entry очищается лишь если её area присутствует, после
/// чего точка добавляется в target map. Деление на `15` — literal `idiv
/// 0xF` из owner-а. Возвращается наличие target area, не результат AddWarSoul.
pub fn set_war_soul_position(
    areas: &mut [CArea],
    area_x: i32,
    area_y: i32,
    player_id: u32,
    previous: WarSoulPoint,
    target: WarSoulPoint,
) -> bool {
    let target_x = target.x / WAR_SOUL_AREA_SPAN;
    let target_y = target.y / WAR_SOUL_AREA_SPAN;
    if !has_war_soul_area(areas, area_x, area_y, target) {
        return false;
    }

    let previous_x = previous.x / WAR_SOUL_AREA_SPAN;
    let previous_y = previous.y / WAR_SOUL_AREA_SPAN;
    if let Some(area) = get_area_mut(areas, area_x, area_y, previous_x, previous_y) {
        let _legacy_result = area.del_war_soul(player_id, previous);
    }
    let _legacy_result = get_area_mut(areas, area_x, area_y, target_x, target_y)
        .expect("проверенная target area остаётся в том же grid")
        .add_war_soul(player_id, target);
    true
}

/// `CPlayer::DelWarSoul` сбрасывает player point только при найденной area,
/// независимо от результата удаления прежней записи из её карты.
pub fn delete_war_soul(
    areas: &mut [CArea],
    area_x: i32,
    area_y: i32,
    player_id: u32,
    point: WarSoulPoint,
) -> bool {
    let Some(area) = get_area_mut(
        areas,
        area_x,
        area_y,
        point.x / WAR_SOUL_AREA_SPAN,
        point.y / WAR_SOUL_AREA_SPAN,
    ) else {
        return false;
    };
    let _legacy_result = area.del_war_soul(player_id, point);
    true
}

/// Точный `GetWarSoulXY`: упорядоченная карта одной области боевого духа
/// фильтруется по координатам и сохраняет возрастание идентификаторов игроков.
pub fn war_souls_at(
    areas: &[CArea],
    area_x: i32,
    area_y: i32,
    x: i32,
    y: i32,
) -> BTreeMap<u32, WarSoulPoint> {
    let mut found = BTreeMap::new();
    if let Some(area) = get_area(
        areas,
        area_x,
        area_y,
        x / WAR_SOUL_AREA_SPAN,
        y / WAR_SOUL_AREA_SPAN,
    ) {
        area.find_war_souls(&mut found);
    }
    found.retain(|_, point| point.x == x && point.y == y);
    found
}

/// Индексы действительных областей исходного девяти-area окружения.
pub fn neighbor_area_indices(
    area_x: i32,
    area_y: i32,
    center: ShapeAreaCoordinates,
) -> Vec<usize> {
    let mut indices = Vec::new();
    for (offset_x, offset_y) in NEIGHBOR_AREAS {
        let coordinates = ShapeAreaCoordinates {
            x: center.x.wrapping_add(offset_x),
            y: center.y.wrapping_add(offset_y),
        };
        if let Some(index) = area_index_by_coordinates(area_x, area_y, coordinates) {
            indices.push(index);
        }
    }
    indices
}

pub fn area_index_by_coordinates(
    area_x: i32,
    area_y: i32,
    coordinates: ShapeAreaCoordinates,
) -> Option<usize> {
    if coordinates.x < 0 || coordinates.x >= area_x || coordinates.y < 0 || coordinates.y >= area_y
    {
        return None;
    }
    usize::try_from(area_x * coordinates.y + coordinates.x).ok()
}

/// Индекс области по клетке тайла и area-span; несогласованный span
/// отсекается typed-границей вызывающей стороны до этого вызова.
pub fn area_index_for_tile(
    area_x: i32,
    area_y: i32,
    tile_x: i32,
    tile_y: i32,
    area_width: i32,
    area_height: i32,
) -> Option<usize> {
    let tile_area_x = tile_x / area_width;
    let tile_area_y = tile_y / area_height;
    if tile_area_x < 0 || tile_area_x >= area_x || tile_area_y < 0 || tile_area_y >= area_y {
        return None;
    }
    usize::try_from(area_x * tile_area_y + tile_area_x).ok()
}
