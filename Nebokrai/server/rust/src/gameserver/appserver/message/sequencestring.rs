//! Владелец проверки последовательности GameServer из `message/sequencestring.cpp`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает конструктор,
//! `Initialize` и `Serialize`: в реестр добавляется `count` значений прямого
//! MSVCRT `rand`, ноль заменяется единицей, а новый player-owner выбирает
//! стартовый индекс через `rand % len`. Формат содержит знаковые начальную позицию
//! и `count`, затем все `u32` в little-endian порядке.
//!
//! Общий CRT RNG принадлежит `CGame`; этот владелец получает только обратный вызов
//! следующего 15-битного значения, чтобы не создавать второй поток случайных
//! чисел. `Vec` заменяет статический для процесса `std::vector`. Проверка продвижения
//! после выдачи строки остаются границей `logmessage/player` и не подменяются
//! догадкой из менее доверенного C++-донора.

use std::collections::TryReserveError;
use thiserror::Error;

use crate::gameserver::appserver::legacycodec::LegacyWriter;

#[derive(Debug, Error)]
#[error("не удалось зарезервировать Game sequence registry")]
pub(crate) struct SequenceRegistryInitializationError {
    #[source]
    source: TryReserveError,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum SequenceSerializeError {
    #[error("число Game sequence elements не представимо Windows long")]
    CountOutsideLegacyRange,
}

#[derive(Default)]
pub(crate) struct CSequenceRegistry {
    elements: Vec<u32>,
}

impl CSequenceRegistry {
    /// Добавляет новый initialized batch; исходный static vector не очищался.
    pub(crate) fn initialize(
        &mut self,
        count: u32,
        mut next_random: impl FnMut() -> u32,
    ) -> Result<(), SequenceRegistryInitializationError> {
        self.elements
            .try_reserve(count as usize)
            .map_err(|source| SequenceRegistryInitializationError { source })?;
        for _ in 0..count {
            let value = next_random();
            self.elements.push(if value == 0 { 1 } else { value });
        }
        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        self.elements.clear();
    }

    pub(crate) fn len(&self) -> usize {
        self.elements.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

pub(crate) struct CSequenceString {
    position: i32,
    usable: bool,
}

impl CSequenceString {
    pub(crate) const fn new() -> Self {
        Self {
            position: -1,
            usable: false,
        }
    }

    /// Строит точный validation payload и сохраняет выбранную позицию owner-а.
    pub(crate) fn serialize(
        &mut self,
        registry: &CSequenceRegistry,
        mut next_random: impl FnMut() -> u32,
    ) -> Result<Vec<u8>, SequenceSerializeError> {
        let count = i32::try_from(registry.elements.len())
            .map_err(|_| SequenceSerializeError::CountOutsideLegacyRange)?;
        let position = if registry.elements.is_empty() {
            self.position = -1;
            self.usable = false;
            -1
        } else {
            let position = (next_random() as usize) % registry.elements.len();
            self.position = position as i32;
            self.usable = true;
            self.position
        };

        let mut payload = Vec::with_capacity(8 + registry.elements.len() * 4);
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_i32(position);
        writer.write_i32(count);
        for element in &registry.elements {
            writer.write_u32(*element);
        }
        Ok(payload)
    }

    pub(crate) const fn position(&self) -> i32 {
        self.position
    }

    pub(crate) const fn is_usable(&self) -> bool {
        self.usable
    }
}

impl Default for CSequenceString {
    fn default() -> Self {
        Self::new()
    }
}
