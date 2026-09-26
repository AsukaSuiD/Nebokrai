//! Форма `CShape` из `shape.cpp/.h`.
//!
//! Владелец расширяет `CBaseObject` region, position, direction, state/action
//! и speed `2000.0`. Wire пишет base object, optional GUID, region, bit-exact
//! floats X/Y/speed, dir/pos и два `u16`; decoder читает тот же порядок, но
//! намеренно сбрасывает live `m_lPos` в ноль.
//!
//! Tile coordinates используют x87 truncation к нулю. NaN, infinity и значение
//! вне `i32` блокируются вместо неопределённого преобразования. `SetPosX/Y`
//! сохраняют вторую координату через getter и общий `SetPosXY`.
//!
//! Rust-композиция `CBaseObject -> CShape -> CMoveShape` заменяет ABI/vtable;
//! короткий wire сохраняет уже выполненные присваивания до ошибки.

use crate::regions::baseobject::*;

pub use crate::regions::shapetypes::{ShapeDecodeError, ShapeTileCoordinateBlock};

pub struct CShape {
    base_object: CBaseObject,
    region_id: i32,
    pos_x: f32,
    pos_y: f32,
    direction: i32,
    position: i32,
    speed: f32,
    state: u16,
    action: u16,
}

impl CShape {
    pub const fn get_figure(&self) -> u8 {
        0
    }

 /// Создаёт только исходное начальное region-состояние исходного
 /// конструктора.
    pub const fn with_constructor_region_default() -> Self {
        Self {
            base_object: CBaseObject::with_reached_constructor_defaults(),
            region_id: 0,
            pos_x: 0.0,
            pos_y: 0.0,
            direction: 0,
            position: 0,
            speed: 2000.0,
            state: 0,
            action: 0,
        }
    }

    pub const fn get_type(&self) -> i32 {
        self.base_object.get_type()
    }

    pub const fn set_type(&mut self, object_type: i32) {
        self.base_object.set_type(object_type);
    }

    pub const fn get_id(&self) -> i32 {
        self.base_object.get_id()
    }

    pub const fn set_id(&mut self, id: i32) {
        self.base_object.set_id(id);
    }

    pub const fn get_ex_id(&self) -> &nebokrai_shared::values::CGuid {
        self.base_object.get_ex_id()
    }

    pub const fn set_ex_id(&mut self, ex_id: &nebokrai_shared::values::CGuid) {
        self.base_object.set_ex_id(ex_id);
    }

    pub fn get_name(&self) -> &[u8] {
        self.base_object.get_name()
    }

    pub fn set_name(&mut self, name: &[u8]) {
        self.base_object.set_name(name);
    }

