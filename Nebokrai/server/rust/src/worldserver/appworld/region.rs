//! Владелец региона исторического `WorldServer`.
//!
//! Owner `CRegion`; constructor, `New`, resource `Load` и region-wire
//! `AddToByteArray` имеют статус `IMPLEMENTED`, а существенные offsets и
//! возвраты сверены как `VERIFIED_DISASSEMBLY`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\region.h` и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\region.cpp:11`.
//!
//! Существенные RVA: constructor `0x000D7460`, `New` `0x000D7000`, `Load`
//! `0x000D7570`, `GetRandomPosInRange` `0x000D6AA0`, serializer
//! `0x000D6CB0`. Exact EXE подтверждает старые поля:
//! `m_lRegionType +0x6C`, `m_lResourceID +0x70`, `m_fExpScale +0x74`,
//! `m_lWidth/+0x78`, `m_lHeight/+0x7C`, `m_btCountry +0x80`, `m_lNotify +0x84`,
//! cell-owner `+0x88` и `tagSwitch` vector `+0x8C`; Rust layout старый ABI не
//! воспроизводит. Constructor задаёт type `200`, region/resource/size `0`,
//! scale `1.0`, пустые cells/switches, но не назначает country и notify — они
//! остаются `Option` до применения записи `regionlist.ini`.
//!
//! `Load` читает `regions/{ID}.rgn`: `CLS-RGN`, signed version `1`, region type,
//! width/height, ровно `width*height*4` cell-байтов, signed switch-count и по
//! `0x14` байт на switch. Все 872 loose fixtures имеют эту форму, version `1`,
//! type `0`, положительные размеры и точный хвост; SHA-256 `10000.rgn` —
//! `17861A76421D89925BF7D9EFA8257AD6C013EDE525A9487276DF3D84B6B21837`.
//! `New` заменён владеющими `Vec`: он удалял прежние cells/switches, создавал
//! zero-filled cell block и возвращал `1`. Serializer сохраняет literal order и
//! четыре IEEE-754 байта scale; независимый GameServer decoder RVA `0x000F1070`
//! читает их именно как `float`, исправляя ошибочный cast в raw-псевдокоде.
//! `GetRandomPosInRange` сохраняет сначала 1000 случайных попыток, затем scan
//! X-снаружи/Y-внутри и расширение прямоугольника на 10 клеток с каждой
//! стороны. `VERIFIED_DISASSEMBLY` по `0x004D6BA3/0x004D6C14` подтверждает, что
//! клетка закрыта по маске `(byte0 & 7) != 0`, а не по всему первому байту;
//! `0x004D6BA8/0x004D6C19` отдельно проверяет нулевой `u16` по offset `+2`.
//! Process-global `random(long)` передаётся узкой `FnMut(i32) -> i32` границей,
//! поэтому owner сохраняет число, порядок и bounds вызовов без собственной RNG.
//! Неинициализированные country/notify и небезопасные отрицательные/overflow
//! размеры или координатная арифметика останавливают только safe-границу с
//! локальным `BLOCKED_MISSING_FACT`. CRT/STL allocation и cleanup-noise выражены
//! владением Rust и отдельно не восстанавливаются.

use super::baseobject::CBaseObject;

