//! Каноническая загружаемая часть `CBlindState` (`0x76`).
//! Истечение получает ключ конкретного экземпляра общей арены; проверка
//! срока и End не подменяют его первым состоянием с тем же ID.
//! Vtable 0x00662214, слот +0x0c: CBlindState::AI (0x005d5ba0).
//! Срок проверяется как start.wrapping_add(keep) < now, включая keep == 0;
//! elapsed-сравнение не сохраняет исходный переход DWORD через ноль.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/blindstate.cpp`. Достигнутый путь сохраняет восьмибайтную
//! запись `state ID + remaining time`, строгую беззнаковую границу таймера,
//! запреты движения и боя, завершение при `ACTION_DEFENSE` и точные сообщения
//! `0xBFE03/0xBFE04`. Единственным владельцем состояния остаётся
//! `CanonicalStateStorage`; неизменённый legacy payload служит его кодеком.
//! Перегрузки `Begin`, для которых ещё нет настоящего создающего caller-а,
//! сохранены ниже как RAW.
//! Vtable Blind/KnockOut/SpiderWeb/Seal/Strike
//! 0x00662214/0x00660894/0x0065FBCC/0x006615F4/0x00662154 имеют общие
//! AI +0x0C→0x005D5BA0 и End +0x1C→0x005EA9A0. End сначала публикует
//! окончание, затем SetFightable(1), SetMoveable(1), RemoveState и его
//! UpdateProperty. Общая single-key цепочка ниже работает с опубликованным
//! владельцем и перечитывает тот же ключ после доставки, не вынимая payload
//! перед callback. AI не добавляет alive-gate и не исключает нулевой срок.
//! OnAction +0x34 общим не является: Blind/KnockOut/Seal вызывают
//! 0x00607370 (End при ACTION_DEFENSE), SpiderWeb/Strike — 0x00601A70
//! (ret 4). После фактического удаления общий virtual UpdateProperty
//! пересчитывает живые свойства держателя, включая модификаторы монстра.
//! Region-adapter Defense использует того же владельца
//! payload и общий tail End, не копируя временную владеющую форму.
//! Object Begin Blind/KnockOut/SpiderWeb/Seal/Strike по адресам
//! 0x006075E0/0x005F5230/0x005EAA00/0x005FFAD0/0x00606AA0:
//! null sufferer даёт отказ до базы. Далее base Begin→новый visual(0xC)→
//! BeginVisualEffect(1)→Update(state,0)→SetMoveable(0)→SetFightable(0)→1.
//! Begin(NULL,holder) не читает clock и сохраняет timestamp Unserialize;
//! сама Unserialize 0x005EAAC0 читает clock до DWORD оставшегося срока.
//! Clock визуального GetRemainedTime остаётся отдельным от этих операций.
//! OnChangeRegion +0x2C (0x005E3B30) меняет sufferer-region в общей базе.
//! GetRemainedTime0x005F2CD0 и Serialize0x005F51E0 исполняются общими
//! timed getter/ordered codec: ID, затем отдельные clock-read остатка срока.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp

// ============================================================================
// FUNCTION: CBlindState::OnAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.h:33
// RVA: 0x00207370
// ADDRESS: 00607370
// PROTOTYPE: void __thiscall OnAction(tagAction param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::CBlindState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:24
// RVA: 0x00207380
// ADDRESS: 00607380
// PROTOTYPE: undefined __thiscall CBlindState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::~CBlindState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:33
// RVA: 0x002073F0
// ADDRESS: 006073f0
// PROTOTYPE: void __thiscall ~CBlindState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:58
// RVA: 0x00207400
// ADDRESS: 00607400
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:74
// RVA: 0x002074E0
// ADDRESS: 006074e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:44
// RVA: 0x002075E0
// ADDRESS: 006075e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:141
// RVA: 0x002076A0
// ADDRESS: 006076a0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//







// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::{StateData, StateKey};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, resolve_state_move_shape,
    resolve_state_move_shape_mut, timed_client_state_time, update_applied_state_visual_base,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const BLIND_STATE_ID: u32 = 0x76;
pub(crate) const BLIND_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BlindState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl BlindState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
        }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != BLIND_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }

    pub(crate) const fn skill_id(self) -> u32 {
        BLIND_STATE_ID
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    /// Exact `GetRemainedTime`: deadline-check и положительный остаток читают
    /// wrapping clock независимо.
    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds)
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "поля задают точку фактической круговой доставки"
)]
pub(crate) fn send_blind_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BlindState,
    begin: bool,
    mut now_milliseconds: impl FnMut() -> u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_ulong(state.client_state_time(&mut now_milliseconds));
        message.add_ulong(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn restart_blind_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_data(key)).is_some_and(StateData::is_blind)
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key)
        || !begin_applied_state_visual(game, region_id, holder, key, 1)
    {
        return false;
    }
    let snapshot = resolve_state_move_shape(game, region_id, holder).and_then(|shape| {
        let state = shape.applied_state_data(key)?;
        let remaining = match state {
            StateData::Blind(state) => state.client_state_time(&mut *now),
            StateData::KnockOut(state) => state.client_time(&mut *now) as u32,
            StateData::SpiderWeb(state) => state.client_time(&mut *now) as u32,
            StateData::Seal(state) => state.client_time(&mut *now) as u32,
            StateData::Strike(state) => state.client_time(&mut *now),
            _ => return None,
        };
        Some((state.state_id(), remaining))
    });
    let Some((state_id, remaining)) = snapshot else { return false };
    let mut message = CMessage::new(0x000b_fe03);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state_id as i32);
    message.add_ulong(remaining);
    message.add_ulong(0);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let _ = update_applied_state_visual_base(game, region_id, holder, key);
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_moveable(false);
        shape.set_fightable(false);
    }
    true
}

pub(crate) fn update_blind_state(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = resolve_state_move_shape(game, region_id, identity)
        .and_then(|shape| shape.applied_state_data(key))
        .is_some_and(|state| match state {
            StateData::Blind(state) => state.expired(now_ms),
            StateData::KnockOut(state) => state.expired(now_ms),
            StateData::SpiderWeb(state) => state.expired(now_ms),
            StateData::Seal(state) => state.expired(now_ms),
            StateData::Strike(state) => state.expired(now_ms),
            _ => false,
        });
    expired && end_blind_state(game, region_id, identity, key)
}

pub(crate) fn end_blind_state(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    key: StateKey,
) -> bool {
    let context = resolve_state_move_shape(game, region_id, identity).and_then(|shape| {
        let state = shape.applied_state_data(key).filter(|state| state.is_blind())?;
        let position = shape.shape().get_tile_x().ok().zip(shape.shape().get_tile_y().ok());
        Some((state.state_id(), position))
    });
    let Some((state_id, position)) = context else { return false };
    if let Some((tile_x, tile_y)) = position {
        let mut message = CMessage::new(0x000b_fe04);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(state_id as i32);
        let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
    }
    let removed = resolve_state_move_shape_mut(game, region_id, identity)
        .is_some_and(|shape| finish_blind_state(shape, key));
    if removed {
        let _ = game.update_move_shape_properties(region_id, identity);
    }
    removed
}

fn finish_blind_state(
    shape: &mut crate::gameserver::appserver::moveshape::CMoveShape,
    key: StateKey,
) -> bool {
    if !shape.applied_state_data(key).is_some_and(StateData::is_blind) {
        return false;
    }
    shape.set_fightable(true);
    shape.set_moveable(true);
    shape.remove_applied_state_data(key, BLIND_STATE_BYTES).is_some()
}
