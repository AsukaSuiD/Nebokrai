//! Sequence-validation owner GameServer из `message/sequencestring.cpp`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает constructor,
//! `Initialize` и `Serialize`: registry append-ится `count` значениями прямого
//! MSVCRT `rand`, ноль заменяется единицей, а новый player-owner выбирает
//! стартовый индекс через `rand % len`. Wire содержит signed start position,
//! signed count и затем все `u32` в little-endian порядке.
//!
//! Общий CRT RNG принадлежит `CGame`; этот owner получает только callback
//! следующего 15-битного значения, чтобы не создавать второй поток случайных
//! чисел. `Vec` заменяет process-static `std::vector`. Advance/check validation
//! после выдачи строки остаются границей `logmessage/player` и не подменяются
//! догадкой из менее доверенного C++-донора.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub(crate) struct SequenceRegistryInitializationError {
    source: TryReserveError,
}

impl fmt::Display for SequenceRegistryInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("не удалось зарезервировать Game sequence registry")
    }
}

impl Error for SequenceRegistryInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SequenceSerializeError {
    CountOutsideLegacyRange,
}

impl fmt::Display for SequenceSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutsideLegacyRange => {
                formatter.write_str("число Game sequence elements не представимо Windows long")
            }
        }
    }
}

impl Error for SequenceSerializeError {}

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
        payload.extend_from_slice(&position.to_le_bytes());
        payload.extend_from_slice(&count.to_le_bytes());
        for element in &registry.elements {
            payload.extend_from_slice(&element.to_le_bytes());
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