    pub const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.base_object.set_graphics_id(graphics_id);
    }

    pub const fn get_region_id(&self) -> i32 {
        self.region_id
    }

    pub const fn get_pos_x(&self) -> f32 {
        self.pos_x
    }

    pub const fn set_pos_x(&mut self, pos_x: f32) {
        self.set_pos_xy(pos_x, self.get_pos_y());
    }

    pub const fn get_pos_y(&self) -> f32 {
        self.pos_y
    }

    pub const fn set_pos_y(&mut self, pos_y: f32) {
        self.set_pos_xy(self.get_pos_x(), pos_y);
    }

    pub fn get_tile_x(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        truncate_tile_coordinate(self.get_pos_x(), "X")
    }

    pub fn get_tile_y(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        truncate_tile_coordinate(self.get_pos_y(), "Y")
    }

    pub const fn get_direction(&self) -> i32 {
        self.direction
    }

    pub const fn set_position(&mut self, position: i32) {
        self.position = position;
    }

    pub const fn get_speed(&self) -> f32 {
        self.speed
    }

    pub const fn set_region_id(&mut self, region_id: i32) {
        self.region_id = region_id;
    }

    pub const fn set_pos_xy(&mut self, pos_x: f32, pos_y: f32) {
        self.pos_x = pos_x;
        self.pos_y = pos_y;
    }

    pub const fn set_direction(&mut self, direction: i32) -> bool {
        if direction < 0 || direction >= 8 {
            return false;
        }
        self.direction = direction;
        true
    }

    pub const fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn set_tile_xy(&mut self, tile_x: i32, tile_y: i32) {
        self.pos_x = tile_x as f32 + 0.5;
        self.pos_y = tile_y as f32 + 0.5;
    }

    pub const fn set_state(&mut self, state: u16) {
        self.state = state;
    }

    pub const fn get_state(&self) -> u16 {
        self.state
    }

    pub const fn get_action(&self) -> u16 {
        self.action
    }

    pub const fn set_action(&mut self, action: u16) {
        self.action = action;
    }

    pub fn add_to_byte_array(&self, destination: &mut Vec<u8>, include_child: bool) -> bool {
        let _ = self
            .base_object
            .add_to_byte_array(destination, include_child);
        let _ = self.add_shape_to_byte_array(destination);
        true
    }

    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<bool, ShapeDecodeError> {
        let _ = self
            .base_object
            .decord_from_byte_array(source, cursor, include_child)?;
        let _ = self.decord_shape_from_byte_array(source, cursor)?;
        Ok(true)
    }

    fn add_shape_to_byte_array(&self, destination: &mut Vec<u8>) -> bool {
        let ex_id = self.base_object.get_ex_id();
        if ex_id.is_invalid() {
            destination.push(0);
        } else {
            destination.push(16);
            destination.extend_from_slice(ex_id.as_legacy_bytes());
        }
        destination.extend_from_slice(&self.region_id.to_le_bytes());
        destination.extend_from_slice(&self.pos_x.to_le_bytes());
        destination.extend_from_slice(&self.pos_y.to_le_bytes());
        destination.extend_from_slice(&self.direction.to_le_bytes());
        destination.extend_from_slice(&self.position.to_le_bytes());
        destination.extend_from_slice(&self.speed.to_le_bytes());
        destination.extend_from_slice(&self.state.to_le_bytes());
        destination.extend_from_slice(&self.action.to_le_bytes());
        true
    }

    fn decord_shape_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, ShapeDecodeError> {
        let marker = read_shape_array::<1>(source, cursor, "m_guExID marker")?[0];
        if marker == 0 {
            self.base_object
                .set_ex_id(&nebokrai_shared::values::CGuid::GUID_INVALID);
        } else {
            let bytes = read_shape_array::<16>(source, cursor, "m_guExID")?;
            self.base_object
                .set_ex_id(&nebokrai_shared::values::CGuid::from_legacy_bytes(bytes));
        }

        self.region_id = read_shape_i32(source, cursor, "m_lRegionID")?;
        self.pos_x = read_shape_f32(source, cursor, "m_fPosX")?;
        self.pos_y = read_shape_f32(source, cursor, "m_fPosY")?;
        self.direction = read_shape_i32(source, cursor, "m_lDir")?;
        let _serialized_position = read_shape_i32(source, cursor, "m_lPos")?;
        // WorldServer сдвигает cursor через wire m_lPos, но
        // присваивает live m_lPos константу 0 вместо прочитанного DWORD.
        self.position = 0;
        self.speed = read_shape_f32(source, cursor, "m_fSpeed")?;
        self.state = read_shape_u16(source, cursor, "m_wState")?;
        self.action = read_shape_u16(source, cursor, "m_wAction")?;

        Ok(true)
    }
}

fn truncate_tile_coordinate(
    value: f32,
    axis: &'static str,
) -> Result<i32, ShapeTileCoordinateBlock> {
    // x87 `fistp dword` выдаёт integer-indefinite для
    // NaN/inf/out-of-range. Достижимость такого live position и обязанность
    // публиковать именно этот результат соседним owner-ам не определены.
    if !value.is_finite() || !(-2_147_483_648.0..2_147_483_648.0).contains(&value) {
        return Err(ShapeTileCoordinateBlock {
            axis,
            value_bits: value.to_bits(),
        });
    }
    Ok(value.trunc() as i32)
}

fn read_shape_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, ShapeDecodeError> {
    Ok(i32::from_le_bytes(read_shape_array(source, cursor, field)?))
}

fn read_shape_f32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<f32, ShapeDecodeError> {
    Ok(f32::from_le_bytes(read_shape_array(source, cursor, field)?))
}

fn read_shape_u16(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u16, ShapeDecodeError> {
    Ok(u16::from_le_bytes(read_shape_array(source, cursor, field)?))
}

fn read_shape_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], ShapeDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(N) else {
        return Err(ShapeDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        // Старый helper не получал длину источника и
        // продолжал чтение. Safe Rust останавливает только эту границу.
        return Err(ShapeDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    *cursor = end;
    Ok(bytes
        .try_into()
        .expect("slice содержит ровно запрошенное число байт"))
}
