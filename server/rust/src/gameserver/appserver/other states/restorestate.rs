//! Состояния восстановления от расходуемых предметов и интервалы применения.
//! Источник: gameserver.exe + GameServer.pdb, исходные владельцы
//! appserver/player.cpp и appserver/other states/restorehpstate.cpp,
//! restorempstate.cpp. Общие layout, codec и таймер не объединяют экземпляры:
//! HP и MP сохраняют разные ID и варианты, а каждый payload принадлежит
//! единственной записи AppliedStateEntries в исходном порядке m_vStates.
//! Стандартные массивы/целые и заимствования заменяют повторный технический
//! код двух owners; игровые различия HP/MP остаются у их AI.
//!
//! CPlayer::RestoreHp0x00444C80/RestoreMp0x00444D50: clock1 проверяет
//! unsigned last+interval > now; после успеха clock2 меняет только свой last
//! до ctor. Параметрические ctor0x004F8410/0x004F87B0 копируют keep/frequency/
//! gain, оставляя started/count нулевыми и не читая часы. Object Begin
//! HP0x004F8720/MP0x004F8A10 вызывает base0x005DBD70: self/self читает
//! независимый clock3, сохраняет стороны, затем создаёт visual SetRun(1)
//! и обнуляет count. Только после успеха caller добавляет тот же экземпляр;
//! нет instant-ветви для keep=0, замены предыдущего Restore, Update или пакета.
//! Подготовленный Begin ниже не регистрирует payload; общий append сохраняет
//! точные стороны и остаточный visual без повторных часов/Begin.
//! Отказ native allocator не эмулируется.
//!
//! NULL-user restart сохраняет timestamp/user, назначает holder sufferer,
//! создаёт visual SetRun(1), затем сбрасывает count без часов и пакетов.
//! SetRegion обеих vtable0x0065355C/0x006535BC +0x2C = 0x005E3B30
//! меняет только регион sufferer. AI работает с actual sufferer, а payload
//! остаётся у holder: HP принимает CMoveShape, MP требует CPlayer до death-gate.
//! Missing sufferer и MP non-player вызывают End до часов. Живая цель:
//! clock1 → strict frequency*count+started < now → count++ → прибавка с
//! DWORD wrapping и ограничением максимумом → OnChangeStates → clock2 →
//! strict keep+started < now. После callback читается тот же живой ключ.
//! Смерть приостанавливает шаги и истечение. End0x005EEBA0 только вызывает
//! RemoveState(pointer) у actual sufferer, не обновляя visual/state.ended.
//!
//! Serialize0x005F65F0 пишет ID, один вызов dynamic remaining, frequency,
//! gain — четыре DWORD; в отличие от Ex/Undead он НЕ меняет live keep.
//! Client-time0x005F2CD0 читает clock для deadline<=now и при оставшемся
//! сроке второй clock для wrapping вычитания; keep=0 не является отдельным
//! пропуском часов. Additional=0 (0x00601200), без дополнительных часов.
//! Unserialize0x005EEC70 получает собственный clock перед keep/frequency/gain;
//! decoder принимает это уже снятое время, не повторяет часы и не сериализует
//! count/timestamp/стороны. Отдельные интервалы обнуляет constructor игрока;
//! Decode/Clear его состояний не сбрасывают задержку повторного применения.
//! Координатные и typed-target Begin остаются RAW в исходных owners.

use super::restorehpstate::{RESTORE_HP_STATE_ID, RestoreHpState};
use super::restorempstate::{RESTORE_MP_STATE_ID, RestoreMpState};
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual,
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;

pub(crate) const CONSUMABLE_RESTORE_STATE_BYTES: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RestoreStateData<const ID: i32> {
    time_to_keep_ms: u32,
    frequency_ms: u32,
    gain: u32,
    restore_count: u32,
    started_at_ms: u32,
}

impl<const ID: i32> RestoreStateData<ID> {
    pub(crate) const fn new(time_to_keep_ms: u32, frequency_ms: u32, gain: u32) -> Self {
        Self {
            time_to_keep_ms,
            frequency_ms,
            gain,
            restore_count: 0,
            started_at_ms: 0,
        }
    }

    pub(crate) const fn state_id(&self) -> i32 {
        ID
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let _state_id = reader.read_i32()?;
        let mut state = Self::new(reader.read_u32()?, reader.read_u32()?, reader.read_u32()?);
        state.started_at_ms = now_ms;
        Ok(state)
    }

