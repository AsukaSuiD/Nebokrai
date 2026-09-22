//! Проверочная последовательность сессии Zone; перенесена из переходного
//! `src/gameserver/appserver/message/sequencestring.rs`.
//!
//! Исходный владелец PDB: `server/gameserver/appserver/message/sequencestring.cpp`.
//! Пара GameServer: EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB SHA-256 `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`.
//! Машинный код конструктора (VA `0x0042A390`), `Initialize` (`0x0042A6D0`)
//! и `Serialize` (`0x0042A3D0`) подтверждает: в реестр добавляется `count`
//! значений прямого MSVCRT `rand`, ноль заменяется единицей, а новый player-owner выбирает
//! стартовый индекс через `rand % len`. Формат содержит два 32-битных слова
//! (индекс с битовым значением `-1` для пустого реестра и count), затем все
//! `u32` в little-endian порядке.
//!
//! Общий CRT RNG принадлежит `CGame`; этот владелец получает только обратный вызов
//! следующего 15-битного значения, чтобы не создавать второй поток случайных
//! чисел. `Vec` заменяет статический для процесса `std::vector`. Проверка продвижения
//! после выдачи строки остаются границей `logmessage/player` и не подменяются
//! догадкой из менее доверенного C++-донора.

use std::collections::TryReserveError;
use thiserror::Error;

use nebokrai_shared::protocol::LegacyWriter;

#[derive(Debug, Error)]
#[error("не удалось зарезервировать Game sequence registry")]
pub struct SequenceRegistryInitializationError {
    #[source]
    source: TryReserveError,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum SequenceSerializeError {
    #[error("число Game sequence elements не представимо Windows long")]
    CountOutsideLegacyRange,
}

#[derive(Default)]
pub struct CSequenceRegistry {
    elements: Vec<u32>,
}

impl CSequenceRegistry {
    /// Добавляет новый initialized batch; исходный static vector не очищался.
    pub fn initialize(
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

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

pub struct CSequenceString {
    position: i32,
    usable: bool,
}

impl CSequenceString {
    pub const fn new() -> Self {
        Self {
            position: -1,
            usable: false,
        }
    }

    /// Строит точный validation payload и сохраняет выбранную позицию owner-а.
    pub fn serialize(
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

    pub const fn position(&self) -> i32 {
        self.position
    }

    pub const fn is_usable(&self) -> bool {
        self.usable
    }
}

impl Default for CSequenceString {
    fn default() -> Self {
        Self::new()
    }
}
