//! Список дублирующих регионов исторического Miracle.
//!
//! World `CDupliRegionSetup::Load/AddToByteArray/GetRandomRegion` и Game
//! `DecordFromByteArray` подтверждены точными EXE/PDB обоих компонентов;
//! исходный owner `public/dupliregionsetup.cpp`.
//!
//! Оригинал World/Game serializers подтверждают wire: signed 32-битный count и
//! insertion-order records по восемь little-endian bytes (`region_id`,
//! `duplicate_region_id`). `Vec` заменяет старый `std::list`, поскольку
//! наблюдаемый контракт требует только порядка и размера. Typed поля исключают
//! C++ layout/padding, а переполнение count возвращается до изменения buffer-а.
//! `GetRandomRegion` сначала кладёт исходный signed ID во временный vector,
//! затем дописывает все его duplicate ID в list-order и ровно один раз вызывает
//! общий `random(count)`. `Vec` и переданный caller-ом RNG adapter заменяют
//! только STL/process-global plumbing. Контракт adapter-а исходный: при
//! положительном bound он возвращает индекс `0..bound`. Поведение повреждённого
//! ini подтверждено отдельно: оригинал machine вставляла stack-мусор после
//! неуспешного formatted extraction, но всё равно возвращала success открытого
//! файла. Rust сохраняет success и уже прочитанный prefix, но не переносит
//! uninitialized-memory defect и не добавляет неполную запись.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

use super::readwrite::read_to;

/// Точный восьмибайтовый `CDupliRegionSetup::tagDupliRegion`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DupliRegionEntry {
    pub(crate) region_id: i32,
    pub(crate) duplicate_region_id: i32,
}

/// Value-owner вместо process-local `std::list`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CDupliRegionSetup {
    entries: Vec<DupliRegionEntry>,
}

impl CDupliRegionSetup {
    /// Перечитывает оригинал `setup/DupliRegionsSetup.ini` token-формат.
    pub(crate) fn load(&mut self, source: Option<&[u8]>) -> bool {
        self.entries.clear();
        let Some(source) = source else {
            return false;
        };
        let mut tokens = source
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty());
        while read_to(&mut tokens, b"*") {
            let Some(region_id) = read_formatted_long(&mut tokens) else {
                break;
            };
            let Some(duplicate_region_id) = read_formatted_long(&mut tokens) else {
                break;
            };
            self.entries.push(DupliRegionEntry {
                region_id,
                duplicate_region_id,
            });
        }
        true
    }

    /// Декодирует Game startup snapshot: немедленно очищает прежний list,
    /// затем сохраняет каждый полный восьмибайтовый record в wire-order.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), DupliRegionDecodeError> {
        self.entries.clear();
        let count = read_wire_i32(source, cursor)?;
        if count <= 0 {
            return Ok(());
        }
        self.entries
            .try_reserve(count as usize)
            .map_err(DupliRegionDecodeError::Allocation)?;
        for _ in 0..count {
            let region_id = read_wire_i32(source, cursor)?;
            let duplicate_region_id = read_wire_i32(source, cursor)?;
            self.entries.push(DupliRegionEntry {
                region_id,
                duplicate_region_id,
            });
        }
        Ok(())
    }

    pub(crate) fn push(&mut self, entry: DupliRegionEntry) {
        self.entries.push(entry);
    }

    pub(crate) fn entries(&self) -> &[DupliRegionEntry] {
        &self.entries
    }

    /// Выбирает исходный region либо один из его duplicate в точном list-order.
    pub(crate) fn get_random_region(
        &self,
        region_id: i32,
        mut random: impl FnMut(i32) -> i32,
    ) -> i32 {
        let mut candidates = Vec::with_capacity(1 + self.entries.len());
        candidates.push(region_id);
        candidates.extend(
            self.entries
                .iter()
                .filter(|entry| entry.region_id == region_id)
                .map(|entry| entry.duplicate_region_id),
        );
        let bound = i32::try_from(candidates.len())
            .expect("32-битный legacy list не превышает i32::MAX записей");
        let selected = random(bound);
        candidates[usize::try_from(selected)
            .expect("legacy random(count) возвращает неотрицательный индекс")]
    }

    /// Дописывает оригинал `count + insertion-order 8-byte records`.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), DupliRegionSerializeError> {
        let count = i32::try_from(self.entries.len()).map_err(|_| DupliRegionSerializeError {
            count: self.entries.len(),
        })?;
        destination.extend_from_slice(&count.to_le_bytes());
        for entry in &self.entries {
            destination.extend_from_slice(&entry.region_id.to_le_bytes());
            destination.extend_from_slice(&entry.duplicate_region_id.to_le_bytes());
        }
        Ok(())
    }
}

fn read_formatted_long<'a>(tokens: &mut impl Iterator<Item = &'a [u8]>) -> Option<i32> {
    std::str::from_utf8(tokens.next()?).ok()?.parse().ok()
}

/// Невозможный в исходном 32-битном `std::list` размер.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DupliRegionSerializeError {
    pub(crate) count: usize,
}

impl fmt::Display for DupliRegionSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "DupliRegionSetup содержит {} записей вне signed 32-битного диапазона",
            self.count
        )
    }
}

impl Error for DupliRegionSerializeError {}

#[derive(Debug)]
pub(crate) enum DupliRegionDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    Allocation(TryReserveError),
}

impl fmt::Display for DupliRegionDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "DupliRegion snapshot обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::Allocation(_) => formatter.write_str("не удалось выделить DupliRegion snapshot"),
        }
    }
}

impl Error for DupliRegionDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation(source) => Some(source),
            _ => None,
        }
    }
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, DupliRegionDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(4)) else {
        return Err(DupliRegionDecodeError::UnexpectedEnd {
            offset,
            needed: 4,
            available,
        });
    };
    *cursor += 4;
    Ok(i32::from_le_bytes(
        bytes
            .try_into()
            .expect("DupliRegion scalar содержит 4 байта"),
    ))
}
