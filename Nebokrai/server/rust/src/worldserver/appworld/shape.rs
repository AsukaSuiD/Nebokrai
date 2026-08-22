//! Владелец базового shape-состояния исторического `WorldServer`.
//!
//! Статус достигнутого inherited `CBaseObject::GetName`,
//! `CShape::GetRegionID` RVA `0x000530F0`, `CShape::SetRegionID` RVA
//! `0x00053100`, `CShape::SetDir` RVA `0x00053170`, `CShape::SetState` RVA
//! `0x000531D0`, `CShape::SetPosXY` RVA
//! `0x00053200`, `CShape::GetTileX/GetTileY` RVA `0x000D5120/0x000D5150`,
//! `CShape::SetTileXY` RVA `0x000D5290`,
//! `CShape::AddToByteArray` RVA `0x000D5180`,
//! `CShape::DecordFromByteArray` RVA `0x000D51B0`,
//! `CShape::AddShapeToByteArray` RVA `0x000D52B0` и
//! `CShape::DecordShapeFromByteArray` RVA `0x000D5370` — `IMPLEMENTED`; остальной корпус
//! ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:71-72` и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp:12`.
//!
//! Оба virtual-метода буквально читают и присваивают `m_lRegionID` без
//! проверок и побочных эффектов. Signed Windows `long` переносится как `i32`;
//! Rust-тип не объявляет совместимость с исходным ABI, layout или vtable.
//! `CShape::CShape` RVA `0x000D51E0` отдельно доказывает начальное значение
//! region ID `0` и первым вызывает готовый `CBaseObject::CBaseObject`.
//! Полный достигнутый constructor-state задаёт region/position/direction/pos/
//! state/action нулями и speed `2000.0`; helper сознательно не называется
//! `new` и не реализует `Default`, потому что father/child storage остаётся у
//! ещё не материализованной части base-owner-а.
//!
//! Byte-array сначала вызывает готовый `CBaseObject` owner, затем пишет GUID
//! как marker `0` либо `16` и, только после `16`, его 16 legacy-байтов. Далее
//! идут signed region, bit-exact `f32` X/Y, signed dir/pos, bit-exact speed и
//! два `u16` state/action. Exact `0x004D52DA..0x004D532C` подтверждает три
//! `fstp dword ptr [esp]` и общий четырёхбайтовый append target `0x004A3340`:
//! raw-касты float к `long` были ошибкой прототипа, числового преобразования
//! нет. Обратный owner читает тот же порядок, но доказанно отбрасывает
//! serialized `m_lPos` и ставит live поле в `0`.
//! `SetPosXY` буквально присваивает два `f32`. Tile-getter-ы вызывают virtual
//! `GetPosX/GetPosY`, а exact инструкции `0x004D5128..0x004D5144` и
//! `0x004D5158..0x004D5174` ставят x87 RC bits в `11`: conversion усекается к
//! нулю. Это `VERIFIED_DISASSEMBLY`; safe Rust блокирует лишь NaN, infinity и
//! значение вне signed `i32`, для которых достижимое поведение не доказано.
//!
//! CPlayer vtable `0x00543DE4` по slot-ам `+0x50..+0xA0` ссылается на эти же
//! CShape getters/setters и оба shape byte-owner-а; derived override в clone-
//! пути отсутствует. Safe slice/cursor возвращает локальный
//! `BLOCKED_MISSING_FACT` при коротком источнике после уже выполненных
//! присваиваний, не воспроизводя старый безразмерный overread через `unsafe`.
//!
//! Отдельные raw-блоки getter/setter удалены. STL, CRT и compiler-generated
//! механизмов в них не было. Историческая цепочка наследования теперь выражена
//! безопасной композицией `CBaseObject -> CShape -> CMoveShape -> CPlayer`;
//! единственные object type, `id` и `region_id` остаются у своих owners. Точный PDB задаёт
//! размеры старых классов `0x50/0x6C`, а текущий raw — непосредственный
//! base-constructor call, поэтому дополнительное дизассемблирование не
//! требовалось.

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

