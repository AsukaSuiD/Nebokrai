//! Каноническое краткоживущее состояние `CCureState`.
//!
//! Источник: `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/curestate.cpp`. Класс не переопределяет `AI`, поэтому
//! унаследованный `CState::AI` завершает его на следующем снимке
//! `UpdateAbnormality`; состояние успевает участвовать в `OnChangeStates`.

pub(crate) const CURE_STATE_SKILL_ID: u32 = 305;

use super::manashieldstate::{
    MANA_SHIELD_STATE_BEGIN_MESSAGE, MANA_SHIELD_STATE_END_MESSAGE,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CureState {
    keep_time_ms: u32,
}

impl CureState {
    pub(crate) const fn new(keep_time_ms: u32) -> Self {
        Self { keep_time_ms }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        CURE_STATE_SKILL_ID
    }

    pub(crate) const fn client_time(self) -> i32 {
        let _ = self.keep_time_ms;
        0
    }
}

pub(crate) fn send_cure_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: CureState,
    begin: bool,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(if begin {
        MANA_SHIELD_STATE_BEGIN_MESSAGE
    } else {
        MANA_SHIELD_STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(CURE_STATE_SKILL_ID as i32);
    if begin {
        message.add_long(state.client_time());
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\curestate.cpp

// ============================================================================
// FUNCTION: CCureState::CCureState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\curestate.cpp:24
// RVA: 0x001E9EC0
// ADDRESS: 005e9ec0
// PROTOTYPE: undefined __thiscall CCureState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
