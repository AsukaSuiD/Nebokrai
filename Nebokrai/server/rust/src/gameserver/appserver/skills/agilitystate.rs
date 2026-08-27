//! Канонические состояния `CAgilityState/CAgilityState2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `agilitystate.cpp` и `agilitystate2.cpp`. Постоянное состояние `0xda`
//! публикует begin/end, временное `0x81` публикует только begin и завершается
//! при строгом `started + keep < now`. Оба добавляют `full_miss` сложением
//! с переполнением при общем пересчёте свойств. Поля и часы принадлежат
//! `CanonicalStateStorage`; сырой сохранённый псевдокод оставлен ниже.

use super::agility::{AGILITY_2_SKILL_ID, AGILITY_SKILL_ID};
use super::natural::NATURAL_SKILL_ID;
use super::naturalstate::NaturalState;
use super::rapture::RAPTURE_SKILL_ID;
use super::rapturestate::RaptureState;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const AGILITY_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const AGILITY_STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AgilityState {
    skill_id: u32,
    full_miss: u16,
    started_at_ms: u32,
    keep_time_ms: i32,
}

impl AgilityState {
    pub(crate) const fn persistent(full_miss: u16) -> Self {
        Self {
            skill_id: AGILITY_SKILL_ID,
            full_miss,
            started_at_ms: 0,
            keep_time_ms: 0,
        }
    }

    pub(crate) const fn timed(full_miss: u16, started_at_ms: u32, keep_time_ms: i32) -> Self {
        Self {
            skill_id: AGILITY_2_SKILL_ID,
            full_miss,
            started_at_ms,
            keep_time_ms,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 { self.skill_id }
    pub(crate) const fn full_miss(self) -> u16 { self.full_miss }
    pub(crate) const fn is_timed(self) -> bool { self.skill_id == AGILITY_2_SKILL_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.is_timed() && self.started_at_ms.wrapping_add(self.keep_time_ms as u32) < now_ms
    }
    pub(crate) const fn client_time_needs_second_clock(self, first_now_ms: u32) -> bool {
        self.is_timed()
            && first_now_ms < self.started_at_ms.wrapping_add(self.keep_time_ms as u32)
    }
    pub(crate) const fn client_time(self, first_now_ms: u32, second_now_ms: u32) -> i32 {
        if !self.is_timed() || self.started_at_ms.wrapping_add(self.keep_time_ms as u32) <= first_now_ms {
            0
        } else {
            self.started_at_ms.wrapping_sub(second_now_ms).wrapping_add(self.keep_time_ms as u32) as i32
        }
    }
}

/// Одно из трёх взаимно исключающих постоянных состояний семейства.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersistentAgilityFamilyState {
    Agility(AgilityState),
    Natural(NaturalState),
    Rapture(RaptureState),
}

impl PersistentAgilityFamilyState {
    pub(crate) const fn skill_id(self) -> u32 {
        match self {
            Self::Agility(state) => state.skill_id(),
            Self::Natural(state) => state.skill_id(),
            Self::Rapture(state) => state.skill_id(),
        }
    }

    pub(crate) const fn is_known_skill(skill_id: u32) -> bool {
        matches!(
            skill_id,
            AGILITY_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID
        )
    }
}

pub(crate) fn send_agility_family_state_visual(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    begin: bool,
    client_time: i32,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(if begin {
        AGILITY_STATE_BEGIN_MESSAGE
    } else {
        AGILITY_STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(skill_id as i32);
    if begin {
        message.add_long(client_time);
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранены посторонний недостигнутый helper, конструктор по умолчанию и сериализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agilitystate.cpp

// ============================================================================
// FUNCTION: CS2CContainerObjectMove::SetSourceContainerExtendID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agilitystate.cpp:148
// RVA: 0x001D9BA0
// ADDRESS: 005d9ba0
// PROTOTYPE: void __thiscall SetSourceContainerExtendID(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CAgilityState::CAgilityState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agilitystate.cpp:24
// RVA: 0x001F4140
// ADDRESS: 005f4140
// PROTOTYPE: undefined __thiscall CAgilityState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CAgilityState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agilitystate.cpp:156
// RVA: 0x001F3E40
// ADDRESS: 005f3e40
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CAgilityState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agilitystate.cpp:166
// RVA: 0x001F4420
// ADDRESS: 005f4420
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
