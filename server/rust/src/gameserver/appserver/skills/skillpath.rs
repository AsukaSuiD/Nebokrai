//! Прямой путь навыка и снимок блоков карты.
//! Источник: gameserver.exe/GameServer.pdb, appserver/states/skill.cpp.
//!
//! Геометрия создаётся до поиска региона: отсутствующая карта оставляет клетки
//! BLOCK_UNFLY, а не пустой путь. Разности и сумма квадратов имеют signed DWORD
//! wrapping; длина округляется вверх только при остатке строго больше 0.5.
//! Деление выполняется до записи шага в float, каждый шаг координат сохраняется
//! в float отдельно. Vec заменяет владение исходного vector без смены порядка.

use super::fightdefense::truncate_original;
use crate::gameserver::appserver::serverregion::CServerRegion;

fn round_path_coordinate(value: f64) -> i32 {
    let truncated = truncate_original(value);
    if value - f64::from(truncated) > 0.5 { truncated.wrapping_add(1) } else { truncated }
}

pub(crate) fn straight_skill_path(
    region: Option<&CServerRegion>, source_x: i32, source_y: i32,
    target_x: i32, target_y: i32, forced_length: Option<u32>,
) -> Vec<(i32, i32, u8)> {
    let delta_x = target_x.wrapping_sub(source_x);
    let delta_y = target_y.wrapping_sub(source_y);
    let square = delta_x.wrapping_mul(delta_x).wrapping_add(delta_y.wrapping_mul(delta_y));
    let computed_length = round_path_coordinate(f64::from(square).sqrt());
    let path_length = forced_length.unwrap_or_else(|| computed_length.max(0) as u32);
    let step_x = (f64::from(delta_x) / f64::from(computed_length)) as f32;
    let step_y = (f64::from(delta_y) / f64::from(computed_length)) as f32;
    let mut cursor_x = source_x as f32;
    let mut cursor_y = source_y as f32;
    let mut path = Vec::with_capacity(path_length as usize);
    for _ in 0..path_length {
        cursor_x += step_x;
        cursor_y += step_y;
        let x = round_path_coordinate(f64::from(cursor_x));
        let y = round_path_coordinate(f64::from(cursor_y));
        path.push((x, y, 2));
    }
    if let Some(region) = region {
        for cell in &mut path {
            cell.2 = region.skill_cell_block(cell.0, cell.1);
        }
    }
    path
}
