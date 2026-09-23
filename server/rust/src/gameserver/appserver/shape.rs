//! Достигнутая spatial/membership-часть `CShape` исторического GameServer.
//!
//! Constructor RVA `0x0005B9A0`, base `SetPosXY` `0x0002ABE0`,
//! `GetTileX/GetTileY/SetTileXY` `0x0005B110/0x0005B140/0x0005B170`,
//! direction/geometry `0x0004A1C0/0x000FC5C0/0x0005B2B0..0x0005B380` и
//! `SetBlock` `0x0005BA60` имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходники
//! `server/gameserver/appserver/shape.h/.cpp`.
//!
//! Exact EXE подтверждает region-link `+0x40`, region ID `+0x44`, float X/Y,
//! area/next-area links `+0x60/+0x64`, next-area X/Y `+0x68/+0x6C`, нулевые
//! direction/position/state/action и остальной next-state, speed `2000.0`.
//! Старые raw pointers region/area выражены typed link и area-index: живое
//! владение остаётся у `CServerRegion`, без self-reference и `unsafe`.
//! Float-поля хранятся бит-в-бит; tile conversion использует
//! подтверждённое exact EXE x87 truncation toward zero. Неопределённый
//! результат x87 для non-finite/out of
//! range координаты становится локальным `BLOCKED_MISSING_FACT`.
//!
//! `SetBlock` меняет только клетки с исходным block `3 -> 0` либо `0 -> 3` в
//! прямоугольнике virtual figure `DIR 2/DIR 0`. Конкретная figure и RTTI-факты
//! принадлежат derived owner-ам и передаются как `ShapeRuntimeFacts`; это не
//! перенос monster/goods/player семантики в базовый shape. Out-of-bounds
//! footprint-клетки исходный owner пропускает; несогласованный in-bounds
//! storage остаётся typed-границей `CRegion`. Остальная shape
//! поверхность ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально).
//! `IsInAround` RVA `0x0005BCE0` также `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`: region ID должен совпасть, обе area-ссылки должны
//! существовать, а абсолютная разница X и Y обязана быть меньше двух. Typed
//! region argument разрешает owned area-index обратно в координаты вместо
//! сохранения двух сырых `CArea*`.
//! `Distance(CShape*)` RVA `0x0005B390` выражен через immutable `ShapeView`:
//! сохраняются truncation-toward-zero positions, virtual figure extents,
//! wrapping subtraction и signed max без искусственного clamp к нулю.
//! `RealDistance(CShape*)` RVA `0x0005B780` использует тот же view: после
//! center-distance выбирает большую clearance-ось, вычитает figure extent
//! обеих сторон и масштабирует евклидову дистанцию по этой оси. Нулевой
//! указатель исходника выражен `Option` и возвращает `LONG_MAX`.
//! Координатные `Distance(long,long)`, `Distance(long,long,long,long)` и
//! `RealDistance(float,float)/(long,long)` RVA `0x0005B580..0x0005B730`
//! также принадлежат этому owner-у. Общий integer helper теперь используется
//! боевыми и AI-владельцами через тонкий adapter `skills::baseattack`, поэтому
//! расстояние больше не дублируется в конкретном навыке.
//! `InitMoveCheckCellList` RVA `0x0005BE60` материализован как process-owned
//! registry: точные 96 offsets распределены по трём figure и восьми direction,
//! insertion-order и повторный append сохранены, `Vec` заменяет MSVC list.
//! Persistence decode `0x0005B280/0x0005BC30` сохраняет wire-порядок и exact
//! quirk: сериализованная position читается, но live `m_lPos` становится нулём.
//! `AddToByteArray` и `AddShapeToByteArray` сохраняются отдельными слоями:
//! первый добавляет `CBaseObject`, второй пишет GUID и shape-поля в исходном
//! порядке без параллельного wire-кодека.

use super::baseobject::{BaseObjectDecodeError, CBaseObject};
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use super::region::{CRegion, RegionCellAccessBlock};
use super::serverregion::CServerRegion;
use nebokrai_shared::values::CGuid;
use thiserror::Error;

const DIRECTION_OFFSETS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];
const REAR_DIRECTIONS: [i32; 8] = [4, 5, 6, 7, 0, 1, 2, 3];
const LEFT_DIRECTIONS: [i32; 8] = [6, 7, 0, 1, 2, 3, 4, 5];
const RIGHT_DIRECTIONS: [i32; 8] = [2, 3, 4, 5, 6, 7, 0, 1];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MoveCheckCell {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Debug)]
pub(crate) struct MoveCheckCellRegistry {
    cells: [[Vec<MoveCheckCell>; 8]; 3],
}

impl MoveCheckCellRegistry {
    pub(crate) fn new() -> Self {
        Self {
            cells: std::array::from_fn(|_| std::array::from_fn(|_| Vec::new())),
        }
    }

    /// Дописывает exact batch; исходные static lists перед Init не очищались.
    pub(crate) fn initialize(&mut self) {
        for &(figure, direction, cell) in MOVE_CHECK_CELLS {
            self.cells[figure][direction].push(cell);
        }
    }

    pub(crate) fn get(&self, figure: usize, direction: usize) -> Option<&[MoveCheckCell]> {
        self.cells
            .get(figure)
            .and_then(|directions| directions.get(direction))
            .map(Vec::as_slice)
    }

