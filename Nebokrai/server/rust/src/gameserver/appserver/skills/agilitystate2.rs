//! Каноническое временное состояние `CAgilityState2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `agilitystate2.cpp`. Состояние `0x81` добавляет `full_miss` сложением с
//! переполнением, завершается только при строгом `started + keep < now` и при
//! вычислении положительного клиентского остатка второй раз читает часы.
//! Жизненный цикл принадлежит `CanonicalStateStorage`; legacy-сериализация пока
//! не подключена и сохранена ниже как RAW.

use super::agility2::AGILITY_2_SKILL_ID;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::gameserver::game::CGame;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AgilityState2 {
    full_miss: u16,
    started_at_ms: u32,
    keep_time_ms: i32,
}

impl AgilityState2 {
    pub(crate) const fn new(full_miss: u16, started_at_ms: u32, keep_time_ms: i32) -> Self {
        Self {
            full_miss,
            started_at_ms,
            keep_time_ms,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 { AGILITY_2_SKILL_ID }

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.full_miss = properties.full_miss.wrapping_add(self.full_miss);
        properties
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms as u32) < now_ms
    }

    pub(crate) const fn client_time_needs_second_clock(self, first_now_ms: u32) -> bool {
        first_now_ms < self.started_at_ms.wrapping_add(self.keep_time_ms as u32)
    }

    pub(crate) const fn client_time(self, first_now_ms: u32, second_now_ms: u32) -> i32 {
        if self.started_at_ms.wrapping_add(self.keep_time_ms as u32) <= first_now_ms {
            0
        } else {
            self.started_at_ms
                .wrapping_sub(second_now_ms)
                .wrapping_add(self.keep_time_ms as u32) as i32
        }
    }
}

pub(crate) fn expire_player_agility_state_2(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    let ended = game
        .find_player_mut(player_id)
        .and_then(|player| player.take_expired_agility_state_2(now_ms))
        .is_some();
    if ended {
        let _ = game.publish_player_states(player_id);
    }
    ended
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранены неподключённые legacy-сериализация и обратное чтение состояния.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agilitystate2.cpp

// ============================================================================
// FUNCTION: CAgilityState2::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agilitystate2.cpp:158
// RVA: 0x001F1050
// ADDRESS: 005f1050
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAgilityState2::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agilitystate2.cpp:171
// RVA: 0x001F48E0
// ADDRESS: 005f48e0
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
