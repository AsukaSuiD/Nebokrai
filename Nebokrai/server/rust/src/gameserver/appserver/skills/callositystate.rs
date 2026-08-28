//! Каноническое состояние пары `CCallosityState/CCallosityState2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `appserver/skills/callositystate.cpp` и `callositystate2.cpp`. Состояния
//! взаимно исключают друг друга и хранятся одним типизированным владельцем.
//! Подтверждённая
//! странность сохранена: `time_to_keep` не обслуживается отдельным `AI`, а
//! унаследованные `GetClientStateTime/GetAdditionalData` возвращают нули.
//! Коэффициент `CCH` применяется только при общем `UpdateProperty`; каждый
//! такой проход повторно публикует начальный визуальный эффект, как
//! `OnUpdateProperties`.

use super::callosity::{CALLOSITY_2_SKILL_ID, CALLOSITY_SKILL_ID};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const CALLOSITY_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallosityState {
    skill_id: u32,
    blast_factor: u16,
    time_to_keep: i32,
}

impl CallosityState {
    pub(crate) const fn new(skill_id: u32, blast_factor: u16, time_to_keep: i32) -> Self {
        debug_assert!(skill_id == CALLOSITY_SKILL_ID || skill_id == CALLOSITY_2_SKILL_ID);
        Self {
            skill_id,
            blast_factor,
            time_to_keep,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub(crate) const fn blast_factor(self) -> u16 {
        self.blast_factor
    }

    pub(crate) const fn time_to_keep(self) -> i32 {
        self.time_to_keep
    }

    pub(crate) const fn client_state_time(self) -> i32 {
        0
    }

    pub(crate) const fn additional_data(self) -> u32 {
        0
    }

    pub(crate) const fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.cch = properties.cch.wrapping_add(self.blast_factor);
        properties
    }
}

pub(crate) fn send_callosity_state_begin(
    game: &mut CGame,
    player_id: i32,
    state: CallosityState,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(CALLOSITY_STATE_BEGIN_MESSAGE);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    message.add_long(state.client_state_time());
    message.add_ulong(state.additional_data());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp

// ============================================================================
// FUNCTION: CCallosityState::CCallosityState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:25
// RVA: 0x001F4600
// ADDRESS: 005f4600
// PROTOTYPE: undefined __thiscall CCallosityState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