    pub(crate) fn total_len(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|directions| directions.iter())
            .map(Vec::len)
            .sum()
    }
}

impl Default for MoveCheckCellRegistry {
    fn default() -> Self {
        Self::new()
    }
}

const MOVE_CHECK_CELLS: &[(usize, usize, MoveCheckCell)] = &[
    (0, 0, MoveCheckCell { x: 0, y: -1 }),
    (0, 1, MoveCheckCell { x: 1, y: -1 }),
    (0, 2, MoveCheckCell { x: 1, y: 0 }),
    (0, 3, MoveCheckCell { x: 1, y: 1 }),
    (0, 4, MoveCheckCell { x: 0, y: 1 }),
    (0, 5, MoveCheckCell { x: -1, y: 1 }),
    (0, 6, MoveCheckCell { x: -1, y: 0 }),
    (0, 7, MoveCheckCell { x: -1, y: -1 }),
    (1, 0, MoveCheckCell { x: -1, y: -2 }),
    (1, 0, MoveCheckCell { x: 0, y: -2 }),
    (1, 0, MoveCheckCell { x: 1, y: -2 }),
    (1, 1, MoveCheckCell { x: 0, y: -2 }),
    (1, 1, MoveCheckCell { x: 1, y: -2 }),
    (1, 1, MoveCheckCell { x: 2, y: -2 }),
    (1, 1, MoveCheckCell { x: 2, y: -1 }),
    (1, 1, MoveCheckCell { x: 2, y: 0 }),
    (1, 2, MoveCheckCell { x: 2, y: -1 }),
    (1, 2, MoveCheckCell { x: 2, y: 0 }),
    (1, 2, MoveCheckCell { x: 2, y: 1 }),
    (1, 3, MoveCheckCell { x: 2, y: 0 }),
    (1, 3, MoveCheckCell { x: 2, y: 1 }),
    (1, 3, MoveCheckCell { x: 2, y: 2 }),
    (1, 3, MoveCheckCell { x: 1, y: 2 }),
    (1, 3, MoveCheckCell { x: 0, y: 2 }),
    (1, 4, MoveCheckCell { x: -1, y: 2 }),
    (1, 4, MoveCheckCell { x: 0, y: 2 }),
    (1, 4, MoveCheckCell { x: 1, y: 2 }),
    (1, 5, MoveCheckCell { x: 0, y: 2 }),
    (1, 5, MoveCheckCell { x: -1, y: 2 }),
    (1, 5, MoveCheckCell { x: -2, y: 2 }),
    (1, 5, MoveCheckCell { x: -2, y: 1 }),
    (1, 5, MoveCheckCell { x: -2, y: 0 }),
    (1, 6, MoveCheckCell { x: -2, y: 1 }),
    (1, 6, MoveCheckCell { x: -2, y: 0 }),
    (1, 6, MoveCheckCell { x: -2, y: -1 }),
    (1, 7, MoveCheckCell { x: -2, y: 0 }),
    (1, 7, MoveCheckCell { x: -2, y: -1 }),
    (1, 7, MoveCheckCell { x: -2, y: -2 }),
    (1, 7, MoveCheckCell { x: -1, y: -2 }),
    (1, 7, MoveCheckCell { x: 0, y: -2 }),
    (2, 0, MoveCheckCell { x: -2, y: -3 }),
    (2, 0, MoveCheckCell { x: -1, y: -3 }),
    (2, 0, MoveCheckCell { x: 0, y: -3 }),
    (2, 0, MoveCheckCell { x: 1, y: -3 }),
    (2, 0, MoveCheckCell { x: 2, y: -3 }),
    (2, 1, MoveCheckCell { x: -1, y: -3 }),
    (2, 1, MoveCheckCell { x: 0, y: -3 }),
    (2, 1, MoveCheckCell { x: 1, y: -3 }),
    (2, 1, MoveCheckCell { x: 2, y: -3 }),
    (2, 1, MoveCheckCell { x: 3, y: -3 }),
    (2, 1, MoveCheckCell { x: 3, y: -2 }),
    (2, 1, MoveCheckCell { x: 3, y: -1 }),
    (2, 1, MoveCheckCell { x: 3, y: 0 }),
    (2, 1, MoveCheckCell { x: 3, y: 1 }),
    (2, 2, MoveCheckCell { x: 3, y: -2 }),
    (2, 2, MoveCheckCell { x: 3, y: -1 }),
    (2, 2, MoveCheckCell { x: 3, y: 0 }),
    (2, 2, MoveCheckCell { x: 3, y: 1 }),
    (2, 2, MoveCheckCell { x: 3, y: 2 }),
    (2, 3, MoveCheckCell { x: 3, y: -1 }),
    (2, 3, MoveCheckCell { x: 3, y: 0 }),
    (2, 3, MoveCheckCell { x: 3, y: 1 }),
    (2, 3, MoveCheckCell { x: 3, y: 2 }),
    (2, 3, MoveCheckCell { x: 3, y: 3 }),
    (2, 3, MoveCheckCell { x: 2, y: 3 }),
    (2, 3, MoveCheckCell { x: 1, y: 3 }),
    (2, 3, MoveCheckCell { x: 0, y: 3 }),
    (2, 3, MoveCheckCell { x: -1, y: 3 }),
    (2, 4, MoveCheckCell { x: 2, y: 3 }),
    (2, 4, MoveCheckCell { x: 1, y: 3 }),
    (2, 4, MoveCheckCell { x: 0, y: 3 }),
    (2, 4, MoveCheckCell { x: -1, y: 3 }),
    (2, 4, MoveCheckCell { x: -2, y: 3 }),
    (2, 5, MoveCheckCell { x: 1, y: 3 }),
    (2, 5, MoveCheckCell { x: 0, y: 3 }),
    (2, 5, MoveCheckCell { x: -1, y: 3 }),
    (2, 5, MoveCheckCell { x: -2, y: 3 }),
    (2, 5, MoveCheckCell { x: -3, y: 3 }),
    (2, 5, MoveCheckCell { x: -3, y: 2 }),
    (2, 5, MoveCheckCell { x: -3, y: 1 }),
    (2, 5, MoveCheckCell { x: -3, y: 0 }),
    (2, 5, MoveCheckCell { x: -3, y: -1 }),
    (2, 6, MoveCheckCell { x: -3, y: 2 }),
    (2, 6, MoveCheckCell { x: -3, y: 1 }),
    (2, 6, MoveCheckCell { x: -3, y: 0 }),
    (2, 6, MoveCheckCell { x: -3, y: -1 }),
    (2, 6, MoveCheckCell { x: -3, y: -2 }),
    (2, 7, MoveCheckCell { x: -3, y: 1 }),
    (2, 7, MoveCheckCell { x: -3, y: 0 }),
    (2, 7, MoveCheckCell { x: -3, y: -1 }),
    (2, 7, MoveCheckCell { x: -3, y: -2 }),
    (2, 7, MoveCheckCell { x: -3, y: -3 }),
    (2, 7, MoveCheckCell { x: -2, y: -3 }),
    (2, 7, MoveCheckCell { x: -1, y: -3 }),
    (2, 7, MoveCheckCell { x: 0, y: -3 }),
    (2, 7, MoveCheckCell { x: 1, y: -3 }),
];

