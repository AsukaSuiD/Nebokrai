//! Базовый `CRegion`: spatial/persistence поверхность регионов (cells,
//! switches, resource `regions/{ID}.rgn`, byte-array codec). Исходники
//! `appserver/region.h/.cpp` исторического GameServer; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb`.
//!
//! Codec намеренно не round-trip симметричен: serializer после base пишет лишь
//! четырёхбайтовый `lState` каждого switch, а decoder дополнительно читает
//! resource ID и IEEE-754 scale перед размерами — точная разница подтверждена
//! EXE и сохранена. `RegionCell` хранит исходный DWORD byte-exact (block — биты
//! `0..2`, security — `3..5`, city-war marker — `6..7`). Отсутствующий/короткий
//! storage и переполнение signed index дают локальный `BLOCKED_MISSING_FACT`, а
//! не воспроизведение pointer UB; подтверждённая null-pointer ветка security
//! (`SAFE` при нулевом индексе) сохранена, `GetBlock/SetBlock` такого guard-а не
//! имеют. Monster, loot и player death/PK owners не перенесены и остаются
//! неисследованными (RAW).
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#региональное-пространство

use super::baseobject::{BaseObjectDecodeError, CBaseObject};
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};

const REGION_RESOURCE_HEADER: &[u8; 7] = b"CLS-RGN";
const REGION_RESOURCE_VERSION: i32 = 1;
const REGION_SWITCH_SIZE: usize = 0x14;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RegionCell {
    pub bytes: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RegionSwitch {
    bytes: [u8; REGION_SWITCH_SIZE],
}

impl RegionSwitch {
    fn read_i32(self, offset: usize) -> i32 {
        LegacyReader::at(&self.bytes, offset)
            .and_then(|mut reader| reader.read_i32())
            .expect("поле switch занимает четыре байта")
    }

    pub fn state(self) -> i32 {
        self.read_i32(0)
    }

    pub fn region_id(self) -> i32 {
        self.read_i32(4)
    }

    pub fn coordinate_x(self) -> i32 {
        self.read_i32(8)
    }

    pub fn coordinate_y(self) -> i32 {
        self.read_i32(12)
    }

    pub fn direction(self) -> i32 {
        self.read_i32(16)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegionStorageBlock {
    InvalidDimensions { width: i32, height: i32 },
    CellStorageMismatch { expected: usize, available: usize },
    UninitializedField { field: &'static str },
    TooManySwitches { count: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegionDecodeError {
    Base(BaseObjectDecodeError),
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    Storage(RegionStorageBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionResourceWrite {
    pub path: Vec<u8>,
    pub bytes: Vec<u8>,
}

impl RegionCell {
    pub fn block(self) -> u8 {
        self.bytes[0] & 0x07
    }

    pub fn set_block(&mut self, block: u8) {
        self.bytes[0] = (self.bytes[0] & 0xF8) | (block & 0x07);
    }

    pub fn security(self) -> RegionSecurity {
        RegionSecurity((self.bytes[0] >> 3) & 0x07)
    }

    pub fn city_war_marker(self) -> u8 {
        (self.bytes[0] >> 6) & 0x03
    }

    pub fn switch_id(self) -> u16 {
        LegacyReader::at(&self.bytes, 2)
            .and_then(|mut reader| reader.read_u16())
            .expect("switch ID занимает два байта")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionSecurity(u8);

impl RegionSecurity {
    pub const FREE: Self = Self(0);
    pub const FIGHT: Self = Self(1);
    pub const SAFE: Self = Self(2);
    pub const CITY_WAR: Self = Self(3);

    pub fn value(self) -> u8 {
        self.0
    }
}

/// BLOCKED_MISSING_FACT: оригинал вычисляет raw pointer wrapping `imul/add`;
/// реакция на переполнение либо storage, не соответствующий width/height, не
/// доказана и не получает придуманного safe-результата.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionCellAccessBlock {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub available_cells: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RegionReturnPoint {
    pub region_id: i32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub direction: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RegionRandomPosition {
    pub x: i32,
    pub y: i32,
    pub found: bool,
}

pub trait RegionRandomContext {
    /// Выполняет исходный `random(bound)`. Включая реакцию на неположительный
    /// bound, контракт принадлежит RNG-owner-у, а не spatial family.
    fn random_below(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CRegion {
    base_object: CBaseObject,
    file_name: Vec<u8>,
    region_type: i32,
    resource_id: i32,
    exp_scale_bits: u32,
    pub width: i32,
    pub height: i32,
    country: Option<u8>,
    notify: Option<i32>,
    last_notify_hurt_time: u32,
    last_notify_kill_time: u32,
    pub cells: Vec<RegionCell>,
    switches: Vec<RegionSwitch>,
}

impl Default for CRegion {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl CRegion {
    pub const fn with_constructor_defaults() -> Self {
        let mut base_object = CBaseObject::with_reached_constructor_defaults();
        base_object.set_type(200);
        Self {
            base_object,
            file_name: Vec::new(),
            region_type: 0,
            resource_id: 0,
            exp_scale_bits: 1.0f32.to_bits(),
            width: 0,
            height: 0,
            country: None,
            notify: None,
            last_notify_hurt_time: 0,
            last_notify_kill_time: 0,
            cells: Vec::new(),
            switches: Vec::new(),
        }
    }

    pub const fn get_id(&self) -> i32 {
        self.base_object.get_id()
    }

    pub const fn set_id(&mut self, id: i32) {
        self.base_object.set_id(id);
    }

    pub fn get_name(&self) -> &[u8] {
        self.base_object.get_name()
    }

    pub const fn country(&self) -> Option<u8> {
        self.country
    }

    /// Выполняет точный hurt-notify gate `CPlayer::OnBeenHurted`: ненулевой
    /// signed interval сравнивается как `uint` с wrapping-разностью двух
    /// `timeGetTime`, а второй замер часов сохраняется только при срабатывании.
    pub fn take_hurt_notice_due(&mut self, mut now_milliseconds: impl FnMut() -> u32) -> bool {
        let Some(interval_ms) = self.notify.filter(|interval| *interval != 0) else {
            return false;
        };
        if (interval_ms as u32) >= now_milliseconds().wrapping_sub(self.last_notify_hurt_time) {
            return false;
        }
        self.last_notify_hurt_time = now_milliseconds();
        true
    }

    /// Выполняет sibling kill-notify gate `CPlayer::OnDied`: EXE сравнивает
    /// строго `last + interval < now` и только затем читает часы для записи.
    pub fn take_kill_notice_due(&mut self, mut now_milliseconds: impl FnMut() -> u32) -> bool {
        let Some(interval_ms) = self.notify.filter(|interval| *interval != 0) else {
            return false;
        };
        if self.last_notify_kill_time.wrapping_add(interval_ms as u32) >= now_milliseconds() {
            return false;
        }
        self.last_notify_kill_time = now_milliseconds();
        true
    }

    pub fn file_name(&self) -> &[u8] {
        &self.file_name
    }

    pub const fn region_type(&self) -> i32 {
        self.region_type
    }

    pub const fn resource_id(&self) -> i32 {
        self.resource_id
    }

    pub const fn width(&self) -> i32 {
        self.width
    }

    pub const fn height(&self) -> i32 {
        self.height
    }

    pub const fn exp_scale_bits(&self) -> u32 {
        self.exp_scale_bits
    }

    pub fn exp_scale(&self) -> f32 {
        f32::from_bits(self.exp_scale_bits)
    }

    pub fn set_name(&mut self, name: &[u8]) {
        self.base_object.set_name(name);
    }

    /// Выполняет исходный virtual `New`: очищает прежние switches/cells,
    /// создаёт zero-filled cell storage и возвращает `1`.
    pub fn new_region(&mut self) -> Result<i32, RegionStorageBlock> {
        self.recreate_storage()?;
        Ok(1)
    }

    /// Возвращает switch по исходному 1-based ID либо старый `nullptr`.
    pub fn get_switch(&self, id: i32) -> Option<&RegionSwitch> {
        let index = usize::try_from(id.checked_sub(1)?).ok()?;
        self.switches.get(index)
    }

    /// Выполняет coordinate-overload: cell `lSwitch` также является 1-based ID.
    pub fn get_switch_at(
        &self,
        x: i32,
        y: i32,
    ) -> Result<Option<&RegionSwitch>, RegionCellAccessBlock> {
        let Some(cell) = self.get_cell(x, y)? else {
            return Ok(None);
        };
        Ok(self.get_switch(i32::from(cell.switch_id())))
    }

    /// Дописывает точный GameServer region-wire после `CBaseObject`.
    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, RegionStorageBlock> {
        let country = self.country.ok_or(RegionStorageBlock::UninitializedField {
            field: "m_btCountry",
        })?;
        let notify = self
            .notify
            .ok_or(RegionStorageBlock::UninitializedField { field: "m_lNotify" })?;
        let cell_count = self.validated_cell_count()?;
        let switch_count = i32::try_from(self.switches.len()).map_err(|_| {
            RegionStorageBlock::TooManySwitches {
                count: self.switches.len(),
            }
        })?;

        let _ = self
            .base_object
            .add_to_byte_array(destination, include_child);
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(self.region_type);
        writer.write_i32(self.width);
        writer.write_i32(self.height);
        writer.write_u8(country);
        writer.write_i32(notify);
        for cell in &self.cells[..cell_count] {
            writer.write_bytes(&cell.bytes);
        }
        writer.write_i32(switch_count);
        for switch in &self.switches {
            writer.write_bytes(&switch.bytes[..4]);
        }
        Ok(true)
    }

    /// Читает точный GameServer region-wire, включая его асимметрию с serializer-ом.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<bool, RegionDecodeError> {
        self.base_object
            .decord_from_byte_array(source, cursor, include_child)
            .map_err(RegionDecodeError::Base)?;
        self.region_type = read_region_i32(source, cursor, "m_lRegionType")?;
        self.resource_id = read_region_i32(source, cursor, "m_lResourceID")?;
        self.exp_scale_bits = read_region_u32(source, cursor, "m_fExpScale")?;
        self.width = read_region_i32(source, cursor, "m_lWidth")?;
        self.height = read_region_i32(source, cursor, "m_lHeight")?;
        self.country = Some(read_region_u8(source, cursor, "m_btCountry")?);
        self.notify = Some(read_region_i32(source, cursor, "m_lNotify")?);
        self.recreate_storage()
            .map_err(RegionDecodeError::Storage)?;

        for cell in &mut self.cells {
            cell.bytes
                .copy_from_slice(read_region_bytes(source, cursor, 4, "m_pCell")?);
        }
        let switch_count = read_region_i32(source, cursor, "m_vectorSwitch.size")?;
        for _ in 0..switch_count.max(0) {
            let mut switch = RegionSwitch::default();
            switch.bytes.copy_from_slice(read_region_bytes(
                source,
                cursor,
                REGION_SWITCH_SIZE,
                "m_vectorSwitch[]",
            )?);
            self.switches.push(switch);
        }
        Ok(true)
    }

    /// Материализует `Save` до границы filesystem-owner-а: точный path и bytes.
    pub fn save_resource(&mut self) -> Result<RegionResourceWrite, RegionStorageBlock> {
        let path = self.resource_path();
        self.file_name.clone_from(&path);
        let cell_count = self.validated_cell_count()?;
        let switch_count = i32::try_from(self.switches.len()).map_err(|_| {
            RegionStorageBlock::TooManySwitches {
                count: self.switches.len(),
            }
        })?;

        let mut bytes = Vec::new();
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_bytes(REGION_RESOURCE_HEADER);
        writer.write_i32(REGION_RESOURCE_VERSION);
        writer.write_i32(self.region_type);
        writer.write_i32(self.width);
        writer.write_i32(self.height);
        for cell in &self.cells[..cell_count] {
            writer.write_bytes(&cell.bytes);
        }
        writer.write_i32(switch_count);
        for switch in &self.switches {
            writer.write_bytes(&switch.bytes);
        }
        Ok(RegionResourceWrite { path, bytes })
    }

    /// Читает содержимое уже открытого `regions/{ID}.rgn`; `None` сохраняет
    /// исходный open-failure `false`.
    pub fn load_resource(&mut self, source: Option<&[u8]>) -> Result<bool, RegionDecodeError> {
        let Some(source) = source else {
            return Ok(false);
        };
        if source.get(..REGION_RESOURCE_HEADER.len()) != Some(REGION_RESOURCE_HEADER) {
            return Ok(false);
        }
        let mut cursor = REGION_RESOURCE_HEADER.len();
        let Ok(mut version_reader) = LegacyReader::at(source, cursor) else {
            return Ok(false);
        };
        let Ok(version) = version_reader.read_i32() else {
            return Ok(false);
        };
        cursor = version_reader.position();
        if version != REGION_RESOURCE_VERSION {
            return Ok(false);
        }

        self.file_name = self.resource_path();
        self.region_type = read_region_i32(source, &mut cursor, "m_lRegionType")?;
        self.width = read_region_i32(source, &mut cursor, "m_lWidth")?;
        self.height = read_region_i32(source, &mut cursor, "m_lHeight")?;
        self.recreate_storage()
            .map_err(RegionDecodeError::Storage)?;
        for cell in &mut self.cells {
            cell.bytes
                .copy_from_slice(read_region_bytes(source, &mut cursor, 4, "m_pCell")?);
        }
        let switch_count = read_region_i32(source, &mut cursor, "m_vectorSwitch.size")?;
        for _ in 0..switch_count.max(0) {
            let mut switch = RegionSwitch::default();
            switch.bytes.copy_from_slice(read_region_bytes(
                source,
                &mut cursor,
                REGION_SWITCH_SIZE,
                "m_vectorSwitch[]",
            )?);
            self.switches.push(switch);
        }
        Ok(true)
    }

    pub fn get_return_point(&self) -> RegionReturnPoint {
        RegionReturnPoint::default()
    }

    pub fn get_cell(&self, x: i32, y: i32) -> Result<Option<&RegionCell>, RegionCellAccessBlock> {
        let Some(index) = self.cell_index(x, y)? else {
            return Ok(None);
        };
        if let Some(cell) = self.cells.get(index) {
            return Ok(Some(cell));
        }
        if index == 0 && self.cells.is_empty() {
            return Ok(None);
        }
        Err(self.block(x, y))
    }

    pub fn set_block(&mut self, x: i32, y: i32, block: u8) -> Result<(), RegionCellAccessBlock> {
        let Some(index) = self.cell_index(x, y)? else {
            return Ok(());
        };
        let unavailable = self.block(x, y);
        let cell = self.cells.get_mut(index).ok_or(unavailable)?;
        cell.set_block(block);
        Ok(())
    }

    pub fn get_block(&self, x: i32, y: i32) -> Result<u8, RegionCellAccessBlock> {
        let Some(index) = self.cell_index(x, y)? else {
            return Ok(2);
        };
        self.cells
            .get(index)
            .copied()
            .map(RegionCell::block)
            .ok_or_else(|| self.block(x, y))
    }

    pub fn get_security(&self, x: i32, y: i32) -> Result<RegionSecurity, RegionCellAccessBlock> {
        Ok(self
            .get_cell(x, y)?
            .map_or(RegionSecurity::SAFE, |cell| cell.security()))
    }

    pub fn get_random_pos<Context: RegionRandomContext>(
        &self,
        context: &mut Context,
    ) -> Result<RegionRandomPosition, RegionCellAccessBlock> {
        self.get_random_pos_in_range(0, 0, self.width, self.height, context)
    }

    pub fn get_random_pos_in_range<Context: RegionRandomContext>(
        &self,
        mut left: i32,
        mut top: i32,
        mut range_width: i32,
        mut range_height: i32,
        context: &mut Context,
    ) -> Result<RegionRandomPosition, RegionCellAccessBlock> {
        loop {
            let right = left.wrapping_add(range_width);
            let bottom = top.wrapping_add(range_height);
            if left < self.width
                && top < self.height
                && range_width >= 0
                && range_height >= 0
                && right >= 0
                && bottom >= 0
            {
                if left < 0 {
                    left = 0;
                    range_width = right;
                }
                if top < 0 {
                    top = 0;
                    range_height = bottom;
                }
            } else {
                left = 0;
                top = 0;
                range_width = self.width;
                range_height = self.height;
            }

            if self.width < range_width.wrapping_add(left) {
                range_width = self.width.wrapping_sub(left);
            }
            if self.height < range_height.wrapping_add(top) {
                range_height = self.height.wrapping_sub(top);
            }

            for _ in 0..1000 {
                let x = context.random_below(range_width).wrapping_add(left);
                let y = context.random_below(range_height).wrapping_add(top);
                if self.position_is_open(x, y)? {
                    return Ok(RegionRandomPosition { x, y, found: true });
                }
            }

            let right = left.wrapping_add(range_width);
            let bottom = top.wrapping_add(range_height);
            let mut x = left;
            while x < right {
                let mut y = top;
                while y < bottom {
                    if self.position_is_open(x, y)? {
                        return Ok(RegionRandomPosition { x, y, found: true });
                    }
                    y = y.wrapping_add(1);
                }
                x = x.wrapping_add(1);
            }

            if left < 1 && top < 1 && self.width <= range_width && self.height <= range_height {
                return Ok(RegionRandomPosition {
                    x: context.random_below(self.width),
                    y: context.random_below(self.height),
                    found: false,
                });
            }

            top = top.wrapping_sub(10);
            range_width = range_width.wrapping_add(20);
            left = left.wrapping_sub(10);
            range_height = range_height.wrapping_add(20);
        }
    }

    fn position_is_open(&self, x: i32, y: i32) -> Result<bool, RegionCellAccessBlock> {
        Ok(self
            .get_cell(x, y)?
            .is_some_and(|cell| cell.block() == 0 && cell.switch_id() == 0))
    }

    fn cell_index(&self, x: i32, y: i32) -> Result<Option<usize>, RegionCellAccessBlock> {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return Ok(None);
        }
        let index = self
            .width
            .checked_mul(y)
            .and_then(|row| row.checked_add(x))
            .and_then(|index| usize::try_from(index).ok())
            .ok_or_else(|| self.block(x, y))?;
        Ok(Some(index))
    }

    fn block(&self, x: i32, y: i32) -> RegionCellAccessBlock {
        RegionCellAccessBlock {
            x,
            y,
            width: self.width,
            height: self.height,
            available_cells: self.cells.len(),
        }
    }

    fn resource_path(&self) -> Vec<u8> {
        format!("regions/{}.rgn", self.base_object.get_id()).into_bytes()
    }

    fn recreate_storage(&mut self) -> Result<(), RegionStorageBlock> {
        self.cells.clear();
        self.switches.clear();
        let cell_count = checked_cell_count(self.width, self.height)?;
        self.cells.resize(cell_count, RegionCell::default());
        Ok(())
    }

    fn validated_cell_count(&self) -> Result<usize, RegionStorageBlock> {
        let expected = checked_cell_count(self.width, self.height)?;
        if self.cells.len() < expected {
            return Err(RegionStorageBlock::CellStorageMismatch {
                expected,
                available: self.cells.len(),
            });
        }
        Ok(expected)
    }
}

fn checked_cell_count(width: i32, height: i32) -> Result<usize, RegionStorageBlock> {
    let (Ok(width_usize), Ok(height_usize)) = (usize::try_from(width), usize::try_from(height))
    else {
        // BLOCKED_MISSING_FACT: x86 `imul/shl` передавал wrapped signed размер
        // allocator-у; safe runtime-реакция для отрицательного размера не задана.
        return Err(RegionStorageBlock::InvalidDimensions { width, height });
    };
    let Some(cell_count) = width_usize.checked_mul(height_usize) else {
        return Err(RegionStorageBlock::InvalidDimensions { width, height });
    };
    if cell_count > (u32::MAX as usize) / 4 || cell_count.checked_mul(4).is_none() {
        return Err(RegionStorageBlock::InvalidDimensions { width, height });
    }
    Ok(cell_count)
}

fn read_region_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, RegionDecodeError> {
    let mut reader = region_reader(source, *cursor, 4, field)?;
    let value = reader
        .read_i32()
        .map_err(|block| region_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_region_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, RegionDecodeError> {
    let mut reader = region_reader(source, *cursor, 4, field)?;
    let value = reader
        .read_u32()
        .map_err(|block| region_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_region_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, RegionDecodeError> {
    let mut reader = region_reader(source, *cursor, 1, field)?;
    let value = reader
        .read_u8()
        .map_err(|block| region_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_region_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    needed: usize,
    field: &'static str,
) -> Result<&'a [u8], RegionDecodeError> {
    let mut reader = region_reader(source, *cursor, needed, field)?;
    let bytes = reader
        .read_bytes(needed)
        .map_err(|block| region_read_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn region_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    needed: usize,
    field: &'static str,
) -> Result<LegacyReader<'source>, RegionDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| RegionDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed,
        available: block.available,
    })
}

fn region_read_error(
    field: &'static str,
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> RegionDecodeError {
    RegionDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