const REGION_RESOURCE_HEADER: &[u8; 7] = b"CLS-RGN";
const REGION_RESOURCE_VERSION: i32 = 1;
const REGION_CELL_SIZE: usize = 4;
const REGION_SWITCH_SIZE: usize = 0x14;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionLoadError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    InvalidHeader,
    UnsupportedVersion(i32),
    InvalidDimensions {
        width: i32,
        height: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionSerializationBlock {
    UninitializedField { field: &'static str },
    TooManySwitches { count: usize },
}

/// Результат старого поиска позиции, включая его наблюдаемый `bool`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionRandomPosition {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) found_walkable: bool,
}

/// Локальная safe-граница арифметики и cell-storage старого поиска позиции.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionRandomPositionBlock {
    InvalidRegionDimensions { width: i32, height: i32 },
    CoordinateOverflow { operation: &'static str },
    MissingCell { x: i32, y: i32, index: usize },
}

/// Достигнутая base-часть исходного `CRegion`.
pub(crate) struct CRegion {
    base_object: CBaseObject,
    file_name: Vec<u8>,
    region_type: i32,
    resource_id: i32,
    exp_scale: f32,
    width: i32,
    height: i32,
    country: Option<u8>,
    notify: Option<i32>,
    cells: Vec<[u8; REGION_CELL_SIZE]>,
    switches: Vec<[u8; REGION_SWITCH_SIZE]>,
}

impl CRegion {
    /// Создаёт полный доказанный constructor-state `CRegion`.
    pub(crate) const fn with_constructor_base_and_type() -> Self {
        let mut base_object = CBaseObject::with_reached_constructor_defaults();
        base_object.set_type(200);
        Self {
            base_object,
            file_name: Vec::new(),
            region_type: 0,
            resource_id: 0,
            exp_scale: 1.0,
            width: 0,
            height: 0,
            country: None,
            notify: None,
            cells: Vec::new(),
            switches: Vec::new(),
        }
    }

    /// Возвращает унаследованный object type без дополнительных эффектов.
    pub(crate) const fn get_type(&self) -> i32 {
        self.base_object.get_type()
    }

    /// Возвращает унаследованный signed object ID.
    pub(crate) const fn get_id(&self) -> i32 {
        self.base_object.get_id()
    }

    /// Присваивает унаследованный signed object ID.
    pub(crate) const fn set_id(&mut self, id: i32) {
        self.base_object.set_id(id);
    }

    /// Заимствует byte-exact имя из единственного base-подобъекта.
    pub(crate) fn get_name(&self) -> &[u8] {
        self.base_object.get_name()
    }

    /// Присваивает унаследованное byte-exact имя до первого NUL.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.base_object.set_name(name);
    }

    /// Возвращает reached country byte без подстановки constructor-неизвестного значения.
    pub(crate) const fn country(&self) -> Option<u8> {
        self.country
    }

    /// Присваивает достигнутый country byte унаследованного `CRegion`.
    pub(crate) const fn set_country(&mut self, country: u8) {
        self.country = Some(country);
    }

    /// Дописывает только унаследованный `CBaseObject` для отдельного proxy-wire.
    pub(crate) fn add_base_object_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> bool {
        self.base_object
            .add_to_byte_array(destination, include_child)
    }

    /// Выполняет исходный virtual `New`: пересоздаёт cells и очищает switches.
    pub(crate) fn new_region(&mut self) -> Result<i32, RegionLoadError> {
        self.recreate_cells()?;
        Ok(1)
    }

    /// Применяет четыре поля, которые `LoadRegionList` назначает перед virtual `Load`.
    pub(crate) const fn set_region_list_fields(
        &mut self,
        resource_id: u32,
        exp_scale: f32,
        country: u8,
        notify: i32,
    ) {
        self.resource_id = resource_id as i32;
        self.exp_scale = exp_scale;
        self.country = Some(country);
        self.notify = Some(notify);
    }

    /// Загружает точное содержимое уже открытого `regions/{ID}.rgn`.
    pub(crate) fn load_from_resource(
        &mut self,
        path: &[u8],
        source: &[u8],
    ) -> Result<bool, RegionLoadError> {
        if source.get(..REGION_RESOURCE_HEADER.len()) != Some(REGION_RESOURCE_HEADER) {
            return Err(RegionLoadError::InvalidHeader);
        }
        let mut cursor = REGION_RESOURCE_HEADER.len();
        let version = read_region_i32(source, &mut cursor, "version")?;
        if version != REGION_RESOURCE_VERSION {
            return Err(RegionLoadError::UnsupportedVersion(version));
        }

        self.file_name.clear();
        self.file_name.extend_from_slice(path);
        self.region_type = read_region_i32(source, &mut cursor, "m_lRegionType")?;
        self.width = read_region_i32(source, &mut cursor, "m_lWidth")?;
        self.height = read_region_i32(source, &mut cursor, "m_lHeight")?;
        self.recreate_cells()?;

        for cell in &mut self.cells {
            cell.copy_from_slice(read_region_bytes(
                source,
                &mut cursor,
                REGION_CELL_SIZE,
                "m_pCell",
            )?);
        }

        let switch_count = read_region_i32(source, &mut cursor, "m_vectorSwitch.size")?;
        if switch_count > 0 {
            self.switches.reserve(switch_count as usize);
            for _ in 0..switch_count {
                let mut switch = [0; REGION_SWITCH_SIZE];
                switch.copy_from_slice(read_region_bytes(
                    source,
                    &mut cursor,
                    REGION_SWITCH_SIZE,
                    "m_vectorSwitch[]",
                )?);
                self.switches.push(switch);
            }
        }
        Ok(true)
    }

    /// Дописывает полный region-base wire после унаследованного `CBaseObject`.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, RegionSerializationBlock> {
        let country = self
            .country
            .ok_or(RegionSerializationBlock::UninitializedField {
                field: "m_btCountry",
            })?;
        let notify = self
            .notify
            .ok_or(RegionSerializationBlock::UninitializedField { field: "m_lNotify" })?;
        let switch_count = i32::try_from(self.switches.len()).map_err(|_| {
            RegionSerializationBlock::TooManySwitches {
                count: self.switches.len(),
            }
        })?;

        let _ = self
            .base_object
            .add_to_byte_array(destination, include_child);
        destination.extend_from_slice(&self.region_type.to_le_bytes());
        destination.extend_from_slice(&self.resource_id.to_le_bytes());
        destination.extend_from_slice(&self.exp_scale.to_bits().to_le_bytes());
        destination.extend_from_slice(&self.width.to_le_bytes());
        destination.extend_from_slice(&self.height.to_le_bytes());
        destination.push(country);
        destination.extend_from_slice(&notify.to_le_bytes());
        for cell in &self.cells {
            destination.extend_from_slice(cell);
        }
        destination.extend_from_slice(&switch_count.to_le_bytes());
        for switch in &self.switches {
            destination.extend_from_slice(switch);
        }
        Ok(true)
    }

    /// Ищет проходимую клетку в точном порядке исходного owner-а.
    ///
    /// `random` обязан иметь контракт legacy `random(long)`: для
    /// неотрицательного bound возвращать значение, которое можно безопасно
    /// прибавить к началу диапазона. Bound `0` допустим и даёт `0`.
    pub(crate) fn get_random_pos_in_range<R>(
        &self,
        mut left: i32,
        mut top: i32,
        mut span_x: i32,
        mut span_y: i32,
        mut random: R,
    ) -> Result<RegionRandomPosition, RegionRandomPositionBlock>
    where
        R: FnMut(i32) -> i32,
    {
        if self.width < 0 || self.height < 0 {
            // BLOCKED_MISSING_FACT: RVA 0x000D6AA0 передавал отрицательный
            // dimension в `random(long)` и signed-арифметику. Достижимость и
            // реакция старого helper-а для такого region-state не доказаны.
            return Err(RegionRandomPositionBlock::InvalidRegionDimensions {
                width: self.width,
                height: self.height,
            });
        }

        loop {
            let requested_bounds =
                if left < self.width && top < self.height && span_x >= 0 && span_y >= 0 {
                    let right = checked_region_add(left, span_x, "left + span_x")?;
                    if right >= 0 {
                        let bottom = checked_region_add(top, span_y, "top + span_y")?;
                        (bottom >= 0).then_some((right, bottom))
                    } else {
                        None
                    }
                } else {
                    None
                };

            if let Some((right, bottom)) = requested_bounds {
                if left < 0 {
                    left = 0;
                    span_x = right;
                }
                if top < 0 {
                    top = 0;
                    span_y = bottom;
                }
            } else {
                left = 0;
                top = 0;
                span_x = self.width;
                span_y = self.height;
            }

            let right = checked_region_add(left, span_x, "normalized left + span_x")?;
            if self.width < right {
                span_x = checked_region_sub(self.width, left, "region width - left")?;
            }
            let bottom = checked_region_add(top, span_y, "normalized top + span_y")?;
            if self.height < bottom {
                span_y = checked_region_sub(self.height, top, "region height - top")?;
            }

            for _ in 0..1000 {
                let x = checked_region_add(random(span_x), left, "random X + left")?;
                let y = checked_region_add(random(span_y), top, "random Y + top")?;
                if self.is_walkable_cell(x, y)? {
                    return Ok(RegionRandomPosition {
                        x,
                        y,
                        found_walkable: true,
                    });
                }
            }

            let scan_right = checked_region_add(left, span_x, "scan left + span_x")?;
            let scan_bottom = checked_region_add(top, span_y, "scan top + span_y")?;
            let mut x = left;
            while x < scan_right {
                let mut y = top;
                while y < scan_bottom {
                    if self.is_walkable_cell(x, y)? {
                        return Ok(RegionRandomPosition {
                            x,
                            y,
                            found_walkable: true,
                        });
                    }
                    y = checked_region_add(y, 1, "scan Y + 1")?;
                }
                x = checked_region_add(x, 1, "scan X + 1")?;
            }

            if left < 1 && top < 1 && self.width <= span_x && self.height <= span_y {
                return Ok(RegionRandomPosition {
                    x: random(self.width),
                    y: random(self.height),
                    found_walkable: false,
                });
            }

            // BLOCKED_MISSING_FACT: исходные signed операции при overflow дают
            // неопределённое C++-поведение; безопасный owner не назначает ему
            // wrap либо fail-closed результат без доказательства достижимости.
            left = checked_region_sub(left, 10, "left - 10")?;
            top = checked_region_sub(top, 10, "top - 10")?;
            span_x = checked_region_add(span_x, 20, "span_x + 20")?;
            span_y = checked_region_add(span_y, 20, "span_y + 20")?;
        }
    }

    fn is_walkable_cell(&self, x: i32, y: i32) -> Result<bool, RegionRandomPositionBlock> {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return Ok(false);
        }
        let width = usize::try_from(self.width).expect("проверена неотрицательная ширина");
        let x = usize::try_from(x).expect("проверена неотрицательная X");
        let y = usize::try_from(y).expect("проверена неотрицательная Y");
        let index = y
            .checked_mul(width)
            .and_then(|row| row.checked_add(x))
            .ok_or(RegionRandomPositionBlock::CoordinateOverflow {
                operation: "y * region width + x",
            })?;
        let cell = self
            .cells
            .get(index)
            .ok_or(RegionRandomPositionBlock::MissingCell {
                x: x as i32,
                y: y as i32,
                index,
            })?;
        Ok((cell[0] & 0x07) == 0 && u16::from_le_bytes([cell[2], cell[3]]) == 0)
    }

    fn recreate_cells(&mut self) -> Result<(), RegionLoadError> {
        self.switches.clear();
        let (Ok(width), Ok(height)) = (usize::try_from(self.width), usize::try_from(self.height))
        else {
            // BLOCKED_MISSING_FACT: RVA 0x000D7000 умножал signed dimensions
            // с wrap и передавал результат allocator-у. Реакция CRT на такой
            // размер не задаёт безопасное серверное поведение.
            return Err(RegionLoadError::InvalidDimensions {
                width: self.width,
                height: self.height,
            });
        };
        let Some(cell_count) = width.checked_mul(height) else {
            return Err(RegionLoadError::InvalidDimensions {
                width: self.width,
                height: self.height,
            });
        };
        let Some(_) = cell_count.checked_mul(REGION_CELL_SIZE) else {
            return Err(RegionLoadError::InvalidDimensions {
                width: self.width,
                height: self.height,
            });
        };
        self.cells.clear();
        self.cells.resize(cell_count, [0; REGION_CELL_SIZE]);
        Ok(())
    }
}