    pub(crate) fn begin_primary_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
        self.reset_restore_count();
    }

    pub(crate) fn reset_restore_count(&mut self) {
        self.restore_count = 0;
    }

    pub(crate) fn take_due_gain(&mut self, checked_at_ms: u32) -> Option<u32> {
        let due_at_ms = self.frequency_ms
            .wrapping_mul(self.restore_count)
            .wrapping_add(self.started_at_ms);
        if due_at_ms >= checked_at_ms {
            return None;
        }
        self.restore_count = self.restore_count.wrapping_add(1);
        Some(self.gain)
    }

    pub(crate) const fn expired(&self, checked_at_ms: u32) -> bool {
        self.time_to_keep_ms.wrapping_add(self.started_at_ms) < checked_at_ms
    }

    pub(crate) fn client_state_time(&self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.time_to_keep_ms, now) as i32
    }

    pub(crate) fn encoded(&self, now: impl FnMut() -> u32) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        self.encoded_with_remaining(self.client_state_time(now) as u32)
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        self.encoded_with_remaining(self.time_to_keep_ms)
    }

    fn encoded_with_remaining(&self, remaining: u32) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        let mut record = [0; CONSUMABLE_RESTORE_STATE_BYTES];
        for (slot, value) in record.chunks_exact_mut(4).zip([
            ID as u32, remaining, self.frequency_ms, self.gain,
        ]) {
            slot.copy_from_slice(&value.to_le_bytes());
        }
        record
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConsumableRestoreState {
    Health(RestoreHpState),
    Mana(RestoreMpState),
}

pub(crate) fn begin_primary_consumable_restore_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state: &mut ConsumableRestoreState,
    now: &mut dyn FnMut() -> u32,
) -> Option<(i32, ShapeIdentity)> {
    resolve_state_move_shape(game, region_id, holder)?;
    state.begin_primary_at(now());
    let shape = resolve_state_move_shape(game, region_id, holder)?.shape();
    Some((
        shape.get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() },
    ))
}

pub(crate) fn restart_consumable_restore_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ConsumableRestoreState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    if let Some(state) = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<ConsumableRestoreState>(key))
    {
        state.reset_restore_count();
    }
    true
}

impl ConsumableRestoreState {
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Option<Self> {
        match payload.get(offset..offset.checked_add(4)?)? {
            id if id == RESTORE_HP_STATE_ID.to_le_bytes() => {
                RestoreHpState::decode(payload, offset, now_ms).ok().map(Self::Health)
            }
            id if id == RESTORE_MP_STATE_ID.to_le_bytes() => {
                RestoreMpState::decode(payload, offset, now_ms).ok().map(Self::Mana)
            }
            _ => None,
        }
    }

    pub(crate) const fn state_id(&self) -> i32 {
        match self {
            Self::Health(state) => state.state_id(),
            Self::Mana(state) => state.state_id(),
        }
    }

    pub(crate) fn begin_primary_at(&mut self, now_ms: u32) {
        match self {
            Self::Health(state) => state.begin_primary_at(now_ms),
            Self::Mana(state) => state.begin_primary_at(now_ms),
        }
    }

    pub(crate) fn reset_restore_count(&mut self) {
        match self {
            Self::Health(state) => state.reset_restore_count(),
            Self::Mana(state) => state.reset_restore_count(),
        }
    }

    pub(crate) fn take_due_gain(&mut self, checked_at_ms: u32) -> Option<u32> {
        match self {
            Self::Health(state) => state.take_due_gain(checked_at_ms),
            Self::Mana(state) => state.take_due_gain(checked_at_ms),
        }
    }

    pub(crate) fn client_state_time(&self, now: impl FnMut() -> u32) -> i32 {
        match self {
            Self::Health(state) => state.client_state_time(now),
            Self::Mana(state) => state.client_state_time(now),
        }
    }

    pub(crate) const fn expired(&self, checked_at_ms: u32) -> bool {
        match self {
            Self::Health(state) => state.expired(checked_at_ms),
            Self::Mana(state) => state.expired(checked_at_ms),
        }
    }

    pub(crate) const fn is_health(&self) -> bool {
        matches!(self, Self::Health(_))
    }

    pub(crate) fn encoded(&self, now: impl FnMut() -> u32) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        match self {
            Self::Health(state) => state.encoded(now),
            Self::Mana(state) => state.encoded(now),
        }
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; CONSUMABLE_RESTORE_STATE_BYTES] {
        match self {
            Self::Health(state) => state.encoded_for_install(),
            Self::Mana(state) => state.encoded_for_install(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ConsumableRestoreIntervals {
    last_health_begin_ms: u32,
    last_mana_begin_ms: u32,
}

impl ConsumableRestoreIntervals {
    pub(crate) fn try_begin(
        &mut self,
        health: bool,
        interval_ms: u32,
        mut now: impl FnMut() -> u32,
    ) -> bool {
        let checked_at_ms = now();
        let last_begin_ms = if health {
            &mut self.last_health_begin_ms
        } else {
            &mut self.last_mana_begin_ms
        };
        if last_begin_ms.wrapping_add(interval_ms) > checked_at_ms {
            return false;
        }
        *last_begin_ms = now();
        true
    }
}
