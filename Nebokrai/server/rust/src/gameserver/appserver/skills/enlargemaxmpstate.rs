//! Каноническая достигнутая часть `CEnlargeMaxMpState`.
//!
//! Для игрока состояние `602` складывает текущий максимум MP и знаковый
//! параметр как `u32` с переполнением, после чего ограничивает результат
//! значением `i32::MAX`. Собственного визуального сообщения и таймера нет.
//! Исходный owner PDB — `skills/enlargemaxmpstate.cpp/.h`, точная пара
//! GameServer. Constructor RVA `0x001E21D0` задаёт skill ID `0x25A` и нулевой
//! gain; `Default` выражает этот контракт без временного CState/STL noise.
//! Exact persisted-запись общей пары `0x005E23D0/0x00601350` — `ID + i32 gain`.

use super::enlargemaxmp::ENLARGE_MAX_MP_SKILL_ID;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};

pub(crate) const ENLARGE_MAX_MP_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EnlargeMaxMpState {
    gain: i32,
}

impl EnlargeMaxMpState {
    pub(crate) const fn new(gain: i32) -> Self {
        Self { gain }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        ENLARGE_MAX_MP_SKILL_ID
    }
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> { let mut reader = LegacyReader::at(payload, offset)?; if reader.read_u32()? != ENLARGE_MAX_MP_SKILL_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); } Ok(Self::new(reader.read_i32()?)) }
    pub(crate) fn encoded(self) -> [u8; ENLARGE_MAX_MP_STATE_BYTES] { let mut bytes = [0; ENLARGE_MAX_MP_STATE_BYTES]; bytes[..4].copy_from_slice(&ENLARGE_MAX_MP_SKILL_ID.to_le_bytes()); bytes[4..].copy_from_slice(&self.gain.to_le_bytes()); bytes }

    pub(crate) const fn apply(self, value: u32) -> u32 {
        let result = value.wrapping_add(self.gain as u32);
        if result > i32::MAX as u32 {
            i32::MAX as u32
        } else {
            result
        }
    }
}
