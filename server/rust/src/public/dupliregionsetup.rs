//! Список дублирующих регионов исторического Miracle.
//!
//! World `CDupliRegionSetup::Load/AddToByteArray/GetRandomRegion` и Game
//! `DecordFromByteArray` подтверждены точными EXE/PDB обоих компонентов;
//! исходный владелец `public/dupliregionsetup.cpp`.
//!
//! Сериализаторы World/Game подтверждают формат: знаковый 32-битный счётчик и
//! записи по восемь байт в порядке вставки и с младшим байтом первым (`region_id`,
//! `duplicate_region_id`). `Vec` заменяет старый `std::list`, поскольку
//! наблюдаемый контракт требует только порядка и размера. Типизированные поля исключают
//! выравнивание C++, а переполнение счётчика возвращается до изменения буфера.
//! `GetRandomRegion` сначала кладёт исходный знаковый ID во временный вектор,
//! затем дописывает все его дублирующие ID в порядке списка и ровно один раз вызывает
//! общий `random(count)`. `Vec` и переданный вызывающей стороной адаптер RNG заменяют
//! только инфраструктуру STL и глобального состояния процесса. Исходный контракт
//! адаптера: при положительной границе он возвращает индекс в её пределах. Поведение
//! повреждённого ini подтверждено отдельно: исходная машинная реализация вставляла
//! мусор стека после неуспешного форматного чтения, но всё равно сообщала об успешном
//! открытии файла. Rust сохраняет успешный результат и уже прочитанный префикс, но не
//! переносит дефект неинициализированной памяти и не добавляет неполную запись.

use std::collections::TryReserveError;
use thiserror::Error;

use super::readwrite::read_to;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};

/// Точный восьмибайтовый `CDupliRegionSetup::tagDupliRegion`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DupliRegionEntry {
    pub(crate) region_id: i32,
    pub(crate) duplicate_region_id: i32,
}

/// Владелец значений вместо локального для процесса `std::list`.
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
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(count);
        for entry in &self.entries {
            writer.write_i32(entry.region_id);
            writer.write_i32(entry.duplicate_region_id);
        }
        Ok(())
    }
}

fn read_formatted_long<'a>(tokens: &mut impl Iterator<Item = &'a [u8]>) -> Option<i32> {
    std::str::from_utf8(tokens.next()?).ok()?.parse().ok()
}

/// Невозможный в исходном 32-битном `std::list` размер.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("DupliRegionSetup содержит {count} записей вне signed 32-битного диапазона")]
pub(crate) struct DupliRegionSerializeError {
    pub(crate) count: usize,
}

#[derive(Debug, Error)]
pub(crate) enum DupliRegionDecodeError {
    #[error("DupliRegion snapshot обрывается на {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("не удалось выделить DupliRegion snapshot")]
    Allocation(#[source] TryReserveError),
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, DupliRegionDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(map_read_block)?;
    let value = reader.read_i32().map_err(map_read_block)?;
    *cursor = reader.position();
    Ok(value)
}

fn map_read_block(block: LegacyReadBlock) -> DupliRegionDecodeError {
    DupliRegionDecodeError::UnexpectedEnd {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
