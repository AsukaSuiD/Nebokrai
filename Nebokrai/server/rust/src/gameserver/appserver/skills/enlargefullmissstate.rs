//! Каноническая достигнутая часть `CEnlargeFullMissState`.
//!
//! Для игрока состояние `603` прибавляет младшие 16 бит знакового параметра
//! к `full_miss` точным WORD-сложением с переполнением. Собственного
//! визуального сообщения и таймера нет. DB-запись состоит ровно из
//! little-endian ID и знаковой 32-битной прибавки.
//! Constructor RVA `0x001E2080` задаёт ID `0x25B` и нулевой gain; `Default`
//! сохраняет этот контракт без временного base-state/SEH noise.

//! End +0x1C таблицы 0x0065F044 →0x005ECFC0→CState::End0x005DBCE0:
//! ended=1, затем GetUser +0x14 и RemoveState при разрешённом user, без visual.
//! Begin +0x08 0x00601290 передаёт оба аргумента в CState::Begin.
//! StartAllStates0x004CE050 вызывает Begin(0, holder): такой DB-экземпляр
//! не получает user=holder. Общий base End сохраняет эту привязку отдельно
//! от payload и не заменяет отсутствующего user держателем состояния.
//! Runtime state Begin0x00516830 получает одинаковый EBP в обоих аргументах.

use super::enlargefullmiss::ENLARGE_FULL_MISS_SKILL_ID;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_base_applied_state, resolve_state_move_shape};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub(crate) const ENLARGE_FULL_MISS_STATE_BYTES: usize = 8;

pub(crate) fn end_enlarge_full_miss_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnlargeFullMissState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, ENLARGE_FULL_MISS_STATE_BYTES)
}


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
