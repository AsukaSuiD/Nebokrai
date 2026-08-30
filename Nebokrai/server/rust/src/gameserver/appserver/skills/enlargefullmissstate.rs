//! Каноническая достигнутая часть `CEnlargeFullMissState`.
//!
//! Для игрока состояние `603` прибавляет младшие 16 бит знакового параметра
//! к `full_miss` точным WORD-сложением с переполнением. Собственного
//! визуального сообщения и таймера нет. DB-запись состоит ровно из
//! little-endian ID и знаковой 32-битной прибавки.
//! Constructor RVA `0x001E2080` задаёт ID `0x25B` и нулевой gain; `Default`
//! сохраняет этот контракт без временного base-state/SEH noise.

use super::enlargefullmiss::ENLARGE_FULL_MISS_SKILL_ID;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub(crate) const ENLARGE_FULL_MISS_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EnlargeFullMissState {
    gain: i32,
}

impl EnlargeFullMissState {
    pub(crate) const fn new(gain: i32) -> Self {
        Self { gain }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        ENLARGE_FULL_MISS_SKILL_ID
    }
    pub(crate) const fn apply(self, value: u16) -> u16 {
        value.wrapping_add(self.gain as u16)
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ENLARGE_FULL_MISS_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(reader.read_i32()?))
    }

    pub(crate) fn encoded(self) -> [u8; ENLARGE_FULL_MISS_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(ENLARGE_FULL_MISS_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(ENLARGE_FULL_MISS_SKILL_ID);
        writer.write_i32(self.gain);
        bytes
            .try_into()
            .expect("размер состояния полного уклонения фиксирован")
    }
}