/// Достигнутая region-часть исходного `CShape`.
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

    /// Присваивает byte-exact имя через единственный base-owner.
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

    /// Возвращает bit-exact X исходного shape.
    pub(crate) const fn get_pos_x(&self) -> f32 {
        self.pos_x
    }

    /// Возвращает bit-exact Y исходного shape.
    pub(crate) const fn get_pos_y(&self) -> f32 {
        self.pos_y
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

    /// Присваивает signed region ID без проверки и дополнительных эффектов.
    pub(crate) const fn set_region_id(&mut self, region_id: i32) {
        self.region_id = region_id;
    }

    /// Присваивает обе bit-exact координаты без проверок и побочных эффектов.
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

    /// Присваивает bit-exact аргумент virtual `CShape::SetSpeed`.
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
        // WorldServer RVA 0x000D5370 сдвигает cursor через wire m_lPos, но
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
    // BLOCKED_MISSING_FACT: x87 `fistp dword` выдаёт integer-indefinite для
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
        // BLOCKED_MISSING_FACT: старый helper не получал длину источника и
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h

// ============================================================================
// FUNCTION: CShape::GetPosX
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:73
// RVA: 0x00053110
// ADDRESS: 00453110
// PROTOTYPE: float __thiscall GetPosX(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::SetPosX
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:74
// RVA: 0x00053120
// ADDRESS: 00453120
// PROTOTYPE: void __thiscall SetPosX(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::GetPosY
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:75
// RVA: 0x00053140
// ADDRESS: 00453140
// PROTOTYPE: float __thiscall GetPosY(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::SetPosY
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:76
// RVA: 0x00053150
// ADDRESS: 00453150
// PROTOTYPE: void __thiscall SetPosY(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::SetDir
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:78
// RVA: 0x00053170
// ADDRESS: 00453170
// PROTOTYPE: void __thiscall SetDir(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::SetPos
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:80
// RVA: 0x00053190
// ADDRESS: 00453190
// PROTOTYPE: void __thiscall SetPos(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::GetSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:81
// RVA: 0x000531A0
// ADDRESS: 004531a0
// PROTOTYPE: float __thiscall GetSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::SetSpeed
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:82
// RVA: 0x000531B0
// ADDRESS: 004531b0
// PROTOTYPE: void __thiscall SetSpeed(float param_1)
//
// IMPLEMENTED_OWNER: `CShape::set_speed` выше; concrete Rust owner сохраняет
// однополевое присваивание без собственного virtual ABI.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::GetState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:83
// RVA: 0x000531C0
// ADDRESS: 004531c0
// PROTOTYPE: ushort __thiscall GetState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::SetState
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:84
// RVA: 0x000531D0
// ADDRESS: 004531d0
// PROTOTYPE: void __thiscall SetState(ushort param_1)
//
// /* public: virtual void __thiscall CShape::SetState(unsigned short) */
//
// Реализация находится в `CShape::set_state`. Exact CPlayer vtable slot
// `0x00543E68` указывает на `0x004531D0`; инструкции записывают входной `AX`
// ровно в `word ptr [ECX+0x68]`, то есть PDB-поле `m_wState`.

// ============================================================================
// FUNCTION: CShape::GetAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:85
// RVA: 0x000531E0
// ADDRESS: 004531e0
// PROTOTYPE: ushort __thiscall GetAction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::SetAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:86
// RVA: 0x000531F0
// ADDRESS: 004531f0
// PROTOTYPE: void __thiscall SetAction(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWorldCityRegion::tagBuild::tagBuild
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp
// RVA: 0x00079660
// ADDRESS: 00479660
// PROTOTYPE: undefined __thiscall tagBuild(tagBuild * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047982e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp
// RVA: 0x0007982E
// ADDRESS: 0047982e
// PROTOTYPE: undefined Catch@0047982e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CShape::~CShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp:24
// RVA: 0x000D5110
// ADDRESS: 004d5110
// PROTOTYPE: void __thiscall ~CShape(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CShape::AddToByteArray` RVA `0x000D5180` и
// `CShape::DecordFromByteArray` RVA `0x000D51B0` находятся выше; обе функции
// сохраняют композицию base-owner -> shape-owner и normal return `true`.

// ============================================================================
// FUNCTION: CShape::CShape
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp:12
// RVA: 0x000D51E0
// ADDRESS: 004d51e0
// PROTOTYPE: undefined __thiscall CShape(void)
//
// IMPLEMENTED выше; base-construction и все собственные scalar defaults
// сохранены без копирования старого ABI/vtable.

// ============================================================================
// FUNCTION: CShape::SetTileXY
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp:38
// RVA: 0x000D5290
// ADDRESS: 004d5290
// PROTOTYPE: void __thiscall SetTileXY(long param_1, long param_2)
//
// IMPLEMENTED выше: оба signed `long` переводятся в `f32` и смещаются на
// `0.5`, то есть shape ставится в центр указанной клетки.

// ============================================================================
// FUNCTION: CShape::AddShapeToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp:80
// RVA: 0x000D52B0
// ADDRESS: 004d52b0
// PROTOTYPE: bool __thiscall AddShapeToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// IMPLEMENTED выше; exact float/GUID порядок и normal `true` сохранены.

// ============================================================================
// FUNCTION: CShape::DecordShapeFromByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp:95
// RVA: 0x000D5370
// ADDRESS: 004d5370
// PROTOTYPE: bool __thiscall DecordShapeFromByteArray(uchar * param_1, long * param_2)
//
// IMPLEMENTED выше; marker-GUID, cursor и `wire m_lPos -> live 0` сохранены.

// ============================================================================
// FUNCTION: CShape::GetDir
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.h:77
// RVA: 0x000DEA40
// ADDRESS: 004dea40
// PROTOTYPE: long __thiscall GetDir(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00530a70
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\shape.cpp
// RVA: 0x00130A70
// ADDRESS: 00530a70
// PROTOTYPE: undefined Unwind@00530a70()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
