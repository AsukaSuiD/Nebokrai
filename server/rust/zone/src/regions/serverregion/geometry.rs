//! Константы геометрии и чистая арифметика `CServerRegion`. Исходный
//! владелец — `appserver/serverregion.h/.cpp`; точная пара `gameserver.exe`
//! + `GameServer.pdb`.
//!
//! Legacy типы объектов `400/500/600/700` выбирают хранилище region registry
//! и классификацию RTTI-фактов; war-soul span `15` — literal `idiv 0xF` из
//! owner-а `CPlayer::SetWarSoulXY/DelWarSoul`. Точная 49-cell таблица
//! `GetDropGoodsPos` и девять соседних областей сохраняют исходный порядок
//! обхода. `CITY_STATE_*` — значения поля `m_CityState +0x23C`.

use crate::regions::shape::ShapeView;

pub const PLAYER_TYPE: i32 = 400;
pub const NPC_TYPE: i32 = 500;
pub const MONSTER_TYPE: i32 = 600;
pub const GOODS_TYPE: i32 = 700;
pub const WAR_SOUL_AREA_SPAN: i32 = 15;

pub const DROP_GOODS_OFFSETS: [(i32, i32); 49] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
    (-2, -2),
    (-1, -2),
    (0, -2),
    (1, -2),
    (2, -2),
    (-2, -1),
    (2, -1),
    (-2, 0),
    (2, 0),
    (-2, 1),
    (2, 1),
    (-2, 2),
    (-1, 2),
    (0, 2),
    (1, 2),
    (2, 2),
    (-3, -3),
    (-2, -3),
    (-1, -3),
    (0, -3),
    (1, -3),
    (2, -3),
    (3, -3),
    (-3, -2),
    (3, -2),
    (-3, -1),
    (3, -1),
    (-3, 0),
    (3, 0),
    (-3, 1),
    (3, 1),
    (-3, 2),
    (3, 2),
    (-3, 3),
    (-2, 3),
    (-1, 3),
    (0, 3),
    (1, 3),
    (2, 3),
    (3, 3),
];

pub const NEIGHBOR_AREAS: [(i32, i32); 9] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub const CITY_STATE_NONE: i32 = 0;
pub const CITY_STATE_DECLARE: i32 = 1;
pub const CITY_STATE_MASS: i32 = 2;
pub const CITY_STATE_FIGHT: i32 = 3;

/// Ceiling-деление положительных размеров региона на area-span из
/// `CreateAreaArray`; несогласованный span отсекается typed-границей до вызова.
pub fn ceil_positive_division(value: i32, divisor: i32) -> i32 {
    let quotient = value / divisor;
    quotient + i32::from(value % divisor != 0)
}

/// Wrapping abs разности двух signed координат coverage-запросов региона.
pub fn wrapping_abs_difference(left: i32, right: i32) -> i32 {
    let difference = left.wrapping_sub(right);
    let sign = difference >> 31;
    (difference ^ sign).wrapping_sub(sign)
}

/// Figure покрывает tile: горизонтальный half-extent `figure[2]`, вертикальный
/// `figure[0]` — тот же порядок, что у coverage-запросов ground shapes региона.
pub fn shape_covers_tile(shape: ShapeView, tile_x: i32, tile_y: i32) -> bool {
    wrapping_abs_difference(shape.tile_x, tile_x) <= i32::from(shape.figure.get(2))
        && wrapping_abs_difference(shape.tile_y, tile_y) <= i32::from(shape.figure.get(0))
}
