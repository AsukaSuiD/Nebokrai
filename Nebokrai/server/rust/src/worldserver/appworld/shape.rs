//! Владелец базового shape-состояния исторического `WorldServer`.
//!
//! Rust-owner включает унаследованный `CBaseObject::GetName`, полный набор
//! скалярных accessor-ов `CShape`, `CShape::GetRegionID`,
//! `CShape::SetRegionID`, `CShape::SetDir`,
//! `CShape::SetState`, `CShape::SetPosXY`
//! `CShape::GetTileX/GetTileY`,
//! `CShape::SetTileXY`,
//! `CShape::AddToByteArray`,
//! `CShape::DecordFromByteArray`,
//! `CShape::AddShapeToByteArray` и
//! `CShape::DecordShapeFromByteArray`. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Оба virtual-метода буквально читают и присваивают `m_lRegionID` без
//! проверок и побочных эффектов. Signed Windows `long` переносится как `i32`;
//! Rust-тип не объявляет совместимость с исходным ABI, layout или vtable.
//! Базовый виртуальный `GetFigure` не читает объект и всегда
//! возвращает нуль; конкретный `CMonster` задаёт отдельное переопределение
//! через настройку монстра. Его оригинал-дубликат из `union.cpp` закрыт здесь.
//! `CShape::CShape` отдельно доказывает начальное значение
//! region ID `0` и первым вызывает готовый `CBaseObject::CBaseObject`.
//! Полный действующий constructor-state задаёт region/position/direction/pos/
//! state/action нулями и speed `2000.0`; helper сознательно не называется
//! `new` и не реализует `Default`, потому что father/child storage остаётся у
//! ещё не материализованной части base-owner-а.
//!
//! Byte-array сначала вызывает готовый `CBaseObject` owner, затем пишет GUID
//! как marker `0` либо `16` и, только после `16`, его 16 legacy-байтов. Далее
//! идут signed region, bit- `f32` X/Y, signed dir/pos, bit- speed и
//! два `u16` state/action. подтверждает три
//! `fstp dword ptr [esp]` и общий четырёхбайтовый append target:
//! оригинал-касты float к `long` были ошибкой прототипа, числового преобразования
//! нет. Обратный owner читает тот же порядок, но доказанно отбрасывает
//! serialized `m_lPos` и ставит live поле в `0`.
//! `SetPosXY` буквально присваивает два `f32`. Tile-getter-ы вызывают virtual
//! `GetPosX/GetPosY`, а инструкции и
//! ставят x87 RC bits в `11`: conversion усекается к
//! нулю. Это; safe Rust блокирует лишь NaN, infinity и
//! значение вне signed `i32`, для которых достижимое поведение не доказано.
//!
//! CPlayer vtable по slot-ам `+0x50..+0xA0` ссылается на эти же
//! CShape getters/setters и оба shape byte-owner-а; derived override в clone-
//! пути отсутствует. Safe slice/cursor возвращает локальную типизированную
//! ошибку при коротком источнике после уже выполненных
//! присваиваний, не воспроизводя старый безразмерный overread через `unsafe`.
//!
//! Отдельные оригинал-блоки getter/setter удалены. STL, CRT и compiler-generated
//! механизмов в них не было. Историческая цепочка наследования теперь выражена
//! безопасной композицией `CBaseObject -> CShape -> CMoveShape -> CPlayer`;
//! единственные object type, `id` и `region_id` остаются у своих owners. PDB задаёт
//! размеры старых классов `0x50/0x6C`, а текущий оригинал — непосредственный
//! base-constructor call, поэтому дополнительное оригинал не
//! требовалось.
//! `SetPosX/SetPosY` не присваивают поле напрямую: тела берут вторую
//! координату через virtual getter и вызывают virtual `SetPosXY`. В
//! действующих World vtable эти slots не override-ятся, поэтому композиция
//! сохраняет те же X/Y и порядок чтения без искусственного virtual ABI.
//! Destructor `CShape` только передаёт lifecycle в `CBaseObject`; Rust field
//! ownership делает это автоматически.

use std::error::Error;
use std::fmt;

use super::baseobject::{BaseObjectDecodeError, CBaseObject};

/// Ошибка безопасной границы старого shape byte-array decoder-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShapeDecodeError {
    BaseObject(BaseObjectDecodeError),
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

/// Safe-граница x87 `f32 -> signed long` для tile-координаты.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShapeTileCoordinateBlock {
    pub(crate) axis: &'static str,
    pub(crate) value_bits: u32,
}

impl fmt::Display for ShapeDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BaseObject(error) => error.fmt(formatter),
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
        }
    }
}

impl Error for ShapeDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BaseObject(error) => Some(error),
            Self::UnexpectedEnd { .. } => None,
        }
    }
}

impl From<BaseObjectDecodeError> for ShapeDecodeError {
    fn from(error: BaseObjectDecodeError) -> Self {
        Self::BaseObject(error)
    }
}

/// Действующая region-часть исходного `CShape`.
pub(crate) struct CShape {
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
 /// Повторяет базовый виртуальный `CShape::GetFigure`: нуль без чтения состояния.
    pub(crate) const fn get_figure(&self) -> u8 {
        0
    }

