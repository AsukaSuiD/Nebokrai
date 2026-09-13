//! Постоянное сопротивление стихиям CTaiJiState.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/taijistate.cpp/.h.
//! Primary Begin(U,S) в immediatestateinstallation читает базовые часы.
//! Begin(NULL,S), включая DB-restart, отказывает до изменения базы/ended.
//! End отмечает ended, затем удаляет себя через свежий GetUser; NULL U
//! не заменяется держателем. SetRegion меняет только регион U.
//! Property читает свежую S без ended-gate: игрок складывает u16 gain с
//! сопротивлением как u32 и ограничивает i32::MAX, монстр прибавляет полный
//! i32 к модификатору. Итоговый monster getter сохраняет свои clamp/factor.
//! Default задаёт нулевой gain; DB8 — little-endian ID + i32 gain.
//! SlotMap хранит базу отдельно от payload. Собственных часов AI и visual нет.

use super::taiji::TAIJI_SKILL_ID;
use crate::gameserver::appserver::states::state::resolve_applied_state_sufferer;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_base_applied_state, resolve_state_move_shape};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const TAIJI_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TaiJiState {
    element_resistance_gain: i32,
}

impl TaiJiState {
    pub(crate) const fn new(element_resistance_gain: i32) -> Self {
        Self { element_resistance_gain }
    }

    pub(crate) const fn skill_id(self) -> u32 { TAIJI_SKILL_ID }
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != TAIJI_SKILL_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); }
        Ok(Self::new(reader.read_i32()?))
    }
    pub(crate) fn encoded(self) -> [u8; TAIJI_STATE_BYTES] {
        let mut bytes = [0; TAIJI_STATE_BYTES]; bytes[..4].copy_from_slice(&TAIJI_SKILL_ID.to_le_bytes()); bytes[4..].copy_from_slice(&self.element_resistance_gain.to_le_bytes()); bytes
    }
    pub(crate) const fn player_element_resistance_gain(self) -> u16 {
        self.element_resistance_gain as u16
    }

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.element_resistance = properties
            .element_resistance
            .wrapping_add(u32::from(self.player_element_resistance_gain()))
            .min(i32::MAX as u32);
        properties
    }


}

pub(crate) fn update_tai_ji_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TaiJiState>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        if let Some(monster) = game.find_region_mut(target_region)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.element_resistance = modifiers.element_resistance.wrapping_add(state.element_resistance_gain);
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|properties| state.apply_to_player(properties));
        }
    }
    true
}

pub(crate) fn restart_tai_ji_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    false
}

pub(crate) fn end_tai_ji_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TaiJiState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, TAIJI_STATE_BYTES)
}
