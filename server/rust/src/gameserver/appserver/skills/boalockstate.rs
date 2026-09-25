//! Связывание CBoaLockState, ID 0xD2: запрет движения без запрета боя.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/boalockstate.cpp.
//! Данные и 8-байтная запись перенесены в Zone `effects/blind.rs`
//! (конструкторы VA 0x005FB560/0x005FB5D0, vtable 0x006610DC; её собственный
//! End VA 0x005FB800 соответствует `BLOCKS_FIGHTING = false` в типе Zone).
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
//! Additional равен нулю; SetRegion меняет только сохранённый регион S.
//! Неиспользуемые координатная и типизированная перегрузки Begin не заменяются
//! объектным runtime: их проверка GetS после base Begin имеет отдельный контракт.

use super::blindstate;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_and_destroy_state_at, resolve_state_move_shape};
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::effects::{BOA_LOCK_STATE_ID, BoaLockState};

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
