//! GameServer-владелец базовой spatial/persistence поверхности `CRegion`.
//!
//! `GetCell` RVA `0x0002AC10`, `SetBlock` `0x0002AC50`, `GetBlock`
//! `0x0007BCC0`, virtual `GetSecurity` `0x000854F0`, базовый
//! `GetReturnPoint` `0x000F0280`, random-position family
//! `0x000F02C0/0x000F04D0`, serializer `0x000F0540`, обе `GetSwitch`
//! `0x000F0620/0x000F0670`, `New` `0x000F06C0`, resource `Save/Load`
//! `0x000F0830/0x000F0EA0` и decoder
//! `0x000F1070` имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходники
//! `region.h/.cpp`. PDB и EXE фиксируют signed width/height по `+0x6C/+0x70`,
//! cell pointer `+0x84`, row-major index и размер `tagCell == 4`. В random
//! family EXE отдельно подтверждает `lSwitch` как little-endian word `+2`.
//!
//! Exact EXE фиксирует region fields `+0x60..+0x80`, cell pointer `+0x84` и
//! switch-vector `+0x88` (`_Myfirst +0x8C`). Constructor ставит object type
//! `200`, region/resource/size и notify timestamps в `0`, scale в `1.0`, но не
//! инициализирует country/notify; достигнутый Rust prefix хранит их как
//! `Option`, а полный constructor остаётся RAW вместе с недостигнутой base
//! child-tree. `New` сначала
//! освобождает старые cells/switches, затем создаёт zero-filled block и
//! возвращает `1`; ошибочные ранние `return` raw-декомпилята опровергнуты
//! последовательным EXE control flow `0x004F06C0..0x004F0742`.
//!
//! Resource `regions/{ID}.rgn` равен `CLS-RGN`, signed version `1`, region
//! type, width/height, `width*height*4` cell-байт, signed count и полные
//! 20-байтовые switches. `Save` возвращает `0` только при невозможности открыть
//! файл, `1` после close; Linux boundary отдаёт path и точные bytes вызывающему
//! filesystem-owner-у, поскольку runtime открытия ещё не достигнут. `Load`
//! получает `Option<&[u8]>`: отсутствие файла, неверный header/version дают
//! исходный `false`; ignored short `fread` после заголовка становится локальной
//! typed-границей. Старый invalid-header путь не закрывал `FILE`; RAII не
//! сохраняет ненаблюдаемую утечку descriptor-а.
//!
//! Region byte-array намеренно не round-trip симметричен: serializer после
//! base пишет region type, width/height, country/notify, cells и только
//! четырёхбайтовый `lState` каждого switch; decoder дополнительно читает
//! resource ID и IEEE-754 scale перед размерами, а switches — целиком по
//! `0x14`. Эта точная разница подтверждена EXE и сохранена, не исправлена.
//! `RegionCell` хранит исходный DWORD byte-exact: block занимает биты `0..2`,
//! security — `3..5`, city-war marker — `6..7`. `SetBlock` меняет только три
//! младших бита 16-bit word; `RegionSecurity` сохраняет все 3-bit значения,
//! включая доказанные `SAFE=2` и `CITYWAR=3`. `Vec` заменяет raw allocation,
//! не меняя row-major layout. Если размеры допускают координату, но storage
//! отсутствует/короче либо signed index переполняется, safe Rust возвращает
//! локальный `BLOCKED_MISSING_FACT`, а не воспроизводит pointer UB и не выдаёт
//! клетку за safe. Единственная доказанная null-pointer ветка security при
//! нулевом индексе сохраняет `SAFE`; `GetBlock/SetBlock` такого guard-а не
//! имеют. Достигнутый virtual caller в `CPlayer::OnDied` остаётся
//! границей самостоятельной death/PK механики. Отдельные тела STL,
//! `Catch/Unwind`, STL vector internals и deleting-thunks сняты общей
//! технической классификацией после переноса их ownership/codec effects;
//! посторонний domain destructor `CPlayerList::tagPropertiesUpgrade` не
//! затрагивался.
//! Random-position сохраняет нормализацию/расширение прямоугольника, ровно
//! 1000 random-попыток, затем x-major linear scan и финальную random-позицию с
//! `false`, когда проходимой клетки нет во всём регионе. Исторический RNG
//! остаётся явным context-owner-ом; block/switch проверяются через тот же
//! byte-exact cell storage. Monster, loot и player death/PK owners остаются RAW.

