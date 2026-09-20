//! Подвижная форма `CMoveShape` из `moveshape.cpp/.h`, подтверждённая
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Владелец содержит один `CShape`, ex-state bytes и `is_god = false`.
//! `SetExStates` дописывает непустой slice без очистки; player decoder очищает
//! vector отдельно перед вызовом. Собственного wire-формата слой не добавляет.
//!
//! Делегированные type/ID/position methods работают с тем же base object.
//! `Vec<u8>` и Drop заменяют MSVC vector и destructor, не копируя ABI/vtable.

use super::shape::{CShape, ShapeDecodeError, ShapeTileCoordinateBlock};

pub(crate) struct CMoveShape {
    shape_base: CShape,
    ex_states: Vec<u8>,
    is_god: bool,
}

impl CMoveShape {
    pub(crate) const fn get_figure(&self) -> u8 {
        self.shape_base.get_figure()
    }

    pub(crate) const fn with_constructor_shape_base() -> Self {
        Self {
            shape_base: CShape::with_constructor_region_default(),
            ex_states: Vec::new(),
            is_god: false,
        }
    }

    pub(crate) const fn get_type(&self) -> i32 {
        self.shape_base.get_type()
    }

    pub(crate) const fn set_type(&mut self, object_type: i32) {
        self.shape_base.set_type(object_type);
    }

    pub(crate) const fn get_id(&self) -> i32 {
        self.shape_base.get_id()
    }

    pub(crate) const fn set_id(&mut self, id: i32) {
        self.shape_base.set_id(id);
    }

    pub(crate) fn get_name(&self) -> &[u8] {
        self.shape_base.get_name()
    }

    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.shape_base.set_name(name);
    }

    pub(crate) const fn get_region_id(&self) -> i32 {
        self.shape_base.get_region_id()
    }

    pub(crate) const fn get_pos_x(&self) -> f32 {
        self.shape_base.get_pos_x()
    }

    pub(crate) const fn set_pos_x(&mut self, pos_x: f32) {
        self.shape_base.set_pos_x(pos_x);
    }

    pub(crate) const fn get_pos_y(&self) -> f32 {
        self.shape_base.get_pos_y()
    }

    pub(crate) const fn set_pos_y(&mut self, pos_y: f32) {
        self.shape_base.set_pos_y(pos_y);
    }

    pub(crate) fn get_tile_x(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        self.shape_base.get_tile_x()
    }

    pub(crate) fn get_tile_y(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        self.shape_base.get_tile_y()
    }

    pub(crate) const fn get_direction(&self) -> i32 {
        self.shape_base.get_direction()
    }

    pub(crate) const fn set_position(&mut self, position: i32) {
        self.shape_base.set_position(position);
    }

    pub(crate) const fn get_speed(&self) -> f32 {
        self.shape_base.get_speed()
    }

    pub(crate) const fn set_region_id(&mut self, region_id: i32) {
        self.shape_base.set_region_id(region_id);
    }

    pub(crate) const fn set_pos_xy(&mut self, pos_x: f32, pos_y: f32) {
        self.shape_base.set_pos_xy(pos_x, pos_y);
    }

    pub(crate) const fn set_direction(&mut self, direction: i32) -> bool {
        self.shape_base.set_direction(direction)
    }

    pub(crate) const fn set_speed(&mut self, speed: f32) {
        self.shape_base.set_speed(speed);
    }

    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.shape_base.set_graphics_id(graphics_id);
    }

    pub(crate) fn set_tile_xy(&mut self, tile_x: i32, tile_y: i32) {
        self.shape_base.set_tile_xy(tile_x, tile_y);
    }

    pub(crate) const fn set_state(&mut self, state: u16) {
        self.shape_base.set_state(state);
    }

    pub(crate) const fn get_state(&self) -> u16 {
        self.shape_base.get_state()
    }

    pub(crate) const fn get_action(&self) -> u16 {
        self.shape_base.get_action()
    }

    pub(crate) const fn set_action(&mut self, action: u16) {
        self.shape_base.set_action(action);
    }

    pub(crate) fn ex_states(&self) -> &[u8] {
        &self.ex_states
    }

    pub(crate) fn clear_ex_states(&mut self) {
        self.ex_states.clear();
    }

    pub(crate) fn set_ex_states(&mut self, states: &[u8]) {
        if !states.is_empty() {
            self.ex_states.extend_from_slice(states);
        }
    }

    pub(crate) const fn is_god(&self) -> bool {
        self.is_god
    }

    pub(crate) const fn set_is_god(&mut self, is_god: bool) {
        self.is_god = is_god;
    }

    pub(crate) fn add_shape_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> bool {
        self.shape_base
            .add_to_byte_array(destination, include_child)
    }

    pub(crate) fn decord_shape_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<bool, ShapeDecodeError> {
        self.shape_base
            .decord_from_byte_array(source, cursor, include_child)
    }
}
