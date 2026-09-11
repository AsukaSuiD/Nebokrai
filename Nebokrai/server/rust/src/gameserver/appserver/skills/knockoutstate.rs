//! Каноническое состояние оглушения `CKnockOutState` (`0x192`).
//! Истечение получает ключ конкретного экземпляра общей арены; проверка
//! срока и End не подменяют его первым состоянием с тем же ID.
//! Общий CBlindState::AI (0x005d5ba0) сравнивает абсолютный wrapping deadline
//! строго с now, в том числе при нулевом сроке; elapsed здесь неэквивалентен.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/knockoutstate.cpp`. Таблица виртуальных методов EXE
//! подтверждает, что `AI`, `End`, `OnAction`, `Serialize` и `Unserialize`
//! буквально используют реализацию `CBlindState`. Достигнутый путь сохраняет
//! беззнаковую проверку срока, снимает запреты движения и боя при истечении
//! либо защитном действии и публикует `0xBFE03/0xBFE04`. Persisted-запись
//! `ID + remaining time` декодируется, активируется StartAllStates после 8F801 и
//! удаляется вместе с canonical state. Невостребованные координатные
//! перегрузки сохранены ниже.
//! Monster-визуал получает явного владельца региона, поэтому сохраняет around-
//! доставку и тогда, когда AI временно извлёк регион из `CGame`.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;
use super::sealstate::SEAL_STATE_ID;
use super::blindstate::BLIND_STATE_ID;

pub(crate) const KNOCK_OUT_STATE_ID: u32 = 0x192;
pub(crate) const KNOCK_OUT_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KnockOutState { started_at_ms: u32, keep_time_ms: u32 }

impl KnockOutState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self { Self { started_at_ms, keep_time_ms } }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != KNOCK_OUT_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }
    pub(crate) const fn skill_id(self) -> u32 { KNOCK_OUT_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
    pub(crate) fn encoded_for_install(self) -> [u8; KNOCK_OUT_STATE_BYTES] {
        let mut bytes = [0; KNOCK_OUT_STATE_BYTES];
        bytes[..4].copy_from_slice(&KNOCK_OUT_STATE_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&self.keep_time_ms.to_le_bytes());
        bytes
    }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_knock_out_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: KnockOutState, begin: bool, now_milliseconds: impl FnMut() -> u32) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin { message.add_long(state.client_time(now_milliseconds)); message.add_long(0); }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

fn send_owned_monster_knock_out_state_visual(
    game: &CGame,
    region: &CServerRegion,
    shape: &crate::gameserver::appserver::shape::CShape,
    state: KnockOutState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let identity = shape.identity();
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin { message.add_long(state.client_time(now_milliseconds)); message.add_long(0); }
    let _ = game.send_game_shape_around(region, shape, None, &message);
}

pub(crate) fn replace_player_knock_out_state(
    game: &mut CGame,
    player_id: i32,
    state: KnockOutState,
    now_ms: u32,
) -> bool {
    let installed = game.find_player_mut(player_id).and_then(|player| {
        let region_id = player.server_region_id()?;
        let identity = player.shape().identity();
        let tile_x = player.shape().get_tile_x().ok()?;
        let tile_y = player.shape().get_tile_y().ok()?;
        let old = player.replace_knock_out_state(state);
        if old.is_some() {
            player.set_skill_fightable(true);
            player.set_skill_moveable(true);
        }
        player.set_skill_moveable(false);
        player.set_skill_fightable(false);
        Some((old, region_id, identity, tile_x, tile_y))
    });
    let Some((old, region_id, identity, tile_x, tile_y)) = installed else {
        return false;
    };
    if let Some(old) = old {
        send_knock_out_state_visual(game, region_id, identity, tile_x, tile_y, old, false, || now_ms);
    }
    send_knock_out_state_visual(
        game, region_id, identity, tile_x, tile_y, state, true, game_tick_milliseconds,
    );
    let _ = game.publish_player_states(player_id);
    true
}

pub(crate) fn replace_monster_knock_out_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    state: KnockOutState,
    now_ms: u32,
) -> bool {
    let installed = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let shape = monster.move_shape().shape().clone();
        let old = monster.move_shape_mut().replace_knock_out_state(state);
        if old.is_some() {
            monster.move_shape_mut().set_fightable(true);
            monster.move_shape_mut().set_moveable(true);
        }
        monster.move_shape_mut().set_moveable(false);
        monster.move_shape_mut().set_fightable(false);
        Some((old, shape))
    });
    let Some((old, shape)) = installed else {
        return false;
    };
    if let Some(old) = old {
        send_owned_monster_knock_out_state_visual(game, region, &shape, old, false, || now_ms);
    }
    send_owned_monster_knock_out_state_visual(
        game, region, &shape, state, true, game_tick_milliseconds,
    );
    true
}

pub(crate) fn finish_player_knock_out_state_on_defense(game: &mut CGame, player_id: i32, _now_ms: u32) -> bool {
    let context = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().identity(),
            player.move_shape().applied_state_key::<KnockOutState>()?))
    });
    let Some((region_id, identity, key)) = context else { return false };
    super::blindstate::end_blind_state(game, region_id, identity, key)
}

pub(crate) fn finish_player_blind_states_on_defense(game: &mut CGame, player_id: i32, _now_ms: u32) -> bool {
    let context = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().identity(),
            player.move_shape().blind_state_instances()))
    });
    let Some((region_id, identity, order)) = context else { return false };
    let mut changed = false;
    for (key, state_id) in order {
        if matches!(state_id, BLIND_STATE_ID | KNOCK_OUT_STATE_ID | SEAL_STATE_ID) {
            changed |= super::blindstate::end_blind_state(game, region_id, identity, key);
        }
    }
    changed
}

pub(crate) fn finish_blind_states_on_defense(game: &mut CGame, region: &mut CServerRegion, target: ShapeIdentity, now_ms: u32) -> bool {
    if target.object_type == 400 {
        return finish_player_blind_states_on_defense(game, target.id, now_ms);
    }
    if target.object_type != 600 {
        return false;
    }
    let order = region.find_monster_by_id(target.id)
        .map(|monster| monster.move_shape().blind_state_instances()).unwrap_or_default();
    let mut changed = false;
    for (key, state_id) in order {
        if matches!(state_id, BLIND_STATE_ID | KNOCK_OUT_STATE_ID | SEAL_STATE_ID) {
            changed |= super::blindstate::end_owned_monster_blind_state(game, region, target.id, key);
        }
    }
    changed
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.h

// ============================================================================
// FUNCTION: CKnockOutState::CKnockOutState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.cpp:24
// RVA: 0x001F4FA0
// ADDRESS: 005f4fa0
// PROTOTYPE: undefined __thiscall CKnockOutState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CKnockOutState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.cpp:57
// RVA: 0x001F5020
// ADDRESS: 005f5020
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CKnockOutState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.cpp:73
// RVA: 0x001F5100
// ADDRESS: 005f5100
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