pub(crate) const SHAPE_CHANGE_NONE: i32 = 0;
pub(crate) const SHAPE_CHANGE_DELETE: i32 = 1;
pub(crate) const SHAPE_CHANGE_REMOVE: i32 = 2;
pub(crate) const SHAPE_CHANGE_AREA: i32 = 3;
pub(crate) const SHAPE_CHANGE_REGION: i32 = 4;

pub(crate) use nebokrai_zone::regions::ShapeIdentity;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ShapeFigure {
    by_direction: [u8; 4],
}

impl ShapeFigure {
    pub(crate) const fn from_directions(by_direction: [u8; 4]) -> Self {
        Self { by_direction }
    }

    pub(crate) const fn get(self, direction: usize) -> u8 {
        self.by_direction[direction]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MonsterAreaClass {
    Active,
    Pet,
    Carriage,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsAreaFacts {
    pub(crate) particular_attribute: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ShapeRuntimeFacts {
    pub(crate) is_player: bool,
    pub(crate) monster: Option<MonsterAreaClass>,
    pub(crate) is_npc: bool,
    pub(crate) goods: Option<GoodsAreaFacts>,
    pub(crate) is_move_shape: bool,
    /// Совпадает с достигнутым условием `HP != 0 || type == NPC` при
    /// пространственной регистрации `CMoveShape`; само здоровье остаётся у
    /// concrete player/monster owner-а и не копируется в region registry.
    pub(crate) blocks_region_cell: bool,
    pub(crate) figure: ShapeFigure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShapeView {
    pub(crate) identity: ShapeIdentity,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) pos_x_bits: u32,
    pub(crate) pos_y_bits: u32,
    pub(crate) figure: ShapeFigure,
}

impl ShapeView {
    /// Exact Chebyshev-like `CShape::Distance(CShape*)` с вычитанием figure
    /// half-extents каждой стороны. Отрицательный результат допустим.
    pub(crate) fn distance(self, other: Self) -> i32 {
        distance_between_shape_geometry(
            f32::from_bits(self.pos_x_bits),
            f32::from_bits(self.pos_y_bits),
            self.figure,
            f32::from_bits(other.pos_x_bits),
            f32::from_bits(other.pos_y_bits),
            other.figure,
        )
    }

    /// Exact `CShape::RealDistance(CShape*)` для двух живых shape-view.
    pub(crate) fn real_distance(self, other: Option<Self>) -> i32 {
        let Some(other) = other else {
            return i32::MAX;
        };
        let self_x = legacy_truncate_position(f32::from_bits(self.pos_x_bits));
        let self_y = legacy_truncate_position(f32::from_bits(self.pos_y_bits));
        let other_x = legacy_truncate_position(f32::from_bits(other.pos_x_bits));
        let other_y = legacy_truncate_position(f32::from_bits(other.pos_y_bits));
        let center_distance = real_distance_from_truncated_position(
            self_x,
            self_y,
            other_x as f32,
            other_y as f32,
        );
        let horizontal_delta = ((self_x as f64 - other_x as f64).abs() as f32).trunc() as i32;
        let vertical_delta = ((self_y as f64 - other_y as f64).abs() as f32).trunc() as i32;
        let horizontal_clearance = horizontal_delta
            .wrapping_sub(other.figure.get(2) as i32)
            .wrapping_sub(self.figure.get(2) as i32);
        let vertical_clearance = vertical_delta
            .wrapping_sub(other.figure.get(0) as i32)
            .wrapping_sub(self.figure.get(0) as i32);
        let scaled = if horizontal_clearance >= vertical_clearance {
            horizontal_clearance as f64 / horizontal_delta as f64 * center_distance as f64
        } else {
            vertical_clearance as f64 / vertical_delta as f64 * center_distance as f64
        };
        round_legacy_real_distance_f64(scaled)
    }
}

fn legacy_truncate_position(value: f32) -> i32 {
    if !value.is_finite() || value < i32::MIN as f32 || value >= 2_147_483_648.0 {
        i32::MIN
    } else {
        value.trunc() as i32
    }
}

fn legacy_absolute_delta(left: i32, right: i32) -> i32 {
    let delta = left.wrapping_sub(right);
    if delta < 0 {
        delta.wrapping_neg()
    } else {
        delta
    }
}

/// Exact геометрия `CShape::Distance(CShape*)` без требования materialized
/// tile/region view. Это сохраняет исходный x87 integer-indefinite для
/// недопустимой float-координаты и используется caller-ами с живыми shapes.
pub(crate) fn distance_between_shape_geometry(
    source_x: f32,
    source_y: f32,
    source_figure: ShapeFigure,
    target_x: f32,
    target_y: f32,
    target_figure: ShapeFigure,
) -> i32 {
    let source_x = legacy_truncate_position(source_x);
    let source_y = legacy_truncate_position(source_y);
    let target_x = legacy_truncate_position(target_x);
    let target_y = legacy_truncate_position(target_y);
    let horizontal = (source_x.wrapping_sub(target_x).unsigned_abs() as i32)
        .wrapping_sub(i32::from(source_figure.get(2)))
        .wrapping_sub(i32::from(target_figure.get(2)));
    let vertical = (source_y.wrapping_sub(target_y).unsigned_abs() as i32)
        .wrapping_sub(i32::from(source_figure.get(0)))
        .wrapping_sub(i32::from(target_figure.get(0)));
    horizontal.max(vertical)
}

/// Exact `CShape::Distance(long,long,long,long)`: Chebyshev distance с
/// wrapping signed арифметикой исходного x86 owner-а.
pub(crate) fn distance_between_points(
    source_x: i32,
    source_y: i32,
    target_x: i32,
    target_y: i32,
) -> i32 {
    legacy_absolute_delta(source_x, target_x)
        .max(legacy_absolute_delta(source_y, target_y))
}

fn real_distance_from_truncated_position(
    source_x: i32,
    source_y: i32,
    target_x: f32,
    target_y: f32,
) -> f32 {
    let x = target_x - source_x as f32;
    let y = target_y - source_y as f32;
    (x * x + y * y).sqrt()
}

fn round_legacy_real_distance(distance: f32) -> i32 {
    if !distance.is_finite() || distance < i32::MIN as f32 || distance >= 2_147_483_648.0 {
        return i32::MIN;
    }
    let truncated = distance.trunc() as i32;
    if distance - truncated as f32 > 0.5 {
        truncated.saturating_add(1)
    } else {
        truncated
    }
}

fn round_legacy_real_distance_f64(distance: f64) -> i32 {
    if !distance.is_finite() || distance < i32::MIN as f64 || distance >= 2_147_483_648.0 {
        return i32::MIN;
    }
    let truncated = distance.trunc() as i32;
    if distance - truncated as f64 > 0.5 {
        truncated.saturating_add(1)
    } else {
        truncated
    }
}

/// Exact integer result `CShape::RealDistance(long,long)` для двух
/// координатных пар. Native округляет вверх только дробную часть строго
/// больше `0.5`; helper сохраняет эту границу явно.
pub(crate) fn real_distance_between_points(
    source_x: i32,
    source_y: i32,
    target_x: i32,
    target_y: i32,
) -> i32 {
    round_legacy_real_distance(real_distance_from_truncated_position(
        source_x,
        source_y,
        target_x as f32,
        target_y as f32,
    ))
}

pub(crate) trait ShapeResolver {
    /// Возвращает живой `CShape`-view для identity из region registry.
    fn resolve_shape(&self, identity: ShapeIdentity) -> Option<ShapeView>;
}

pub(crate) trait ShapePositionDispatch {
    type Error;

    /// Выполняет virtual `SetPosXY`; достигнутый `CMoveShape` override живёт у
    /// historical owner-а и не подменяется базовой записью координат.
    fn set_pos_xy(
        &mut self,
        region: &mut CRegion,
        shape: &mut CShape,
        x: f32,
        y: f32,
    ) -> Result<(), Self::Error>;
}

/// Вызывает достигнутую базовую реализацию virtual `CShape::SetPosXY` для
/// фигур без пространственного override-а `CMoveShape`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BaseShapePositionDispatch;

impl ShapePositionDispatch for BaseShapePositionDispatch {
    type Error = std::convert::Infallible;

    fn set_pos_xy(
        &mut self,
        _region: &mut CRegion,
        shape: &mut CShape,
        x: f32,
        y: f32,
    ) -> Result<(), Self::Error> {
        shape.set_pos_xy_base(x, y);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShapeCoordinateBlock {
    NonFiniteOrOutOfRange { bits: u32 },
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum ShapeDecodeError {
    #[error(transparent)]
    BaseObject(#[from] BaseObjectDecodeError),
    #[error("shape обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShapeDirectionBlock {
    pub(crate) direction: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShapeGeometryBlock {
    Coordinate(ShapeCoordinateBlock),
    Direction(ShapeDirectionBlock),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ShapeAreaCoordinates {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShapeBlockError {
    CoordinateOverflow { x: i32, y: i32, figure: ShapeFigure },
    Region(RegionCellAccessBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShapeRegionLink {
    Unassigned,
    OwningServerRegion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CShape {
    base_object: CBaseObject,
    region: ShapeRegionLink,
    region_id: i32,
    pos_x_bits: u32,
    pos_y_bits: u32,
    direction: i32,
    position: i32,
    state: u16,
    action: u16,
    area_index: Option<usize>,
    next_area_index: Option<usize>,
    next_area_x: i32,
    next_area_y: i32,
    next_region_id: i32,
    next_tile_x: i32,
    next_tile_y: i32,
    next_direction: i32,
    change_state: i32,
    speed_bits: u32,
}

impl Default for CShape {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl CShape {
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            base_object: CBaseObject::with_reached_constructor_defaults(),
            region: ShapeRegionLink::Unassigned,
            region_id: 0,
            pos_x_bits: 0.0f32.to_bits(),
            pos_y_bits: 0.0f32.to_bits(),
            direction: 0,
            position: 0,
            state: 0,
            action: 0,
            area_index: None,
            next_area_index: None,
            next_area_x: 0,
            next_area_y: 0,
            next_region_id: 0,
            next_tile_x: 0,
            next_tile_y: 0,
            next_direction: 0,
            change_state: 0,
            speed_bits: 2000.0f32.to_bits(),
        }
    }

    pub(crate) const fn identity(&self) -> ShapeIdentity {
        ShapeIdentity {
            object_type: self.base_object.get_type(),
            id: self.base_object.get_id(),
            ex_id: self.base_object.get_ex_id(),
        }
    }

    pub(crate) const fn base_object(&self) -> &CBaseObject {
        &self.base_object
    }

    pub(crate) const fn base_object_mut(&mut self) -> &mut CBaseObject {
        &mut self.base_object
    }

    pub(crate) const fn set_identity(&mut self, identity: ShapeIdentity) {
        self.base_object.set_type(identity.object_type);
        self.base_object.set_id(identity.id);
        self.base_object.set_ex_id(identity.ex_id);
    }

    pub(crate) const fn assign_to_server_region(&mut self) {
        self.region = ShapeRegionLink::OwningServerRegion;
    }

    pub(crate) const fn is_assigned_to_server_region(&self) -> bool {
        matches!(self.region, ShapeRegionLink::OwningServerRegion)
    }

    pub(crate) const fn get_region_id(&self) -> i32 {
        self.region_id
    }

    pub(crate) const fn set_region_id(&mut self, region_id: i32) {
        self.region_id = region_id;
    }

    pub(crate) fn get_pos_x(&self) -> f32 {
        f32::from_bits(self.pos_x_bits)
    }

    pub(crate) fn get_pos_y(&self) -> f32 {
        f32::from_bits(self.pos_y_bits)
    }

    /// Exact `CShape::Distance(long,long)`: virtual X/Y сначала усекаются к
    /// нулю через x87 `fistp`, затем выбирается максимальная абсолютная дельта.
    pub(crate) fn distance_to_point(&self, x: i32, y: i32) -> i32 {
        distance_between_points(
            legacy_truncate_position(self.get_pos_x()),
            legacy_truncate_position(self.get_pos_y()),
            x,
            y,
        )
    }

    /// Exact primitive `CShape::RealDistance(float,float)`.
    pub(crate) fn real_distance_to_position(&self, x: f32, y: f32) -> f32 {
        real_distance_from_truncated_position(
            legacy_truncate_position(self.get_pos_x()),
            legacy_truncate_position(self.get_pos_y()),
            x,
            y,
        )
    }

    /// Exact rounded overload `CShape::RealDistance(long,long)`.
    pub(crate) fn real_distance_to_point(&self, x: i32, y: i32) -> i32 {
        round_legacy_real_distance(self.real_distance_to_position(x as f32, y as f32))
    }

    pub(crate) const fn get_direction(&self) -> i32 {
        self.direction
    }

    pub(crate) fn set_direction(&mut self, direction: i32) {
        if (0..8).contains(&direction) {
            self.direction = direction;
        }
    }

    pub(crate) const fn get_position(&self) -> i32 {
        self.position
    }

    pub(crate) const fn set_position(&mut self, position: i32) {
        self.position = position;
    }

    pub(crate) fn get_speed(&self) -> f32 {
        f32::from_bits(self.speed_bits)
    }

    pub(crate) fn set_speed(&mut self, speed: f32) {
        self.speed_bits = speed.to_bits();
    }

    pub(crate) const fn get_state(&self) -> u16 {
        self.state
    }

    pub(crate) const fn set_state(&mut self, state: u16) {
        self.state = state;
    }

    pub(crate) const fn get_action(&self) -> u16 {
        self.action
    }

    pub(crate) const fn set_action(&mut self, action: u16) {
        self.action = action;
    }

    /// Exact `AddShapeToByteArray`: GUID marker и scalar order зеркальны
    /// `DecordShapeFromByteArray` и не включают базовый `CBaseObject`.
    pub(crate) fn add_shape_to_byte_array(&self, destination: &mut Vec<u8>) -> bool {
        let ex_id = self.base_object.get_ex_id();
        let mut writer = LegacyWriter::new(destination);
        if ex_id.is_invalid() {
            writer.write_u8(0);
        } else {
            writer.write_u8(0x10);
            writer.write_bytes(ex_id.as_legacy_bytes());
        }
        writer.write_i32(self.region_id);
        writer.write_u32(self.pos_x_bits);
        writer.write_u32(self.pos_y_bits);
        writer.write_i32(self.direction);
        writer.write_i32(self.position);
        writer.write_u32(self.speed_bits);
        writer.write_u16(self.state);
        writer.write_u16(self.action);
        true
    }

    /// Exact virtual `AddToByteArray`: сначала базовый объект, затем shape-
    /// часть через отдельный виртуальный слой исходного owner-а.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> bool {
        if !self
            .base_object
            .add_to_byte_array(destination, include_child)
        {
            return false;
        }
        self.add_shape_to_byte_array(destination)
    }

    /// Exact `DecordFromByteArray + DecordShapeFromByteArray`; wire position
    /// читается, но live `m_lPos` намеренно сбрасывается в ноль.
    pub(crate) fn decode_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<(), ShapeDecodeError> {
        self.base_object
            .decord_from_byte_array(source, cursor, include_child)?;
        let marker = read_shape_wire::<1>(source, cursor, "m_guExID marker")?[0];
        let ex_id = if marker == 0 {
            CGuid::GUID_INVALID
        } else {
            CGuid::from_legacy_bytes(read_shape_wire::<16>(source, cursor, "m_guExID")?)
        };
        self.base_object.set_ex_id(ex_id);
        self.region_id = read_shape_i32(source, cursor, "m_lRegionID")?;
        self.pos_x_bits = read_shape_u32(source, cursor, "m_fPosX")?;
        self.pos_y_bits = read_shape_u32(source, cursor, "m_fPosY")?;
        self.direction = read_shape_i32(source, cursor, "m_lDir")?;
        let _serialized_position = read_shape_i32(source, cursor, "m_lPos")?;
        self.position = 0;
        self.speed_bits = read_shape_u32(source, cursor, "m_fSpeed")?;
        self.state = read_shape_u16(source, cursor, "m_wState")?;
        self.action = read_shape_u16(source, cursor, "m_wAction")?;
        Ok(())
    }

    pub(crate) const fn area_index(&self) -> Option<usize> {
        self.area_index
    }

    pub(crate) const fn set_area_index(&mut self, area_index: Option<usize>) {
        self.area_index = area_index;
    }

    pub(crate) fn is_in_around(&self, other: &CShape, region: &CServerRegion) -> bool {
        if self.get_region_id() != other.get_region_id() {
            return false;
        }
        let (Some(self_index), Some(other_index)) = (self.area_index(), other.area_index()) else {
            return false;
        };
        let (Ok(self_area), Ok(other_area)) = (
            region.get_area_by_index(self_index as i32),
            region.get_area_by_index(other_index as i32),
        ) else {
            return false;
        };
        self_area.x().abs_diff(other_area.x()) < 2 && self_area.y().abs_diff(other_area.y()) < 2
    }

    pub(crate) const fn next_area_index(&self) -> Option<usize> {
        self.next_area_index
    }

    pub(crate) const fn set_next_area_index(&mut self, area_index: Option<usize>) {
        self.next_area_index = area_index;
    }

    pub(crate) const fn next_area_coordinates(&self) -> ShapeAreaCoordinates {
        ShapeAreaCoordinates {
            x: self.next_area_x,
            y: self.next_area_y,
        }
    }

    pub(crate) const fn set_next_area_coordinates(&mut self, coordinates: ShapeAreaCoordinates) {
        self.next_area_x = coordinates.x;
        self.next_area_y = coordinates.y;
    }

    pub(crate) const fn next_region_id(&self) -> i32 {
        self.next_region_id
    }

    pub(crate) const fn next_direction(&self) -> i32 {
        self.next_direction
    }

    pub(crate) const fn change_state(&self) -> i32 {
        self.change_state
    }

    pub(crate) const fn set_change_state(&mut self, change_state: i32) {
        self.change_state = change_state;
    }

    /// Exact `CPlayer::ChangeRegion` local-server tail. Region membership
    /// remains owned by `CServerRegion::AI`; the player only publishes its
    /// deferred destination and `CS_CHANGEREGION` marker here.
    pub(crate) const fn stage_region_change(
        &mut self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
    ) {
        self.next_region_id = region_id;
        self.next_tile_x = tile_x;
        self.next_tile_y = tile_y;
        self.next_direction = direction;
        self.change_state = SHAPE_CHANGE_REGION;
    }

    /// Deferred `CServerRegion::AI` tail after removal from the old registry.
    /// The destination registry is populated only by client enter ack `8F801`.
    pub(crate) fn apply_staged_region_change(&mut self) -> (i32, i32, i32, i32) {
        let destination = (
            self.next_region_id,
            self.next_tile_x,
            self.next_tile_y,
            self.next_direction,
        );
        self.region_id = self.next_region_id;
        self.set_pos_xy_base(self.next_tile_x as f32 + 0.5, self.next_tile_y as f32 + 0.5);
        self.set_direction(self.next_direction);
        self.action = 0;
        self.change_state = SHAPE_CHANGE_NONE;
        destination
    }

    pub(crate) fn get_tile_x(&self) -> Result<i32, ShapeCoordinateBlock> {
        tile_from_bits(self.pos_x_bits)
    }

    pub(crate) fn get_tile_y(&self) -> Result<i32, ShapeCoordinateBlock> {
        tile_from_bits(self.pos_y_bits)
    }

    pub(crate) fn tile_from_value(value: f32) -> Result<i32, ShapeCoordinateBlock> {
        tile_from_bits(value.to_bits())
    }

    pub(crate) fn set_tile_xy<Dispatch: ShapePositionDispatch>(
        &mut self,
        region: &mut CRegion,
        x: i32,
        y: i32,
        dispatch: &mut Dispatch,
    ) -> Result<(), Dispatch::Error> {
        dispatch.set_pos_xy(region, self, (x as f32) + 0.5, (y as f32) + 0.5)
    }

    pub(crate) fn set_pos_xy_base(&mut self, x: f32, y: f32) {
        self.pos_x_bits = x.to_bits();
        self.pos_y_bits = y.to_bits();
    }

    pub(crate) fn set_pos_xy_move_order(&mut self, x: f32, y: f32) {
        self.pos_y_bits = y.to_bits();
        self.pos_x_bits = x.to_bits();
    }

    pub(crate) fn set_pos_x<Dispatch: ShapePositionDispatch>(
        &mut self,
        region: &mut CRegion,
        x: f32,
        dispatch: &mut Dispatch,
    ) -> Result<(), Dispatch::Error> {
        dispatch.set_pos_xy(region, self, x, self.get_pos_y())
    }

    pub(crate) fn set_pos_y<Dispatch: ShapePositionDispatch>(
        &mut self,
        region: &mut CRegion,
        y: f32,
        dispatch: &mut Dispatch,
    ) -> Result<(), Dispatch::Error> {
        dispatch.set_pos_xy(region, self, self.get_pos_x(), y)
    }

    pub(crate) fn get_face_position(&self) -> Result<ShapeAreaCoordinates, ShapeGeometryBlock> {
        let offset = direction_offset(self.direction).map_err(ShapeGeometryBlock::Direction)?;
        let x = self
            .get_tile_x()
            .map_err(ShapeGeometryBlock::Coordinate)?
            .wrapping_add(offset.x);
        let y = self
            .get_tile_y()
            .map_err(ShapeGeometryBlock::Coordinate)?
            .wrapping_add(offset.y);
        Ok(ShapeAreaCoordinates { x, y })
    }

    pub(crate) fn get_direction_position(
        direction: i32,
        position: ShapeAreaCoordinates,
    ) -> Result<ShapeAreaCoordinates, ShapeDirectionBlock> {
        let offset = direction_offset(direction)?;
        Ok(ShapeAreaCoordinates {
            x: position.x.wrapping_add(offset.x),
            y: position.y.wrapping_add(offset.y),
        })
    }

    pub(crate) fn get_rear_direction(&self) -> Result<i32, ShapeDirectionBlock> {
        transformed_direction(self.direction, &REAR_DIRECTIONS)
    }

    pub(crate) fn get_left_direction(&self) -> Result<i32, ShapeDirectionBlock> {
        transformed_direction(self.direction, &LEFT_DIRECTIONS)
    }

    pub(crate) fn get_right_direction(&self) -> Result<i32, ShapeDirectionBlock> {
        transformed_direction(self.direction, &RIGHT_DIRECTIONS)
    }

    pub(crate) fn set_block(
        &self,
        region: &mut CRegion,
        x: i32,
        y: i32,
        block: u8,
        figure: ShapeFigure,
    ) -> Result<(), ShapeBlockError> {
        if block != 0 && block != 3 {
            return Ok(());
        }

        let horizontal = i32::from(figure.get(2));
        let vertical = i32::from(figure.get(0));
        let Some(left) = x.checked_sub(horizontal) else {
            return Err(ShapeBlockError::CoordinateOverflow { x, y, figure });
        };
        let Some(right) = x.checked_add(horizontal) else {
            return Err(ShapeBlockError::CoordinateOverflow { x, y, figure });
        };
        let Some(top) = y.checked_sub(vertical) else {
            return Err(ShapeBlockError::CoordinateOverflow { x, y, figure });
        };
        let Some(bottom) = y.checked_add(vertical) else {
            return Err(ShapeBlockError::CoordinateOverflow { x, y, figure });
        };

        for cell_x in left..=right {
            for cell_y in top..=bottom {
                if cell_x < 0 || cell_x >= region.width || cell_y < 0 || cell_y >= region.height {
                    continue;
                }
                let current = region
                    .get_block(cell_x, cell_y)
                    .map_err(ShapeBlockError::Region)?;
                if (block == 0 && current == 3) || (block == 3 && current == 0) {
                    region
                        .set_block(cell_x, cell_y, block)
                        .map_err(ShapeBlockError::Region)?;
                }
            }
        }
        Ok(())
    }
}

fn tile_from_bits(bits: u32) -> Result<i32, ShapeCoordinateBlock> {
    let truncated = f32::from_bits(bits).trunc();
    if !truncated.is_finite() || truncated < i32::MIN as f32 || truncated >= 2_147_483_648.0 {
        // BLOCKED_MISSING_FACT: x87 invalid-conversion result для этого пути
        // не задаёт переносимый safe Rust contract.
        return Err(ShapeCoordinateBlock::NonFiniteOrOutOfRange { bits });
    }
    Ok(truncated as i32)
}

fn direction_offset(direction: i32) -> Result<ShapeAreaCoordinates, ShapeDirectionBlock> {
    let Ok(index) = usize::try_from(direction) else {
        return Err(ShapeDirectionBlock { direction });
    };
    DIRECTION_OFFSETS
        .get(index)
        .copied()
        .map(|(x, y)| ShapeAreaCoordinates { x, y })
        .ok_or(ShapeDirectionBlock { direction })
}

fn transformed_direction(direction: i32, table: &[i32; 8]) -> Result<i32, ShapeDirectionBlock> {
    let Ok(index) = usize::try_from(direction) else {
        return Err(ShapeDirectionBlock { direction });
    };
    table
        .get(index)
        .copied()
        .ok_or(ShapeDirectionBlock { direction })
}

fn read_shape_wire<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], ShapeDecodeError> {
    let mut reader = shape_reader(source, *cursor, field, N)?;
    let bytes = reader.read_bytes(N).map_err(|block| shape_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes.try_into().expect("прочитано точное число байт"))
}

fn read_shape_i32(source: &[u8], cursor: &mut usize, field: &'static str) -> Result<i32, ShapeDecodeError> {
    let mut reader = shape_reader(source, *cursor, field, 4)?;
    let value = reader.read_i32().map_err(|block| shape_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_shape_u32(source: &[u8], cursor: &mut usize, field: &'static str) -> Result<u32, ShapeDecodeError> {
    let mut reader = shape_reader(source, *cursor, field, 4)?;
    let value = reader.read_u32().map_err(|block| shape_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_shape_u16(source: &[u8], cursor: &mut usize, field: &'static str) -> Result<u16, ShapeDecodeError> {
    let mut reader = shape_reader(source, *cursor, field, 2)?;
    let value = reader.read_u16().map_err(|block| shape_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn shape_reader<'source>(source: &'source [u8], cursor: usize, field: &'static str, needed: usize) -> Result<LegacyReader<'source>, ShapeDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| ShapeDecodeError::UnexpectedEnd { field, offset: block.offset, needed, available: block.available })
}

fn shape_error(field: &'static str, block: nebokrai_shared::protocol::LegacyReadBlock) -> ShapeDecodeError {
    ShapeDecodeError::UnexpectedEnd { field, offset: block.offset, needed: block.needed, available: block.available }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp

// IMPLEMENTED: `CShape::SetPosXY` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetChangeState` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetRegionID` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetRegionID` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetPosX` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetPosX` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetPosY` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetPosY` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetDir` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetPos` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetPos` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetSpeed` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetSpeed` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetState` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetState` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetAction` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CShape::~CShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:180
// RVA: 0x0005B100
// ADDRESS: 0045b100
// PROTOTYPE: void __thiscall ~CShape(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CShape::GetTileX` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetTileY` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetTileXY` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CShape::AddShapeToByteArray
// STATUS: IMPLEMENTED ABOVE; DECOMPILER STORED LOCALLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:236
// RVA: 0x0005B1A0
// ADDRESS: 0045b1a0
// PROTOTYPE: bool __thiscall AddShapeToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::AddToByteArray
// STATUS: IMPLEMENTED ABOVE; DECOMPILER STORED LOCALLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:271
// RVA: 0x0005B250
// ADDRESS: 0045b250
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CShape::GetFacePos` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetDirPos` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetRearDir` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetLeftDir` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::GetRightDir` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CShape::Distance
// STATUS: IMPLEMENTED
// IMPLEMENTED: `CShape::distance_to_point`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:415
// RVA: 0x0005B580
// ADDRESS: 0045b580
// PROTOTYPE: long __thiscall Distance(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::Distance
// STATUS: IMPLEMENTED
// IMPLEMENTED: `distance_between_points`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:420
// RVA: 0x0005B670
// ADDRESS: 0045b670
// PROTOTYPE: long __thiscall Distance(long param_1, long param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::RealDistance
// STATUS: IMPLEMENTED
// IMPLEMENTED: `CShape::real_distance_to_position`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:425
// RVA: 0x0005B6A0
// ADDRESS: 0045b6a0
// PROTOTYPE: float __thiscall RealDistance(float param_1, float param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::RealDistance
// STATUS: IMPLEMENTED
// IMPLEMENTED: `CShape::real_distance_to_point` и общий
// `real_distance_between_points` для фактических AI/combat callers.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:432
// RVA: 0x0005B730
// ADDRESS: 0045b730
// PROTOTYPE: long __thiscall RealDistance(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::RealDistance
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: `ShapeView::real_distance`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:450
// RVA: 0x0005B780
// ADDRESS: 0045b780
// PROTOTYPE: long __thiscall RealDistance(CShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CShape::CShape` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::SetBlock` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CShape::IsInAround` материализован выше; покрытый raw-блок
// удалён.
