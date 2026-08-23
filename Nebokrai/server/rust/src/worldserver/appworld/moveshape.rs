//! Владелец подвижного shape-слоя исторического `WorldServer`.
//!
//! Base-подобъект `CShape`, включая inherited `GetName`,
//! `CMoveShape::SetExStates`, destructor
//! и `CMoveShape::CMoveShape`
//! представлены действующим Rust-owner-ом. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Исходный конструктор первым вызывает `CShape::CShape`, затем меняет vtable,
//! создаёт пустой `m_vExStates` и ставит `m_bIsGod = false`. Безопасная
//! композиция хранит действующий `CShape` отдельным полем, `Vec<u8>` заменяет
//! только STL storage ex-state bytes, а обычный `Drop` — его destructor.
//! `SetExStates` при non-null и ненулевой длине дописывает bytes в хвост без
//! предварительной очистки; Rust-slice исключает null, пустой slice сохраняет
//! no-op. Отдельная clear-операция нужна точному `CPlayer::DecordFromByteArray`,
//! который очищает vector раньше вызова `SetExStates`.
//! Для связанный точного `CPlayer` clone этот owner уже пропускает готовые
//! `CShape::AddToByteArray/DecordFromByteArray` через единственный base-
//! подобъект; это композиция, а не отдельный wire-формат `CMoveShape`.
//!
//! Делегированные type/ID/region/position-методы не создают нового владельца поведения: они
//! только выражают доступ производного типа к готовым `CBaseObject` и
//! `CShape`. PDB задаёт размеры старых `CShape/CMoveShape` `0x6C/0x80`,
//! а оригинал-конструктор — base-вызов по offset `0`; Rust layout и исходный
//! ABI/vtable не отождествляются. оригинал constructor/destructor/SetExStates
//! удалены после замены; дополнительное оригинал не требовалось.
//! Полный scalar accessor-набор `CShape` делегируется тем же единственным
//! base-owner-ом, чтобы CPlayer не терял доступ к действующим virtual slots.

use super::shape::{CShape, ShapeDecodeError, ShapeTileCoordinateBlock};

/// Материализованная собственная и base-часть исходного `CMoveShape`.
pub(crate) struct CMoveShape {
    shape_base: CShape,
    ex_states: Vec<u8>,
    is_god: bool,
}

impl CMoveShape {
 /// Делегирует базовый виртуальный `CShape::GetFigure` без дополнительных эффектов.
    pub(crate) const fn get_figure(&self) -> u8 {
        self.shape_base.get_figure()
    }

 /// Создаёт действующий `CShape` и оба точных default собственного owner-а.
    pub(crate) const fn with_constructor_shape_base() -> Self {
        Self {
            shape_base: CShape::with_constructor_region_default(),
            ex_states: Vec::new(),
            is_god: false,
        }
    }

 /// Возвращает object type через унаследованный base-object owner.
    pub(crate) const fn get_type(&self) -> i32 {
        self.shape_base.get_type()
    }

 /// Присваивает object type через унаследованный base-object owner.
    pub(crate) const fn set_type(&mut self, object_type: i32) {
        self.shape_base.set_type(object_type);
    }

 /// Возвращает ID через унаследованный base-object owner.
    pub(crate) const fn get_id(&self) -> i32 {
        self.shape_base.get_id()
    }

 /// Присваивает ID через унаследованный base-object owner.
    pub(crate) const fn set_id(&mut self, id: i32) {
        self.shape_base.set_id(id);
    }

 /// Заимствует имя через унаследованный shape/base-object owner.
    pub(crate) fn get_name(&self) -> &[u8] {
        self.shape_base.get_name()
    }

