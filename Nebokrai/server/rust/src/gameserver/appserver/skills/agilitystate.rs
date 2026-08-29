//! Каноническое постоянное состояние `CAgilityState`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `agilitystate.cpp`. Состояние `0xda` публикует начало/завершение и добавляет
//! `full_miss` сложением с переполнением. Временным состоянием `0x81` владеет
//! отдельный `agilitystate2.rs`; оба slot-а принадлежат `CanonicalStateStorage`.

use super::agility::AGILITY_SKILL_ID;
use super::natural::NATURAL_SKILL_ID;
use super::naturalstate::NaturalState;
use super::rapture::RAPTURE_SKILL_ID;
use super::rapturestate::RaptureState;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const AGILITY_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const AGILITY_STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AgilityState {
    full_miss: u16,
}

impl AgilityState {
    pub(crate) const fn persistent(full_miss: u16) -> Self {
        Self {
            full_miss,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 { AGILITY_SKILL_ID }
    pub(crate) const fn full_miss(self) -> u16 { self.full_miss }
    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.full_miss = properties.full_miss.wrapping_add(self.full_miss);
        properties
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

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        match self {
            Self::Agility(state) => {
                properties = state.apply_to_player(properties);
            }
            Self::Natural(state) => {
                properties.element_resistance = properties
                    .element_resistance
                    .wrapping_add(u32::from(state.element_resistance_gain()))
                    .min(i32::MAX as u32);
            }
            Self::Rapture(state) => {
                properties.blast_attack = properties
                    .blast_attack
                    .wrapping_add(state.blast_attack_gain());
            }
        }
        properties
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
