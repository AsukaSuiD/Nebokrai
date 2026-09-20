//! Связывание CBoaLockState, ID 0xD2: запрет движения без запрета боя.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/boalockstate.cpp.
//!
//! Объектный Begin требует S; NULL U сохраняет время и источник. Общий
//! control-lifecycle обновляет timestamp при U, публикует loop1 visual,
//! запрещает движение исходному S и только затем передаёт запись SlotMap.
//! При применении первый ID 0xD2 получает End/destructor, а новый экземпляр
//! добавляется в конец. Старые одноимённые записи не сливаются с новой.
//!
//! End: visual → свежий S → снятие одного move-lock → RemoveState(pointer).
//! Состояние не записывает ended; чужой или отсутствующий S не удаляет
//! запись из арены держателя. Полный destructor остатка принадлежит caller-у.
//! Restart сохраняет время и U, обновляет S и создаёт новый loop1 visual.
//! В отличие от KnockOut, Defense не завершает BoaLock.
//!
//! AI сравнивает wrapping start+keep строго с now, включая keep=0.
//! DB-запись ID/remaining занимает восемь байт. Load читает часы перед сроком,
//! Save и клиентский visual используют общий срок с одним либо двумя чтениями.
//! Additional равен нулю; SetRegion меняет только сохранённый регион S.
//! Неиспользуемые координатная и типизированная перегрузки Begin не заменяются
//! объектным runtime: их проверка GetS после base Begin имеет отдельный контракт.

use super::blindstate::{self, BlindStatePayload};
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_state_move_shape, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;

pub(crate) const BOA_LOCK_STATE_ID: u32 = 0xd2;
pub(crate) const BOA_LOCK_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BoaLockState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl BoaLockState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != BOA_LOCK_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }

    pub(crate) const fn skill_id(self) -> u32 { BOA_LOCK_STATE_ID }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }

    pub(crate) fn encoded_for_install(self) -> [u8; BOA_LOCK_STATE_BYTES] {
        self.encode_record(|| self.keep_time_ms)
    }

    pub(crate) fn encoded(self, now: impl FnMut() -> u32) -> [u8; BOA_LOCK_STATE_BYTES] {
        self.encode_record(|| self.client_time(now) as u32)
    }

    fn encode_record(self, remaining: impl FnOnce() -> u32) -> [u8; BOA_LOCK_STATE_BYTES] {
        let mut record = [0; BOA_LOCK_STATE_BYTES];
        record[..4].copy_from_slice(&BOA_LOCK_STATE_ID.to_le_bytes());
        record[4..].copy_from_slice(&remaining().to_le_bytes());
        record
    }
}

impl BlindStatePayload for BoaLockState {
    fn blind_state_id(&self) -> u32 { BOA_LOCK_STATE_ID }
    fn begin_at(&mut self, now_ms: u32) { self.started_at_ms = now_ms; }
    fn remaining(&self, now: &mut dyn FnMut() -> u32) -> u32 { self.client_time(now) as u32 }
    fn install_record(&self) -> [u8; BOA_LOCK_STATE_BYTES] { self.encoded_for_install() }
    fn blocks_fighting(&self) -> bool { false }
}

pub(super) fn replace_boa_lock_state(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    state: BoaLockState, now: &mut dyn FnMut() -> u32,
) -> bool {
    let previous = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|old| old.state_id() == BOA_LOCK_STATE_ID));
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    blindstate::begin_primary_blind_state(
        game, target.0, target.1, Some(source), Some(target), state, now,
    ).is_some()
}

pub(crate) fn restart_boa_lock_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    blindstate::restart_blind_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn update_boa_lock_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    blindstate::update_blind_state(game, region_id, holder, key, now_ms)
}

pub(crate) fn end_boa_lock_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    blindstate::end_blind_state(game, region_id, holder, key)
}
