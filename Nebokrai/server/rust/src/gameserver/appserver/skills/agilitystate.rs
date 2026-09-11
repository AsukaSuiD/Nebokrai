//! Каноническое постоянное состояние `CAgilityState`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `agilitystate.cpp`. Состояние `0xda` публикует начало/завершение и добавляет
//! `full_miss` сложением с переполнением. Временным состоянием `0x81` владеет
//! отдельный `agilitystate2.rs`; все экземпляры принадлежат общей арене
//! `CanonicalStateStorage` и сохраняют порядок повторных DB-записей.
//! Три постоянных варианта используют общую шестибайтную DB-запись `ID + WORD`.
//! Vtable Agility/Natural/Rapture 0x00660774/0x006606B4/0x00660714:
//! End +0x1C→0x005FD420 публикует visual phase1, затем GetSufferer и
//! RemoveState. Он не вызывает базовый End и не пишет IsEnded.
//! Удаляется достигнутый ключ; UpdateProperty следует за RemoveState.

use super::agility::AGILITY_SKILL_ID;
use super::natural::NATURAL_SKILL_ID;
use super::naturalstate::NaturalState;
use super::rapture::RAPTURE_SKILL_ID;
use super::rapturestate::RaptureState;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const AGILITY_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const AGILITY_STATE_END_MESSAGE: i32 = 0x000b_fe04;
pub(crate) const PERSISTENT_AGILITY_FAMILY_STATE_BYTES: usize = 6;

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

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let value = reader.read_u16()?;
        match skill_id {
            AGILITY_SKILL_ID => Ok(Self::Agility(AgilityState::persistent(value))),
            NATURAL_SKILL_ID => Ok(Self::Natural(NaturalState::new(value))),
            RAPTURE_SKILL_ID => Ok(Self::Rapture(RaptureState::new(value))),
            _ => Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }),
        }
    }

    pub(crate) fn encoded(self) -> [u8; PERSISTENT_AGILITY_FAMILY_STATE_BYTES] {
        let value = match self {
            Self::Agility(state) => state.full_miss(),
            Self::Natural(state) => state.element_resistance_gain(),
            Self::Rapture(state) => state.blast_attack_gain(),
        };
        let mut bytes = Vec::with_capacity(PERSISTENT_AGILITY_FAMILY_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(self.skill_id());
        writer.write_u16(value);
        bytes.try_into().expect("размер постоянного состояния ловкости фиксирован")
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

pub(crate) fn end_persistent_agility_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PersistentAgilityFamilyState>(key)).copied()
        else { return false };
    let mut message = CMessage::new(AGILITY_STATE_END_MESSAGE);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<PersistentAgilityFamilyState>(key, PERSISTENT_AGILITY_FAMILY_STATE_BYTES))
        .is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    removed
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранены посторонний недостигнутый helper и конструктор по умолчанию.

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


// COMPONENT_VARIANT_END: GameServer
