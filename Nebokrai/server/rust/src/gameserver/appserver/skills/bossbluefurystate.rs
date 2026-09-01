//! Каноническое состояние ярости синего босса `CBossBlueFuryState` (`0x1f7`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluefurystate.cpp`. Достигнутые пути игрока и монстра
//! заменяют прежнее состояние до установки нового, запрещают движение и бой на слабой
//! фазе, а затем сохраняют состояние до общего срока. Только для монстра минимальная
//! и максимальная атака заменяются указанной долей коэффициента с исходным округлением
//! дробной части строго больше `0.5`; визуальные начало и завершение сохраняют
//! `0xBFE03/04`. DB-запись содержит остаток срока и коэффициент атаки;
//! `weak_time` после загрузки остаётся нулевым, а `Begin` повторно не вызывается.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp

// ============================================================================
// FUNCTION: CBossBlueFuryState::CBossBlueFuryState
// STATUS: PARTIALLY_IMPLEMENTED
// Объектная ветвь монстра выполняется `BossBlueFuryState::new`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:15
// RVA: 0x001E8A60
// ADDRESS: 005e8a60
// PROTOTYPE: undefined __thiscall CBossBlueFuryState(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::CBossBlueFuryState
// STATUS: PARTIALLY_IMPLEMENTED
// Объектная ветвь монстра выполняется `BossBlueFuryState::new`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:26
// RVA: 0x001E8AE0
// ADDRESS: 005e8ae0
// PROTOTYPE: undefined __thiscall CBossBlueFuryState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::~CBossBlueFuryState
// STATUS: PARTIALLY_IMPLEMENTED
// Завершение достигнутых путей выполняют `expire_monster_boss_blue_fury_state`
// и `expire_player_boss_blue_fury_state`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:37
// RVA: 0x001E8B50
// ADDRESS: 005e8b50
// PROTOTYPE: void __thiscall ~CBossBlueFuryState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:89
// RVA: 0x001E8B60
// ADDRESS: 005e8b60
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:106
// RVA: 0x001E8C30
// ADDRESS: 005e8c30
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::End
// STATUS: IMPLEMENTED
// Завершение достигнутых путей выполняют функции истечения для игрока и монстра,
// а также ветви замены в соответствующих исполнителях навыка.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:123
// RVA: 0x001E8D10
// ADDRESS: 005e8d10
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::AI
// STATUS: IMPLEMENTED
// Слабая и общая границы времени выполняются `BossBlueFuryState::tick`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:147
// RVA: 0x001E8D50
// ADDRESS: 005e8d50
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::OnUpdateProperties
// STATUS: PARTIALLY_IMPLEMENTED
// Формула монстра выполняется `BossBlueFuryState::apply_to_monster_attack`;
// исходная проверка типа `600` не применяет коэффициент к игроку.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:48
// RVA: 0x001E8DC0
// ADDRESS: 005e8dc0
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектные ветви игрока и монстра выполняются `BossBlueFuryState::new`
// и каноническим владельцем в `CMoveShape`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:75
// RVA: 0x001E8ED0
// ADDRESS: 005e8ed0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryStateVisualEffect::UpdateVisualEffect
// STATUS: IMPLEMENTED
// Достигнутый путь выполняет `send_boss_blue_fury_state_visual`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:204
// RVA: 0x001E8F80
// ADDRESS: 005e8f80
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Restart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:142
// RVA: 0x001FD450
// ADDRESS: 005fd450
// PROTOTYPE: void __thiscall Restart(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    send_owned_state_visual, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const BOSS_BLUE_FURY_STATE_ID: u32 = 0x1f7;
pub(crate) const BOSS_BLUE_FURY_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BossBlueFuryState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_factor_percent: i32,
    weak_time_ms: u32,
    weak_released: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BossBlueFuryTick {
    pub(crate) release_control: bool,
    pub(crate) expired: bool,
}

impl BossBlueFuryState {
    pub(crate) const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        attack_factor_percent: i32,
        weak_time_ms: u32,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            attack_factor_percent,
            weak_time_ms,
            weak_released: false,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        BOSS_BLUE_FURY_STATE_ID
    }

    pub(crate) const fn control_locked(self) -> bool {
        !self.weak_released
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != BOSS_BLUE_FURY_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?, 0))
    }

    pub(crate) fn encoded(self, now_ms: u32) -> [u8; BOSS_BLUE_FURY_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(BOSS_BLUE_FURY_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(BOSS_BLUE_FURY_STATE_ID);
        writer.write_u32(self.client_time(|| now_ms) as u32);
        writer.write_i32(self.attack_factor_percent);
        bytes.try_into().expect("размер состояния ярости синего босса фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; BOSS_BLUE_FURY_STATE_BYTES] {
        self.encoded(self.started_at_ms)
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub(crate) fn tick(&mut self, now_ms: u32) -> BossBlueFuryTick {
        let expired = self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms;
        let release_control = !self.weak_released
            && (self.started_at_ms.wrapping_add(self.weak_time_ms) < now_ms || expired);
        if release_control {
            self.weak_released = true;
        }
        BossBlueFuryTick {
            release_control,
            expired,
        }
    }

    pub(crate) fn apply_to_monster_attack(self, attack: u32) -> u32 {
        let scaled = self.attack_factor_percent as f32 * 0.01 * attack as f32;
        let truncated = scaled.trunc() as i32;
        let rounded = if scaled - truncated as f32 > 0.5 {
            truncated.wrapping_add(1)
        } else {
            truncated
        };
        rounded as u32
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_boss_blue_fury_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BossBlueFuryState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(|| now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn expire_monster_boss_blue_fury_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    let Some((shape, state, tick)) = region
        .find_monster_by_id_mut(monster_id)
        .and_then(|monster| {
            let shape = monster.move_shape().shape().clone();
            let (state, tick) = monster.move_shape_mut().tick_boss_blue_fury_state(now_ms)?;
            if tick.release_control {
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
            }
            Some((shape, state, tick))
        })
    else {
        return false;
    };
    if tick.expired {
        send_owned_state_visual(game, region, &shape, state.skill_id(), false, 0, 0);
    }
    true
}

pub(crate) fn expire_player_boss_blue_fury_state<Runtime: crate::gameserver::gameserver::game::GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
    _runtime: &mut Runtime,
) -> bool {
    let Some((region_id, identity, tile_x, tile_y, state, tick)) = game
        .find_player_mut(player_id)
        .and_then(|player| {
            let region_id = player.server_region_id()?;
            let identity = player.shape().identity();
            let tile_x = player.shape().get_tile_x().ok()?;
            let tile_y = player.shape().get_tile_y().ok()?;
            let (state, tick) = player.tick_boss_blue_fury_state(now_ms)?;
            if tick.release_control {
                player.set_skill_moveable(true);
                player.set_skill_fightable(true);
            }
            Some((region_id, identity, tile_x, tile_y, state, tick))
        })
    else {
        return false;
    };
    if tick.expired {
        send_boss_blue_fury_state_visual(
            game, region_id, identity, tile_x, tile_y, state, false, now_ms,
        );
        let _ = game.update_player_properties(player_id);
    }
    true
}