 /// Создаёт только доказанное начальное region-состояние исходного
 /// конструктора.
    pub(crate) const fn with_constructor_region_default() -> Self {
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

 /// Возвращает object type через унаследованный `CBaseObject` owner.
    pub(crate) const fn get_type(&self) -> i32 {
        self.base_object.get_type()
    }

 /// Присваивает object type через унаследованный `CBaseObject` owner.
    pub(crate) const fn set_type(&mut self, object_type: i32) {
        self.base_object.set_type(object_type);
    }

 /// Возвращает ID через унаследованный `CBaseObject` owner.
    pub(crate) const fn get_id(&self) -> i32 {
        self.base_object.get_id()
    }

 /// Присваивает ID через унаследованный `CBaseObject` owner.
    pub(crate) const fn set_id(&mut self, id: i32) {
        self.base_object.set_id(id);
    }

 /// Заимствует GUID через единственный унаследованный `CBaseObject` owner.
    pub(crate) const fn get_ex_id(&self) -> &crate::public::guid::CGuid {
        self.base_object.get_ex_id()
    }

 /// Копирует GUID через единственный унаследованный `CBaseObject` owner.
    pub(crate) const fn set_ex_id(&mut self, ex_id: &crate::public::guid::CGuid) {
        self.base_object.set_ex_id(ex_id);
    }

 /// Заимствует имя через единственный унаследованный `CBaseObject` owner.
    pub(crate) fn get_name(&self) -> &[u8] {
        self.base_object.get_name()
    }

 /// Присваивает byte- имя через единственный base-owner.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.base_object.set_name(name);
    }

 /// Присваивает signed graphics ID через единственный base-owner.
    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.base_object.set_graphics_id(graphics_id);
    }

 /// Возвращает signed region ID без преобразования битового шаблона.
    pub(crate) const fn get_region_id(&self) -> i32 {
        self.region_id
    }

 /// Возвращает bit- X исходного shape.
    pub(crate) const fn get_pos_x(&self) -> f32 {
        self.pos_x
    }

 /// Повторяет `SetPosX`: сохраняет текущий Y и делегирует `SetPosXY`.
    pub(crate) const fn set_pos_x(&mut self, pos_x: f32) {
        self.set_pos_xy(pos_x, self.get_pos_y());
    }

 /// Возвращает bit- Y исходного shape.
    pub(crate) const fn get_pos_y(&self) -> f32 {
        self.pos_y
    }

 /// Повторяет `SetPosY`: сохраняет текущий X и делегирует `SetPosXY`.
    pub(crate) const fn set_pos_y(&mut self, pos_y: f32) {
        self.set_pos_xy(self.get_pos_x(), pos_y);
    }

 /// Возвращает X-клетку с точным x87 truncation toward zero.
    pub(crate) fn get_tile_x(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        truncate_tile_coordinate(self.get_pos_x(), "X")
    }

 /// Возвращает Y-клетку с точным x87 truncation toward zero.
    pub(crate) fn get_tile_y(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        truncate_tile_coordinate(self.get_pos_y(), "Y")
    }

 /// Возвращает signed direction исходного shape.
    pub(crate) const fn get_direction(&self) -> i32 {
        self.direction
    }

 /// Присваивает signed legacy `m_lPos` без дополнительных эффектов.
    pub(crate) const fn set_position(&mut self, position: i32) {
        self.position = position;
    }

 /// Возвращает bit- скорость shape.
    pub(crate) const fn get_speed(&self) -> f32 {
        self.speed
    }

 /// Присваивает signed region ID без проверки и дополнительных эффектов.
    pub(crate) const fn set_region_id(&mut self, region_id: i32) {
        self.region_id = region_id;
    }

 /// Присваивает обе bit- координаты без проверок и побочных эффектов.
    pub(crate) const fn set_pos_xy(&mut self, pos_x: f32, pos_y: f32) {
        self.pos_x = pos_x;
        self.pos_y = pos_y;
    }

 /// Сохраняет исходную проверку `0 <= direction < 8`.
    pub(crate) const fn set_direction(&mut self, direction: i32) -> bool {
        if direction < 0 || direction >= 8 {
            return false;
        }
        self.direction = direction;
        true
    }

 /// Присваивает bit- аргумент virtual `CShape::SetSpeed`.
    pub(crate) const fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

 /// Ставит shape в центр двух signed tile-координат.
    pub(crate) fn set_tile_xy(&mut self, tile_x: i32, tile_y: i32) {
        self.pos_x = tile_x as f32 + 0.5;
        self.pos_y = tile_y as f32 + 0.5;
    }

 /// Присваивает полный unsigned 16-битный shape-state без побочных эффектов.
    pub(crate) const fn set_state(&mut self, state: u16) {
        self.state = state;
    }

 /// Возвращает полный unsigned 16-битный shape-state.
    pub(crate) const fn get_state(&self) -> u16 {
        self.state
    }

 /// Возвращает полный unsigned 16-битный shape-action.
    pub(crate) const fn get_action(&self) -> u16 {
        self.action
    }

 /// Присваивает полный unsigned 16-битный shape-action.
    pub(crate) const fn set_action(&mut self, action: u16) {
        self.action = action;
    }

 /// Дописывает base и shape части в точном legacy-порядке.
    pub(crate) fn add_to_byte_array(&self, destination: &mut Vec<u8>, include_child: bool) -> bool {
        let _ = self
            .base_object
            .add_to_byte_array(destination, include_child);
        let _ = self.add_shape_to_byte_array(destination);
        true
    }

 /// Читает base и shape части, сохраняя cursor и уже выполненные мутации.
    pub(crate) fn decord_from_byte_array(
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
                .set_ex_id(&crate::public::guid::CGuid::GUID_INVALID);
        } else {
            let bytes = read_shape_array::<16>(source, cursor, "m_guExID")?;
            self.base_object
                .set_ex_id(&crate::public::guid::CGuid::from_legacy_bytes(bytes));
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
 // публиковать именно этот результат соседним owner-ам не доказаны.
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
