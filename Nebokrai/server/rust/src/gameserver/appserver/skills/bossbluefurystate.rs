//! Каноническое состояние ярости синего босса `CBossBlueFuryState` (`0x1f7`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluefurystate.cpp`. Достигнутый путь монстра заменяет
//! прежнее состояние до установки нового, запрещает движение и бой на слабой
//! фазе, а затем сохраняет усиление до общего срока. Минимальная и максимальная
//! атака заменяются указанной долей коэффициента с исходным округлением дробной части
//! строго больше `0.5`; визуальные начало и завершение сохраняют `0xBFE03/04`.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp

// ============================================================================
// FUNCTION: CBossBlueFuryState::Unserialize
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутый путь монстра хранит те же длительность и коэффициент в
// `BossBlueFuryState`; чтение сохранённого состояния остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:190
// RVA: 0x001D6190
// ADDRESS: 005d6190
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Serialize
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутый путь монстра выражен `BossBlueFuryState::client_time` и
// каноническим хранилищем; сохранение игрока остаётся неподключённым.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:177
// RVA: 0x001E7330
// ADDRESS: 005e7330
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
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
// Завершение достигнутого пути выполняет `expire_monster_boss_blue_fury_state`.
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
// Завершение достигнутого пути выполняет `expire_monster_boss_blue_fury_state`
// и ветвь замены в `execute_owned_boss_blue_fury`.
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
// отличающаяся ветвь игрока отсутствует в достигнутом графе вызовов.
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
// Объектная ветвь монстра выполняется `BossBlueFuryState::new` и каноническим
// владельцем в `CMoveShape`.
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
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const BOSS_BLUE_FURY_STATE_ID: u32 = 0x1f7;

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

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms);
        if deadline <= now_ms {
            0
        } else {
            deadline.wrapping_sub(now_ms) as i32
        }
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
        message.add_long(state.client_time(now_ms));
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
    let Some((identity, tile_x, tile_y, state, tick)) = region
        .find_monster_by_id_mut(monster_id)
        .and_then(|monster| {
            let identity = monster.move_shape().shape().identity();
            let tile_x = monster.move_shape().shape().get_tile_x().unwrap_or_default();
            let tile_y = monster.move_shape().shape().get_tile_y().unwrap_or_default();
            let (state, tick) = monster.move_shape_mut().tick_boss_blue_fury_state(now_ms)?;
            if tick.release_control {
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
            }
            Some((identity, tile_x, tile_y, state, tick))
        })
    else {
        return false;
    };
    if tick.expired {
        send_boss_blue_fury_state_visual(
            game, region.id, identity, tile_x, tile_y, state, false, now_ms,
        );
    }
    true
}
