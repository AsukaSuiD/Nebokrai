//! Каноническое состояние паутины `CSpiderWebState` (`0x199`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spiderwebstate.cpp`. Состояние сохраняет wrapping-время,
//! запрещает движение и бой через счётчики `CMoveShape`, снимает оба запрета
//! при замене, истечении или защитном действии и публикует `0xBFE03/0xBFE04`.
//! Координатные перегрузки и legacy-сериализация остаются RAW ниже.

use super::spiderweb::SPIDER_WEB_SKILL_ID;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpiderWebState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl SpiderWebState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        SPIDER_WEB_SKILL_ID
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let elapsed = now_ms.wrapping_sub(self.started_at_ms);
        if elapsed >= self.keep_time_ms {
            0
        } else {
            self.keep_time_ms.wrapping_sub(elapsed) as i32
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_spider_web_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: SpiderWebState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

fn finish_player_state(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
    only_expired: bool,
) -> bool {
    let finished = game.find_player_mut(player_id).and_then(|player| {
        let state = if only_expired {
            player.take_expired_spider_web_state(now_ms)?
        } else {
            player.take_spider_web_state()?
        };
        player.set_skill_moveable(true);
        player.set_skill_fightable(true);
        Some((
            state,
            player.server_region_id()?,
            player.shape().identity(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        ))
    });
    let Some((state, region_id, identity, tile_x, tile_y)) = finished else {
        return false;
    };
    send_spider_web_state_visual(
        game, region_id, identity, tile_x, tile_y, state, false, now_ms,
    );
    true
}

fn finish_monster_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
    only_expired: bool,
) -> bool {
    let finished = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = if only_expired {
            monster
                .move_shape_mut()
                .take_expired_spider_web_state(now_ms)?
        } else {
            monster.move_shape_mut().take_spider_web_state()?
        };
        monster.move_shape_mut().set_moveable(true);
        monster.move_shape_mut().set_fightable(true);
        Some((
            state,
            monster.move_shape().shape().identity(),
            monster.move_shape().shape().get_tile_x().ok()?,
            monster.move_shape().shape().get_tile_y().ok()?,
        ))
    });
    let Some((state, identity, tile_x, tile_y)) = finished else {
        return false;
    };
    send_spider_web_state_visual(
        game, region.id, identity, tile_x, tile_y, state, false, now_ms,
    );
    true
}

pub(crate) fn expire_player_spider_web_state(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    finish_player_state(game, player_id, now_ms, true)
}

pub(crate) fn finish_player_spider_web_state_on_defense(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    finish_player_state(game, player_id, now_ms, false)
}

pub(crate) fn expire_monster_spider_web_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    finish_monster_state(game, region, monster_id, now_ms, true)
}

/// `CMoveShape::OnAction(ACTION_DEFENSE)` вызывает `CBlindState::OnAction`
/// до итогового пакета полученного удара. Здесь сохраняется тот же момент,
/// включая снятие обоих вложенных запретов перед `0xBFE04`.
pub(crate) fn finish_spider_web_state_on_defense(
    game: &mut CGame,
    region: &mut CServerRegion,
    target: ShapeIdentity,
    now_ms: u32,
) -> bool {
    match target.object_type {
        400 => finish_player_state(game, target.id, now_ms, false),
        600 => finish_monster_state(game, region, target.id, now_ms, false),
        _ => false,
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp

// ============================================================================
// FUNCTION: CSpiderWebState::CSpiderWebState(long)
// STATUS: IMPLEMENTED
// `SpiderWebState::new` хранит исходную wrapping-длительность и ID `0x199`.

// ============================================================================
// FUNCTION: CSpiderWebState::CSpiderWebState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp:24
// RVA: 0x001EA750
// ADDRESS: 005ea750
// PROTOTYPE: undefined __thiscall CSpiderWebState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWebState::~CSpiderWebState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp:33
// RVA: 0x001EA7C0
// ADDRESS: 005ea7c0
// PROTOTYPE: void __thiscall ~CSpiderWebState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWebState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp:61
// RVA: 0x001EA7D0
// ADDRESS: 005ea7d0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWebState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderwebstate.cpp:81
// RVA: 0x001EA8B0
// ADDRESS: 005ea8b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWebState::Begin(CMoveShape *, CMoveShape *)
// STATUS: IMPLEMENTED
// Установка через `execute_owned_spider_web` атомарно заменяет каноническое
// состояние, счётчики движения и боя и начальный визуальный эффект.

// ============================================================================
// FUNCTION: CSpiderWebStateVisualEffect::UpdateVisualEffect
// STATUS: IMPLEMENTED
// `send_spider_web_state_visual` сохраняет точные begin/end payload и момент
// круговой доставки.

// COMPONENT_VARIANT_END: GameServer
