//! Каноническое состояние оглушения `CKnockOutState` (`0x192`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/knockoutstate.cpp`. Таблица виртуальных методов EXE
//! подтверждает, что `AI`, `End`, `OnAction`, `Serialize` и `Unserialize`
//! буквально используют реализацию `CBlindState`. Достигнутый путь сохраняет
//! беззнаковую проверку срока, снимает запреты движения и боя при истечении
//! либо защитном действии и публикует `0xBFE03/0xBFE04`. Координатные
//! перегрузки и ещё не
//! подключённая загрузка старой записи сохранены ниже.
//! Monster-визуал получает явного владельца региона, поэтому сохраняет around-
//! доставку и тогда, когда AI временно извлёк регион из `CGame`.

use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use super::spiderweb::SPIDER_WEB_SKILL_ID;
use super::spiderwebstate::{
    expire_monster_spider_web_state, expire_player_spider_web_state,
    finish_player_spider_web_state_on_defense, finish_spider_web_state_on_defense,
};
use super::sealstate::{
    SEAL_STATE_ID, expire_monster_seal_state, finish_monster_seal_state_on_defense,
};

pub(crate) const KNOCK_OUT_STATE_ID: u32 = 0x192;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KnockOutState { started_at_ms: u32, keep_time_ms: u32 }

impl KnockOutState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self { Self { started_at_ms, keep_time_ms } }
    pub(crate) const fn skill_id(self) -> u32 { KNOCK_OUT_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms }
    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let elapsed = now_ms.wrapping_sub(self.started_at_ms);
        if elapsed >= self.keep_time_ms { 0 } else { self.keep_time_ms.wrapping_sub(elapsed) as i32 }
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_knock_out_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: KnockOutState, begin: bool, now_ms: u32) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin { message.add_long(state.client_time(now_ms)); message.add_long(0); }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

fn send_owned_monster_knock_out_state_visual(
    game: &CGame,
    region: &CServerRegion,
    shape: &crate::gameserver::appserver::shape::CShape,
    state: KnockOutState,
    begin: bool,
    now_ms: u32,
) {
    let identity = shape.identity();
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin { message.add_long(state.client_time(now_ms)); message.add_long(0); }
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
        send_knock_out_state_visual(game, region_id, identity, tile_x, tile_y, old, false, now_ms);
    }
    send_knock_out_state_visual(game, region_id, identity, tile_x, tile_y, state, true, now_ms);
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
        send_owned_monster_knock_out_state_visual(game, region, &shape, old, false, now_ms);
    }
    send_owned_monster_knock_out_state_visual(game, region, &shape, state, true, now_ms);
    true
}

fn finish_player_state(game: &mut CGame, player_id: i32, now_ms: u32, only_expired: bool) -> bool {
    let finished = game.find_player_mut(player_id).and_then(|player| {
        let state = if only_expired { player.take_expired_knock_out_state(now_ms)? } else { player.take_knock_out_state()? };
        player.set_skill_fightable(true);
        player.set_skill_moveable(true);
        Some((state, player.server_region_id()?, player.shape().identity(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
    });
    let Some((state, region_id, identity, tile_x, tile_y)) = finished else { return false };
    send_knock_out_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms);
    true
}

fn finish_monster_state(game: &mut CGame, region: &mut CServerRegion, monster_id: i32, now_ms: u32, only_expired: bool) -> bool {
    let finished = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = if only_expired { monster.move_shape_mut().take_expired_knock_out_state(now_ms)? } else { monster.move_shape_mut().take_knock_out_state()? };
        monster.move_shape_mut().set_fightable(true);
        monster.move_shape_mut().set_moveable(true);
        Some((state, monster.move_shape().shape().clone()))
    });
    let Some((state, shape)) = finished else { return false };
    send_owned_monster_knock_out_state_visual(game, region, &shape, state, false, now_ms);
    true
}

pub(crate) fn expire_player_knock_out_state(game: &mut CGame, player_id: i32, now_ms: u32) -> bool { finish_player_state(game, player_id, now_ms, true) }
pub(crate) fn finish_player_knock_out_state_on_defense(game: &mut CGame, player_id: i32, now_ms: u32) -> bool { finish_player_state(game, player_id, now_ms, false) }
pub(crate) fn expire_monster_knock_out_state(game: &mut CGame, region: &mut CServerRegion, monster_id: i32, now_ms: u32) -> bool { finish_monster_state(game, region, monster_id, now_ms, true) }
pub(crate) fn finish_knock_out_state_on_defense(game: &mut CGame, region: &mut CServerRegion, target: ShapeIdentity, now_ms: u32) -> bool {
    match target.object_type { 400 => finish_player_state(game, target.id, now_ms, false), 600 => finish_monster_state(game, region, target.id, now_ms, false), _ => false }
}

pub(crate) fn expire_player_blind_states(game: &mut CGame, player_id: i32, now_ms: u32) -> bool {
    let order = game.find_player(player_id).map(|player| player.blind_state_order()).unwrap_or_default();
    let mut changed = false;
    for state_id in order {
        changed |= match state_id {
            SPIDER_WEB_SKILL_ID => expire_player_spider_web_state(game, player_id, now_ms),
            KNOCK_OUT_STATE_ID => expire_player_knock_out_state(game, player_id, now_ms),
            _ => false,
        };
    }
    changed
}

pub(crate) fn finish_player_blind_states_on_defense(game: &mut CGame, player_id: i32, now_ms: u32) -> bool {
    let order = game.find_player(player_id).map(|player| player.blind_state_order()).unwrap_or_default();
    let mut changed = false;
    for state_id in order {
        changed |= match state_id {
            SPIDER_WEB_SKILL_ID => finish_player_spider_web_state_on_defense(game, player_id, now_ms),
            KNOCK_OUT_STATE_ID => finish_player_knock_out_state_on_defense(game, player_id, now_ms),
            _ => false,
        };
    }
    changed
}

pub(crate) fn expire_monster_blind_states(game: &mut CGame, region: &mut CServerRegion, monster_id: i32, now_ms: u32) -> bool {
    let order = region.find_monster_by_id(monster_id).map(|monster| monster.move_shape().blind_state_order()).unwrap_or_default();
    let mut changed = false;
    for state_id in order {
        changed |= match state_id {
            SPIDER_WEB_SKILL_ID => expire_monster_spider_web_state(game, region, monster_id, now_ms),
            KNOCK_OUT_STATE_ID => expire_monster_knock_out_state(game, region, monster_id, now_ms),
            SEAL_STATE_ID => expire_monster_seal_state(game, region, monster_id, now_ms),
            _ => false,
        };
    }
    changed
}

pub(crate) fn finish_blind_states_on_defense(game: &mut CGame, region: &mut CServerRegion, target: ShapeIdentity, now_ms: u32) -> bool {
    let order = match target.object_type {
        400 => game.find_player(target.id).map(|player| player.blind_state_order()),
        600 => region.find_monster_by_id(target.id).map(|monster| monster.move_shape().blind_state_order()),
        _ => None,
    }.unwrap_or_default();
    let mut changed = false;
    for state_id in order {
        changed |= match state_id {
            SPIDER_WEB_SKILL_ID => finish_spider_web_state_on_defense(game, region, target, now_ms),
            KNOCK_OUT_STATE_ID => finish_knock_out_state_on_defense(game, region, target, now_ms),
            SEAL_STATE_ID if target.object_type == 600 => {
                finish_monster_seal_state_on_defense(game, region, target.id, now_ms)
            }
            _ => false,
        };
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
