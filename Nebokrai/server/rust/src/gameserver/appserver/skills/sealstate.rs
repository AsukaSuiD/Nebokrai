//! Каноническое состояние печати `CSealState` (`0x138`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/sealstate.cpp`. Достигнутая объектная перегрузка запрещает
//! монстру движение и бой и публикует `0xBFE03`. Унаследованные от
//! `CBlindState` таймер, `End` и реакция на защиту снимают оба запрета и
//! публикуют `0xBFE04`; беззнаковая строгая проверка срока и порядок вставки
//! сохраняются каноническим хранилищем. Exact vtable также подтверждает
//! 8-байтовую persisted-запись `ID + remaining time`; codec подключён к
//! общему `CMoveShape` storage и восстанавливает унаследованный blind lifecycle.
//! Конструктор по умолчанию и
//! координатные перегрузки остаются в RAW ниже. Унаследованный клиентский срок
//! использует общее тело `CBlindState` по `0x005F2CD0`.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    send_owned_state_visual, timed_client_state_time,
};
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;

pub(crate) const SEAL_STATE_ID: u32 = 0x138;
pub(crate) const SEAL_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SealState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl SealState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        SEAL_STATE_ID
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != SEAL_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(0, reader.read_u32()?))
    }

    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self {
        self.started_at_ms = now_ms;
        self
    }

    pub(crate) fn encoded_for_install(self) -> [u8; SEAL_STATE_BYTES] {
        let mut bytes = [0; SEAL_STATE_BYTES];
        bytes[..4].copy_from_slice(&SEAL_STATE_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&self.keep_time_ms.to_le_bytes());
        bytes
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_seal_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: SealState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn replace_monster_seal_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    state: SealState,
    _now_ms: u32,
) -> bool {
    let installed = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let shape = monster.move_shape().shape().clone();
        let previous = monster.move_shape_mut().replace_seal_state(state);
        if previous.is_some() {
            monster.move_shape_mut().set_moveable(true);
            monster.move_shape_mut().set_fightable(true);
        }
        monster.move_shape_mut().set_moveable(false);
        monster.move_shape_mut().set_fightable(false);
        Some((previous, shape))
    });
    let Some((previous, shape)) = installed else {
        return false;
    };
    if let Some(previous) = previous {
        send_owned_state_visual(game, region, &shape, previous.skill_id(), false, 0, 0);
    }
    send_owned_state_visual(
        game,
        region,
        &shape,
        state.skill_id(),
        true,
        state.client_time(game_tick_milliseconds),
        0,
    );
    true
}

pub(crate) fn expire_monster_seal_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    let finished = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_expired_seal_state(now_ms)?;
        monster.move_shape_mut().set_moveable(true);
        monster.move_shape_mut().set_fightable(true);
        Some((
            state,
            monster.move_shape().shape().clone(),
        ))
    });
    let Some((state, shape)) = finished else {
        return false;
    };
    send_owned_state_visual(game, region, &shape, state.skill_id(), false, 0, 0);
    true
}

pub(crate) fn finish_monster_seal_state_on_defense(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    _now_ms: u32,
) -> bool {
    let finished = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_seal_state()?;
        monster.move_shape_mut().set_moveable(true);
        monster.move_shape_mut().set_fightable(true);
        Some((
            state,
            monster.move_shape().shape().clone(),
        ))
    });
    let Some((state, shape)) = finished else {
        return false;
    };
    send_owned_state_visual(game, region, &shape, state.skill_id(), false, 0, 0);
    true
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\sealstate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\sealstate.h

// ============================================================================
// FUNCTION: CSealState::CSealState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\sealstate.cpp:24
// RVA: 0x001FF870
// ADDRESS: 005ff870
// PROTOTYPE: undefined __thiscall CSealState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSealState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\sealstate.cpp:57
// RVA: 0x001FF8F0
// ADDRESS: 005ff8f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSealState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\sealstate.cpp:73
// RVA: 0x001FF9D0
// ADDRESS: 005ff9d0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