 /// Присваивает имя через тот же inherited `CBaseObject::SetName` owner.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.shape_base.set_name(name);
    }

 /// Возвращает region ID через унаследованный shape-owner.
    pub(crate) const fn get_region_id(&self) -> i32 {
        self.shape_base.get_region_id()
    }

 /// Возвращает X через единственный shape-owner.
    pub(crate) const fn get_pos_x(&self) -> f32 {
        self.shape_base.get_pos_x()
    }

 /// Делегирует inherited `CShape::SetPosX`.
    pub(crate) const fn set_pos_x(&mut self, pos_x: f32) {
        self.shape_base.set_pos_x(pos_x);
    }

 /// Возвращает Y через единственный shape-owner.
    pub(crate) const fn get_pos_y(&self) -> f32 {
        self.shape_base.get_pos_y()
    }

 /// Делегирует inherited `CShape::SetPosY`.
    pub(crate) const fn set_pos_y(&mut self, pos_y: f32) {
        self.shape_base.set_pos_y(pos_y);
    }

 /// Возвращает X-клетку через единственный shape-owner.
    pub(crate) fn get_tile_x(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        self.shape_base.get_tile_x()
    }

 /// Возвращает Y-клетку через единственный shape-owner.
    pub(crate) fn get_tile_y(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        self.shape_base.get_tile_y()
    }

 /// Возвращает direction через единственный shape-owner.
    pub(crate) const fn get_direction(&self) -> i32 {
        self.shape_base.get_direction()
    }

 /// Делегирует inherited `CShape::SetPos`.
    pub(crate) const fn set_position(&mut self, position: i32) {
        self.shape_base.set_position(position);
    }

 /// Делегирует inherited `CShape::GetSpeed`.
    pub(crate) const fn get_speed(&self) -> f32 {
        self.shape_base.get_speed()
    }

 /// Присваивает region ID через унаследованный shape-owner.
    pub(crate) const fn set_region_id(&mut self, region_id: i32) {
        self.shape_base.set_region_id(region_id);
    }

 /// Присваивает обе координаты через единственный shape-owner.
    pub(crate) const fn set_pos_xy(&mut self, pos_x: f32, pos_y: f32) {
        self.shape_base.set_pos_xy(pos_x, pos_y);
    }

    pub(crate) const fn set_direction(&mut self, direction: i32) -> bool {
        self.shape_base.set_direction(direction)
    }

    pub(crate) const fn set_speed(&mut self, speed: f32) {
        self.shape_base.set_speed(speed);
    }

 /// Присваивает graphics ID через единственный shape-owner.
    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.shape_base.set_graphics_id(graphics_id);
    }

 /// Ставит shape в центр клетки через единственный shape-owner.
    pub(crate) fn set_tile_xy(&mut self, tile_x: i32, tile_y: i32) {
        self.shape_base.set_tile_xy(tile_x, tile_y);
    }

 /// Присваивает state единственного унаследованного shape-owner-а.
    pub(crate) const fn set_state(&mut self, state: u16) {
        self.shape_base.set_state(state);
    }

 /// Делегирует inherited `CShape::GetState`.
    pub(crate) const fn get_state(&self) -> u16 {
        self.shape_base.get_state()
    }

 /// Делегирует inherited `CShape::GetAction`.
    pub(crate) const fn get_action(&self) -> u16 {
        self.shape_base.get_action()
    }

 /// Делегирует inherited `CShape::SetAction`.
    pub(crate) const fn set_action(&mut self, action: u16) {
        self.shape_base.set_action(action);
    }

 /// Заимствует byte- extended states в исходном vector-order.
    pub(crate) fn ex_states(&self) -> &[u8] {
        &self.ex_states
    }

 /// Сохраняет доказанную раннюю очистку vector-а player decoder-ом.
    pub(crate) fn clear_ex_states(&mut self) {
        self.ex_states.clear();
    }

 /// Дописывает ненулевой byte-срез как исходный `SetExStates`.
    pub(crate) fn set_ex_states(&mut self, states: &[u8]) {
        if !states.is_empty() {
            self.ex_states.extend_from_slice(states);
        }
    }

 /// Возвращает действующий `m_bIsGod` без изменения представления.
    pub(crate) const fn is_god(&self) -> bool {
        self.is_god
    }

 /// Присваивает действующий `m_bIsGod` из player byte-array.
    pub(crate) const fn set_is_god(&mut self, is_god: bool) {
        self.is_god = is_god;
    }

 /// Делегирует точный base/shape byte-array owner единственному `CShape`.
    pub(crate) fn add_shape_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> bool {
        self.shape_base
            .add_to_byte_array(destination, include_child)
    }

 /// Делегирует обратный base/shape owner с сохранением caller cursor.
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
