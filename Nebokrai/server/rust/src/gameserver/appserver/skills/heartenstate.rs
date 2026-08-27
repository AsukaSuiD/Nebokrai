//! Каноническое достигнутое состояние `CHeartenState`.
//!
//! Состояние `324` хранит wrapping-часы и прибавляет знаковый параметр к
//! максимальному HP через `u32`, затем ограничивает результат `i32::MAX`.
//! Начальный визуальный пакет повторяется при каждом пересчёте свойств;
//! завершение публикуется при замене или строгом истечении срока.

use super::hearten::HEARTEN_SKILL_ID;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const HEARTEN_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const HEARTEN_STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeartenState {
    started_at_ms: u32,
    keep_time_ms: u32,
    max_hp_gain: i32,
}

impl HeartenState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, max_hp_gain: i32) -> Self {
        Self { started_at_ms, keep_time_ms, max_hp_gain }
    }
    pub(crate) const fn skill_id(self) -> u32 { HEARTEN_SKILL_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }
    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms);
        if deadline <= now_ms { 0 } else { deadline.wrapping_sub(now_ms) as i32 }
    }
    pub(crate) const fn apply(self, value: u32) -> u32 {
        let result = value.wrapping_add(self.max_hp_gain as u32);
        if result > i32::MAX as u32 { i32::MAX as u32 } else { result }
    }
}

pub(crate) fn send_hearten_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: HeartenState,
    begin: bool,
    now_ms: u32,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(if begin {
        HEARTEN_STATE_BEGIN_MESSAGE
    } else {
        HEARTEN_STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_ms));
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранены только не подключённые конструктор по умолчанию и сериализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\heartenstate.cpp

// ============================================================================
// FUNCTION: CHeartenState::CHeartenState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\heartenstate.cpp:25
// RVA: 0x001EE580
// ADDRESS: 005ee580
// PROTOTYPE: undefined __thiscall CHeartenState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CHeartenState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\heartenstate.cpp:133
// RVA: 0x001D4D10
// ADDRESS: 005d4d10
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CHeartenState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\heartenstate.cpp:146
// RVA: 0x000F9D80
// ADDRESS: 004f9d80
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
