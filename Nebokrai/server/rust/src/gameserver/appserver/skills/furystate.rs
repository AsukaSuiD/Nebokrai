//! Каноническое накапливаемое состояние `CFuryState` (`0x1a3`).
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает добавление без
//! добавление без замены, строгую проверку истечения и последовательное процентное
//! увеличение только максимальной атаки. Округление использует исходное
//! правило дробной части `> 0.5`, а визуальные начало/завершение сохраняют
//! `0xBFE03/04`.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const FURY_STATE_SKILL_ID: u32 = 0x1a3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FuryState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_gain_percent: i32,
}

impl FuryState {
    pub(crate) const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        attack_gain_percent: i32,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            attack_gain_percent,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        FURY_STATE_SKILL_ID
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms);
        if deadline <= now_ms {
            0
        } else {
            deadline.wrapping_sub(now_ms) as i32
        }
    }

    pub(crate) fn apply_to_monster_max_attack(self, maximum: u32) -> u32 {
        let scaled = self.attack_gain_percent as f32 * 0.01 * maximum as f32;
        let truncated = scaled.trunc() as i32;
        let gain = if scaled - truncated as f32 > 0.5 {
            truncated.wrapping_add(1)
        } else {
            truncated
        };
        let result = maximum.wrapping_add(gain as u32);
        if result as i32 <= 0 {
            1
        } else {
            result.min(i32::MAX as u32)
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_fury_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: FuryState,
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

pub(crate) fn expire_monster_fury_states(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> usize {
    let expired = region.find_monster_by_id_mut(monster_id).map(|monster| {
        let identity = monster.move_shape().shape().identity();
        let tile_x = monster
            .move_shape()
            .shape()
            .get_tile_x()
            .unwrap_or_default();
        let tile_y = monster
            .move_shape()
            .shape()
            .get_tile_y()
            .unwrap_or_default();
        let states = monster
            .move_shape_mut()
            .take_expired_fury_states(now_ms);
        (identity, tile_x, tile_y, states)
    });
    let Some((identity, tile_x, tile_y, states)) = expired else {
        return 0;
    };
    let count = states.len();
    for state in states {
        send_fury_state_visual(
            game, region.id, identity, tile_x, tile_y, state, false, now_ms,
        );
    }
    count
}

// Статус оставшегося корпуса: PARTIALLY_IMPLEMENTED
// Не достигнуты ветвь игрока, устаревшая сериализация и координатные перегрузки.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp

// ============================================================================
// FUNCTION: CFuryState::CFuryState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:15
// RVA: 0x001EA2E0
// ADDRESS: 005ea2e0
// PROTOTYPE: undefined __thiscall CFuryState(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryState::~CFuryState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:37
// RVA: 0x001EA360
// ADDRESS: 005ea360
// PROTOTYPE: void __thiscall ~CFuryState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:93
// RVA: 0x001EA370
// ADDRESS: 005ea370
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:103
// RVA: 0x001EA410
// ADDRESS: 005ea410
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryState::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Истечение состояния монстра достигнуто в `expire_monster_fury_states`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:127
// RVA: 0x001EA4C0
// ADDRESS: 005ea4c0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryState::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектная ветвь монстра достигнута через `FuryState::new` и
// `send_fury_state_visual`; ветвь игрока остаётся неподключённой.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:83
// RVA: 0x001EA500
// ADDRESS: 005ea500
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryStateVisualEffect::UpdateVisualEffect
// STATUS: IMPLEMENTED
// Реализовано функцией `send_fury_state_visual`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:170
// RVA: 0x001EA5A0
// ADDRESS: 005ea5a0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)

// ============================================================================
// FUNCTION: CFuryState::OnUpdateProperties
// STATUS: PARTIALLY_IMPLEMENTED
// Ветка монстра реализована `FuryState::apply_to_monster_max_attack`;
// отличающаяся ветвь игрока сохранена ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:48
// RVA: 0x001FD480
// ADDRESS: 005fd480
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:156
// RVA: 0x001FD660
// ADDRESS: 005fd660
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryState::End
// STATUS: PARTIALLY_IMPLEMENTED
// Завершение монстра выполняет `expire_monster_fury_states`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:113
// RVA: 0x002059A0
// ADDRESS: 006059a0
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryState::GetRemainedTime
// STATUS: IMPLEMENTED
// Реализовано методом `FuryState::client_time`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:41
// RVA: 0x00205E10
// ADDRESS: 00605e10
// PROTOTYPE: ulong __thiscall GetRemainedTime(void)




// COMPONENT_VARIANT_END: GameServer