use super::baseobject::{BaseObjectDecodeError, CBaseObject};

const REGION_RESOURCE_HEADER: &[u8; 7] = b"CLS-RGN";
const REGION_RESOURCE_VERSION: i32 = 1;
const REGION_SWITCH_SIZE: usize = 0x14;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionCell {
    pub(crate) bytes: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionSwitch {
    bytes: [u8; REGION_SWITCH_SIZE],
}

impl RegionSwitch {
    pub(crate) fn state(self) -> i32 {
        i32::from_le_bytes(
            self.bytes[..4]
                .try_into()
                .expect("switch state занимает четыре байта"),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionStorageBlock {
    InvalidDimensions { width: i32, height: i32 },
    CellStorageMismatch { expected: usize, available: usize },
    UninitializedField { field: &'static str },
    TooManySwitches { count: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionDecodeError {
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
pub(crate) struct RegionResourceWrite {
    pub(crate) path: Vec<u8>,
    pub(crate) bytes: Vec<u8>,
}

impl RegionCell {
    pub(crate) fn block(self) -> u8 {
        self.bytes[0] & 0x07
    }

    pub(crate) fn set_block(&mut self, block: u8) {
        self.bytes[0] = (self.bytes[0] & 0xF8) | (block & 0x07);
    }

    pub(crate) fn security(self) -> RegionSecurity {
        RegionSecurity((self.bytes[0] >> 3) & 0x07)
    }

    pub(crate) fn city_war_marker(self) -> u8 {
        (self.bytes[0] >> 6) & 0x03
    }

    pub(crate) fn switch_id(self) -> u16 {
        u16::from_le_bytes([self.bytes[2], self.bytes[3]])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionSecurity(u8);

impl RegionSecurity {
    pub(crate) const FREE: Self = Self(0);
    pub(crate) const FIGHT: Self = Self(1);
    pub(crate) const SAFE: Self = Self(2);
    pub(crate) const CITY_WAR: Self = Self(3);

    pub(crate) fn value(self) -> u8 {
        self.0
    }
}

/// BLOCKED_MISSING_FACT: оригинал вычисляет raw pointer wrapping `imul/add`;
/// реакция на переполнение либо storage, не соответствующий width/height, не
/// доказана и не получает придуманного safe-результата.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionCellAccessBlock {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) available_cells: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionReturnPoint {
    pub(crate) region_id: i32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
    pub(crate) direction: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegionRandomPosition {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) found: bool,
}

pub(crate) trait RegionRandomContext {
    /// Выполняет исходный `random(bound)`. Включая реакцию на неположительный
    /// bound, контракт принадлежит RNG-owner-у, а не spatial family.
    fn random_below(&mut self, bound: i32) -> i32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CRegion {
    base_object: CBaseObject,
    file_name: Vec<u8>,
    region_type: i32,
    resource_id: i32,
    exp_scale_bits: u32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    country: Option<u8>,
    notify: Option<i32>,
    last_notify_hurt_time: u32,
    last_notify_kill_time: u32,
    pub(crate) cells: Vec<RegionCell>,
    switches: Vec<RegionSwitch>,
}

impl Default for CRegion {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl CRegion {
    pub(crate) const fn with_constructor_defaults() -> Self {
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

    pub(crate) const fn get_id(&self) -> i32 {
        self.base_object.get_id()
    }

    pub(crate) const fn set_id(&mut self, id: i32) {
        self.base_object.set_id(id);
    }

    pub(crate) fn get_name(&self) -> &[u8] {
        self.base_object.get_name()
    }

    pub(crate) const fn country(&self) -> Option<u8> {
        self.country
    }

    pub(crate) const fn notify_interval_ms(&self) -> Option<i32> {
        self.notify
    }

    pub(crate) const fn last_notify_kill_time_ms(&self) -> u32 {
        self.last_notify_kill_time
    }

    pub(crate) const fn set_last_notify_kill_time_ms(&mut self, value: u32) {
        self.last_notify_kill_time = value;
    }

    pub(crate) fn file_name(&self) -> &[u8] {
        &self.file_name
    }

    pub(crate) const fn region_type(&self) -> i32 {
        self.region_type
    }

    pub(crate) const fn resource_id(&self) -> i32 {
        self.resource_id
    }

    pub(crate) const fn exp_scale_bits(&self) -> u32 {
        self.exp_scale_bits
    }

    pub(crate) fn exp_scale(&self) -> f32 {
        f32::from_bits(self.exp_scale_bits)
    }

    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.base_object.set_name(name);
    }

    /// Выполняет исходный virtual `New`: очищает прежние switches/cells,
    /// создаёт zero-filled cell storage и возвращает `1`.
    pub(crate) fn new_region(&mut self) -> Result<i32, RegionStorageBlock> {
        self.recreate_storage()?;
        Ok(1)
    }

    /// Возвращает switch по исходному 1-based ID либо старый `nullptr`.
    pub(crate) fn get_switch(&self, id: i32) -> Option<&RegionSwitch> {
        let index = usize::try_from(id.checked_sub(1)?).ok()?;
        self.switches.get(index)
    }

    /// Выполняет coordinate-overload: cell `lSwitch` также является 1-based ID.
    pub(crate) fn get_switch_at(
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
    pub(crate) fn add_to_byte_array(
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
        destination.extend_from_slice(&self.region_type.to_le_bytes());
        destination.extend_from_slice(&self.width.to_le_bytes());
        destination.extend_from_slice(&self.height.to_le_bytes());
        destination.push(country);
        destination.extend_from_slice(&notify.to_le_bytes());
        for cell in &self.cells[..cell_count] {
            destination.extend_from_slice(&cell.bytes);
        }
        destination.extend_from_slice(&switch_count.to_le_bytes());
        for switch in &self.switches {
            destination.extend_from_slice(&switch.bytes[..4]);
        }
        Ok(true)
    }

    /// Читает точный GameServer region-wire, включая его асимметрию с serializer-ом.
    pub(crate) fn decord_from_byte_array(
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
    pub(crate) fn save_resource(&mut self) -> Result<RegionResourceWrite, RegionStorageBlock> {
        let path = self.resource_path();
        self.file_name.clone_from(&path);
        let cell_count = self.validated_cell_count()?;
        let switch_count = i32::try_from(self.switches.len()).map_err(|_| {
            RegionStorageBlock::TooManySwitches {
                count: self.switches.len(),
            }
        })?;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(REGION_RESOURCE_HEADER);
        bytes.extend_from_slice(&REGION_RESOURCE_VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.region_type.to_le_bytes());
        bytes.extend_from_slice(&self.width.to_le_bytes());
        bytes.extend_from_slice(&self.height.to_le_bytes());
        for cell in &self.cells[..cell_count] {
            bytes.extend_from_slice(&cell.bytes);
        }
        bytes.extend_from_slice(&switch_count.to_le_bytes());
        for switch in &self.switches {
            bytes.extend_from_slice(&switch.bytes);
        }
        Ok(RegionResourceWrite { path, bytes })
    }

    /// Читает содержимое уже открытого `regions/{ID}.rgn`; `None` сохраняет
    /// исходный open-failure `false`.
    pub(crate) fn load_resource(
        &mut self,
        source: Option<&[u8]>,
    ) -> Result<bool, RegionDecodeError> {
        let Some(source) = source else {
            return Ok(false);
        };
        if source.get(..REGION_RESOURCE_HEADER.len()) != Some(REGION_RESOURCE_HEADER) {
            return Ok(false);
        }
        let mut cursor = REGION_RESOURCE_HEADER.len();
        let Some(version_bytes) = source.get(cursor..cursor + 4) else {
            return Ok(false);
        };
        cursor += 4;
        if i32::from_le_bytes(
            version_bytes
                .try_into()
                .expect("version занимает четыре байта"),
        ) != REGION_RESOURCE_VERSION
        {
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

    pub(crate) fn get_return_point(&self) -> RegionReturnPoint {
        RegionReturnPoint::default()
    }

    pub(crate) fn get_cell(
        &self,
        x: i32,
        y: i32,
    ) -> Result<Option<&RegionCell>, RegionCellAccessBlock> {
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

    pub(crate) fn set_block(
        &mut self,
        x: i32,
        y: i32,
        block: u8,
    ) -> Result<(), RegionCellAccessBlock> {
        let Some(index) = self.cell_index(x, y)? else {
            return Ok(());
        };
        let unavailable = self.block(x, y);
        let cell = self.cells.get_mut(index).ok_or(unavailable)?;
        cell.set_block(block);
        Ok(())
    }

    pub(crate) fn get_block(&self, x: i32, y: i32) -> Result<u8, RegionCellAccessBlock> {
        let Some(index) = self.cell_index(x, y)? else {
            return Ok(2);
        };
        self.cells
            .get(index)
            .copied()
            .map(RegionCell::block)
            .ok_or_else(|| self.block(x, y))
    }

    pub(crate) fn get_security(
        &self,
        x: i32,
        y: i32,
    ) -> Result<RegionSecurity, RegionCellAccessBlock> {
        Ok(self
            .get_cell(x, y)?
            .map_or(RegionSecurity::SAFE, |cell| cell.security()))
    }

    pub(crate) fn get_random_pos<Context: RegionRandomContext>(
        &self,
        context: &mut Context,
    ) -> Result<RegionRandomPosition, RegionCellAccessBlock> {
        self.get_random_pos_in_range(0, 0, self.width, self.height, context)
    }

    pub(crate) fn get_random_pos_in_range<Context: RegionRandomContext>(
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
    Ok(i32::from_le_bytes(
        read_region_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("проверены четыре байта"),
    ))
}

fn read_region_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, RegionDecodeError> {
    Ok(u32::from_le_bytes(
        read_region_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("проверены четыре байта"),
    ))
}

fn read_region_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, RegionDecodeError> {
    Ok(read_region_bytes(source, cursor, 1, field)?[0])
}

fn read_region_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    needed: usize,
    field: &'static str,
) -> Result<&'a [u8], RegionDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(needed) else {
        return Err(RegionDecodeError::UnexpectedEnd {
            field,
            offset,
            needed,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        // BLOCKED_MISSING_FACT: старые fread/pointer helpers игнорировали
        // short-read и не получали длину byte-array.
        return Err(RegionDecodeError::UnexpectedEnd {
            field,
            offset,
            needed,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.cpp

// ============================================================================
// FUNCTION: CRegion::GetCell
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.h:262
// RVA: 0x0002AC10
//
// Реализовано выше в связной region security chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: CRegion::SetBlock
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.h:273
// RVA: 0x0002AC50
//
// Реализовано выше в связной region security chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: CRegion::GetBlock
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.h:274
// RVA: 0x0007BCC0
//
// Реализовано выше в связной region security chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: CRegion::GetSecurity
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.h:266
// RVA: 0x000854F0
//
// Реализовано выше в связной region security chain; VERIFIED_DISASSEMBLY.

// ============================================================================
// FUNCTION: CRegion::GetReturnPoint
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.cpp:366
// RVA: 0x000F0280
//
// Реализовано выше: все шесть результатов безусловно обнуляются.

// ============================================================================
// FUNCTION: CRegion::GetRandomPosInRange
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.cpp:199
// RVA: 0x000F02C0
//
// Реализовано выше; `lSwitch` word `+2` и точный порядок random/scan/expand
// подтверждены дизассемблированием RVA.

// ============================================================================
// FUNCTION: CRegion::GetRandomPos
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.cpp:189
// RVA: 0x000F04D0
//
// Реализовано выше прямым делегированием full-region range.

// IMPLEMENTED: `CRegion::AddToByteArray` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CRegion::GetSwitch` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CRegion::GetSwitch` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CRegion::New` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CRegion::~CRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.cpp:24
// RVA: 0x000F0750
// ADDRESS: 004f0750
// PROTOTYPE: void __thiscall ~CRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CRegion::Save` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CRegion::CRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.cpp:11
// RVA: 0x000F0D90
// ADDRESS: 004f0d90
// PROTOTYPE: undefined __thiscall CRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CRegion::Load` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CRegion::DecordFromByteArray` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CPlayerList::tagPropertiesUpgrade::~tagPropertiesUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\region.cpp
// RVA: 0x001CF700
// ADDRESS: 005cf700
// PROTOTYPE: void __thiscall ~tagPropertiesUpgrade(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