fn checked_region_add(
    left: i32,
    right: i32,
    operation: &'static str,
) -> Result<i32, RegionRandomPositionBlock> {
    left.checked_add(right)
        .ok_or(RegionRandomPositionBlock::CoordinateOverflow { operation })
}

fn checked_region_sub(
    left: i32,
    right: i32,
    operation: &'static str,
) -> Result<i32, RegionRandomPositionBlock> {
    left.checked_sub(right)
        .ok_or(RegionRandomPositionBlock::CoordinateOverflow { operation })
}

fn read_region_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, RegionLoadError> {
    let bytes = read_region_bytes(source, cursor, 4, field)?;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("ровно четыре байта"),
    ))
}

fn read_region_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    length: usize,
    field: &'static str,
) -> Result<&'a [u8], RegionLoadError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(length) else {
        return Err(RegionLoadError::UnexpectedEnd {
            field,
            offset,
            needed: length,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        // BLOCKED_MISSING_FACT: `CRFile::ReadData` result was ignored. Safe Rust
        // cannot expose the original buffer contents after a short read.
        return Err(RegionLoadError::UnexpectedEnd {
            field,
            offset,
            needed: length,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\region.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\region.h

// ============================================================================
// FUNCTION: CRegion::~CRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\region.cpp:22
// RVA: 0x000D6D90
// ADDRESS: 004d6d90
// PROTOTYPE: void __thiscall ~CRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegion::Save
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\region.cpp:85
// RVA: 0x000D6E70
// ADDRESS: 004d6e70
// PROTOTYPE: int __thiscall Save(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRegion::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\region.cpp:159
// RVA: 0x000D7720
// ADDRESS: 004d7720
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
