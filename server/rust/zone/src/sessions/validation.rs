//! Состояние проверок игрового входа по player ID.
//!
//! Источник: GameServer/gameserver.exe + GameServer.pdb, пара из `sequence.rs`.
//! `CGame::AppendSequenceMap` VA 0x00409340 удаляет прежнего владельца при
//! повторном ID и возвращает отказ; `AppendValidateTime` VA 0x004093D0
//! добавляет запись без замены. Формат последовательности описан в `sequence.rs`.

use std::collections::BTreeMap;

use super::sequence::{CSequenceRegistry, CSequenceString};
use super::{SequenceRegistryInitializationError, SequenceSerializeError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequencePreparationError {
    DuplicateSequenceOwner { player_id: i32 },
    Sequence(SequenceSerializeError),
}

#[derive(Debug, Eq, PartialEq)]
pub struct PreparedSequence {
    pub payload: Vec<u8>,
    pub position: i32,
    pub elements: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoginValidationRelease {
    pub sequences: usize,
    pub validate_times: usize,
    pub registry_elements: usize,
}

#[derive(Default)]
pub struct LoginValidationState {
    registry: CSequenceRegistry,
    sequences: BTreeMap<i32, CSequenceString>,
    validate_times: BTreeMap<i32, bool>,
}

impl LoginValidationState {
    pub fn initialize_registry(
        &mut self,
        count: u32,
        next_random: impl FnMut() -> u32,
    ) -> Result<(), SequenceRegistryInitializationError> {
        self.registry.initialize(count, next_random)
    }

    pub fn registry_len(&self) -> usize {
        self.registry.len()
    }

    pub fn has_sequences(&self) -> bool {
        !self.registry.is_empty()
    }

    pub fn append_validate_time(&mut self, player_id: i32, value: bool) {
        self.validate_times.entry(player_id).or_insert(value);
    }

    pub fn prepare_sequence(
        &mut self,
        player_id: i32,
        next_random: impl FnMut() -> u32,
    ) -> Result<PreparedSequence, SequencePreparationError> {
        if self.sequences.remove(&player_id).is_some() {
            return Err(SequencePreparationError::DuplicateSequenceOwner { player_id });
        }
        let mut sequence = CSequenceString::new();
        let payload = sequence
            .serialize(&self.registry, next_random)
            .map_err(SequencePreparationError::Sequence)?;
        let position = sequence.position();
        let elements = self.registry.len();
        self.sequences.insert(player_id, sequence);
        Ok(PreparedSequence {
            payload,
            position,
            elements,
        })
    }

    pub fn remove_player(&mut self, player_id: i32) {
        self.sequences.remove(&player_id);
        self.validate_times.remove(&player_id);
    }

    pub fn release(&mut self) -> LoginValidationRelease {
        let released = LoginValidationRelease {
            sequences: self.sequences.len(),
            validate_times: self.validate_times.len(),
            registry_elements: self.registry.len(),
        };
        self.sequences.clear();
        self.validate_times.clear();
        self.registry.clear();
        released
    }
}
