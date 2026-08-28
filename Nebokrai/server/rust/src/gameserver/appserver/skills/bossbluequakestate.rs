//! Каноническое состояние землетрясения синего босса `CBossBlueQuakeState` (`0x1f8`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluequakestate.cpp`. Достигнутый путь хранит состояние
//! только в `CanonicalStateStorage`, при замене завершает прежнюю блокировку,
//! затем запрещает движение и бой до строгой границы срока. Пакеты начала и
//! завершения сохраняют `0xBFE03/04`. Неподключённые перегрузки и сохранение в
//! долговременный формат остаются исходным материалом ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequakestate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequakestate.h

// ============================================================================
// FUNCTION: CBossBlueQuakeState::CBossBlueQuakeState
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутая длительность хранится в `BossBlueQuakeState`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequakestate.cpp:15
// RVA: 0x001E8590
// ADDRESS: 005e8590
// PROTOTYPE: undefined __thiscall CBossBlueQuakeState(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuakeState::CBossBlueQuakeState
// STATUS: PARTIALLY_IMPLEMENTED
// Пустой исходный конструктор не используется каноническим хранилищем.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequakestate.cpp:24
// RVA: 0x001E8600
// ADDRESS: 005e8600
// PROTOTYPE: undefined __thiscall CBossBlueQuakeState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuakeState::~CBossBlueQuakeState
// STATUS: PARTIALLY_IMPLEMENTED
// Техническое разрушение экземпляра заменено владением Rust.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequakestate.cpp:33
// RVA: 0x001E8670
// ADDRESS: 005e8670
// PROTOTYPE: void __thiscall ~CBossBlueQuakeState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuakeState::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Координатная перегрузка не подключена к достигнутому пути.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequakestate.cpp:57
// RVA: 0x001E8680
// ADDRESS: 005e8680
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuakeState::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Перегрузка с отдельными типом и идентификатором цели остаётся исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequakestate.cpp:73
// RVA: 0x001E8760
// ADDRESS: 005e8760
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuakeState::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутая цель изменяется через каноническое хранилище состояния.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequakestate.cpp:44
// RVA: 0x001E8860
// ADDRESS: 005e8860
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuakeStateVisualEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутые начало и завершение формирует `send_boss_blue_quake_state_visual`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequakestate.cpp:144
// RVA: 0x001E8920
// ADDRESS: 005e8920
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const BOSS_BLUE_QUAKE_STATE_ID: u32 = 0x1f8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BossBlueQuakeState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl BossBlueQuakeState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) const fn skill_id(self) -> u32 { BOSS_BLUE_QUAKE_STATE_ID }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms);
        if deadline <= now_ms { 0 } else { deadline.wrapping_sub(now_ms) as i32 }
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точную точку круговой доставки состояния")]
pub(crate) fn send_boss_blue_quake_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BossBlueQuakeState,
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

pub(crate) fn expire_player_boss_blue_quake_state(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    let Some((region_id, identity, tile_x, tile_y, state)) = game.find_player_mut(player_id).and_then(|player| {
        let state = player.take_expired_boss_blue_quake_state(now_ms)?;
        player.set_skill_moveable(true);
        player.set_skill_fightable(true);
        Some((player.server_region_id()?, player.shape().identity(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, state))
    }) else { return false };
    send_boss_blue_quake_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms);
    true
}

pub(crate) fn expire_monster_boss_blue_quake_state(
    game: &mut CGame,
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    let Some((identity, tile_x, tile_y, state)) = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_expired_boss_blue_quake_state(now_ms)?;
        monster.move_shape_mut().set_moveable(true);
        monster.move_shape_mut().set_fightable(true);
        Some((monster.move_shape().shape().identity(), monster.move_shape().shape().get_tile_x().ok()?, monster.move_shape().shape().get_tile_y().ok()?, state))
    }) else { return false };
    send_boss_blue_quake_state_visual(game, region.id, identity, tile_x, tile_y, state, false, now_ms);
    true
}
