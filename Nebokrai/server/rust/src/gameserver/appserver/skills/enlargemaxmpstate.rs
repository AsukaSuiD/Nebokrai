//! Каноническая достигнутая часть `CEnlargeMaxMpState`.
//!
//! Для игрока состояние `602` складывает текущий максимум MP и знаковый
//! параметр как `u32` с переполнением, после чего ограничивает результат
//! значением `i32::MAX`. Собственного визуального сообщения и таймера нет.
//! Исходный owner PDB — `skills/enlargemaxmpstate.cpp/.h`, точная пара
//! GameServer. Constructor RVA `0x001E21D0` задаёт skill ID `0x25A` и нулевой
//! gain; `Default` выражает этот контракт без временного CState/STL noise.
//! Exact persisted-запись общей пары `0x005E23D0/0x00601350` — `ID + i32 gain`.

//! End +0x1C таблицы 0x0065F094 →0x005ECFC0→CState::End0x005DBCE0:
//! ended=1, затем GetUser +0x14 и RemoveState при разрешённом user, без visual.
//! Begin +0x08 0x00601290 передаёт оба аргумента в CState::Begin и возвращает 1.
//! Restart Begin(NULL, holder) сохраняет user и timestamp, снимает IsEnded;
//! собственных guards, visual, часов и сброса payload нет.
//! StartAllStates0x004CE050 вызывает Begin(0, holder): такой DB-экземпляр
//! не получает user=holder. Общий base End сохраняет эту привязку отдельно
//! от payload и не заменяет отсутствующего user держателем состояния.
//! Runtime state Begin0x00516AC0 получает одинаковый EBP в обоих аргументах.

use super::enlargemaxmp::ENLARGE_MAX_MP_SKILL_ID;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};

pub(crate) const ENLARGE_MAX_MP_STATE_BYTES: usize = 8;

pub(crate) fn restart_enlarge_max_mp_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnlargeMaxMpState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
}

pub(crate) fn end_enlarge_max_mp_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnlargeMaxMpState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, ENLARGE_MAX_MP_STATE_BYTES)
}


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
