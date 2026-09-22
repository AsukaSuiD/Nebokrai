//! Постоянные состояния Agility/Natural/Rapture (0xda/0xdc/0xdb).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/agilitystate.cpp,
//! naturalstate.cpp и `rapturestate .cpp`. Типизированные варианты сохраняют разные
//! игровые величины: WORD full-miss, WORD прирост сопротивления стихиям и WORD
//! blast_attack. У Agility/Rapture сложение WORD с
//! переполнением; Natural складывает DWORD с переполнением, затем ограничивает
//! результат INT_MAX. OnUpdateProperties только разрешает S и применяет
//! player-формулу: visual, ended-gate и чтения часов отсутствуют.
//!
//! Наложение обходит живые слоты и удаляет все ID этого постоянного семейства,
//! не затрагивая временную Agility2. После каждого End уничтожается свежий
//! остаток той же позиции; пустые слоты не уплотняются. Только после обхода
//! caller читает новый коэффициент. Объектный Begin(U,U) читает часы,
//! сохраняет участников и публикует visual до append; UpdateProperty следует
//! после попытки Begin независимо от результата. Ненужный постоянному
//! состоянию timestamp не дублируется в payload.
//!
//! AI пустой; DB состоит из DWORD ID и WORD величины, без часов. Клиентские
//! время и дополнительные данные нулевые. End не пишет ended: существующий
//! visual получает Update(1) с базовым tail, затем свежий S удаляет именно
//! этот экземпляр. Чужой/NULL S не подменяется держателем арены. SetRegion
//! меняет только регион U; restart Begin(NULL, holder) сохраняет U и меняет S.
//! Общая арена, безопасный enum и шестибайтный массив заменяют указатели,
//! дублирующие классы и промежуточный вектор сериализации.

use super::agility::AGILITY_SKILL_ID;
use super::natural::NATURAL_SKILL_ID;
use super::rapture::RAPTURE_SKILL_ID;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    end_and_destroy_state_at, remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut, update_applied_state_end_visual,
    update_player_state_properties, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) const PERSISTENT_AGILITY_FAMILY_STATE_BYTES: usize = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersistentAgilityFamilyState {
    Agility { full_miss: u16 },
    Natural { element_resistance_gain: u16 },
    Rapture { blast_attack_gain: u16 },
}

impl PersistentAgilityFamilyState {
    pub(crate) const fn skill_id(self) -> u32 {
        match self {
            Self::Agility { .. } => AGILITY_SKILL_ID,
            Self::Natural { .. } => NATURAL_SKILL_ID,
            Self::Rapture { .. } => RAPTURE_SKILL_ID,
        }
    }

    pub(crate) const fn is_known_skill(skill_id: u32) -> bool {
        matches!(skill_id, AGILITY_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID)
    }

    pub(crate) fn apply_to_player(
        self, mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        match self {
            Self::Agility { full_miss } => {
                properties.full_miss = properties.full_miss.wrapping_add(full_miss);
            }
            Self::Natural { element_resistance_gain } => {
                properties.element_resistance = properties.element_resistance
                    .wrapping_add(u32::from(element_resistance_gain)).min(i32::MAX as u32);
            }
            Self::Rapture { blast_attack_gain } => {
                properties.blast_attack = properties.blast_attack.wrapping_add(blast_attack_gain);
            }
        }
        properties
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let value = reader.read_u16()?;
        match skill_id {
            AGILITY_SKILL_ID => Ok(Self::Agility { full_miss: value }),
            NATURAL_SKILL_ID => Ok(Self::Natural { element_resistance_gain: value }),
            RAPTURE_SKILL_ID => Ok(Self::Rapture { blast_attack_gain: value }),
            _ => Err(LegacyReadBlock {
                offset, needed: 4, available: payload.len().saturating_sub(offset),
            }),
        }
    }

    pub(crate) fn encoded(self) -> [u8; PERSISTENT_AGILITY_FAMILY_STATE_BYTES] {
        let value = match self {
            Self::Agility { full_miss } => full_miss,
            Self::Natural { element_resistance_gain } => element_resistance_gain,
            Self::Rapture { blast_attack_gain } => blast_attack_gain,
        };
        let mut bytes = [0; PERSISTENT_AGILITY_FAMILY_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id().to_le_bytes());
        bytes[4..].copy_from_slice(&value.to_le_bytes());
        bytes
    }
}

fn participant(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..shape.identity()
    }))
}

pub(crate) fn replace_persistent_agility_state(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce() -> PersistentAgilityFamilyState, now: &mut dyn FnMut() -> u32,
) -> bool {
    let mut index = 0;
    loop {
        let Some(shape) = resolve_state_move_shape(game, source.0, source.1) else { break; };
        if index >= shape.state_slot_count() { break; }
        if shape.state_at(index).is_some_and(|(_, state)| {
            PersistentAgilityFamilyState::is_known_skill(state.state_id())
        }) {
            let _ = end_and_destroy_state_at(game, source.0, source.1, index);
        }
        index += 1;
    }
    let state = create();
    let begun = (|| {
        resolve_state_move_shape(game, source.0, source.1)?;
        let _ = now();
        let user = participant(game, source)?;
        let sufferer = participant(game, source)?;
        if resolve_state_move_shape(game, sufferer.0, sufferer.1).is_some() {
            let mut message = CMessage::new(0x000b_fe03);
            message.add_long(sufferer.1.object_type);
            message.add_long(sufferer.1.id);
            message.add_ulong(state.skill_id());
            message.add_long(0);
            message.add_ulong(0);
            let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
        }
        let record = state.encoded();
        let shape = resolve_state_move_shape_mut(game, source.0, source.1)?;
        let key = shape.append_applied_state_record(state, &record);
        shape.mark_applied_state_begun(key);
        shape.set_applied_state_user(key, Some(user));
        shape.set_applied_state_sufferer(key, Some(sufferer));
        shape.begin_applied_state_visual(key, 1);
        shape.update_applied_state_visual_base(key);
        Some(())
    })().is_some();
    let _ = game.update_move_shape_properties(source.0, source.1);
    begun
}

pub(crate) fn update_persistent_agility_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    update_player_state_properties::<PersistentAgilityFamilyState>(
        game, region_id, holder, key, |state, player| {
            player.update_state_combat_properties(|properties| state.apply_to_player(properties));
        },
    )
}

pub(crate) fn restart_persistent_agility_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PersistentAgilityFamilyState>(key)).is_none()
    { return false; }
    let Some(sufferer) = participant(game, (region_id, holder)) else { return false; };
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_applied_state_sufferer(key, Some(sufferer));
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<PersistentAgilityFamilyState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |_, _| 0,
        );
    }
    true
}

pub(crate) fn end_persistent_agility_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PersistentAgilityFamilyState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(
        game, region_id, holder, key, target, PERSISTENT_AGILITY_FAMILY_STATE_BYTES,
    )
}
