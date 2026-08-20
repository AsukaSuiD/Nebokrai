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

use super::baseobject::CBaseObject;
use super::region::{CRegion, RegionCellAccessBlock};
use super::serverregion::CServerRegion;
use crate::public::guid::CGuid;

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

pub(crate) const SHAPE_CHANGE_NONE: i32 = 0;
pub(crate) const SHAPE_CHANGE_AREA: i32 = 3;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ShapeIdentity {
    pub(crate) object_type: i32,
    pub(crate) id: i32,
    pub(crate) ex_id: CGuid,
}

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
    pub(crate) figure: ShapeFigure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShapeView {
    pub(crate) identity: ShapeIdentity,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) figure: ShapeFigure,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShapeCoordinateBlock {
    NonFiniteOrOutOfRange { bits: u32 },
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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

// ============================================================================
// FUNCTION: CShape::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:279
// RVA: 0x0005B280
// ADDRESS: 0045b280
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:408
// RVA: 0x0005B390
// ADDRESS: 0045b390
// PROTOTYPE: long __thiscall Distance(CShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::Distance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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

// ============================================================================
// FUNCTION: CShape::DecordShapeFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:251
// RVA: 0x0005BC30
// ADDRESS: 0045bc30
// PROTOTYPE: bool __thiscall DecordShapeFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CShape::IsInAround` материализован выше; покрытый raw-блок
// удалён.

// ============================================================================
// FUNCTION: CShape::InitMoveCheckCellList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\shape.cpp:37
// RVA: 0x0005BE60
// ADDRESS: 0045be60
// PROTOTYPE: void __cdecl InitMoveCheckCellList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CShape::GetDir` материализован выше; покрытый raw-блок удалён.


// IMPLEMENTED: `CShape::SetAction` материализован выше; покрытый raw-блок удалён.

// COMPONENT_VARIANT_END: GameServer
