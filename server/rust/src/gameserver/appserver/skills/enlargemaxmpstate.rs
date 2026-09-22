//! Постоянное увеличение максимума MP CEnlargeMaxMpState.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/enlargemaxmpstate.cpp/.h.
//! Property читает свежую S без ended-gate; только игрок получает u32
//! wrapping-add gain к максимуму MP с ограничением i32::MAX.
//! Primary Begin(U,S) в immediatestateinstallation читает базовые часы;
//! DB-restart Begin(NULL,S) сохраняет U/timestamp, обновляет S и снимает ended.
//! End отмечает ended и удаляет себя через свежий U, не подставляя держателя;
//! SetRegion меняет только регион U. SlotMap хранит базу отдельно от payload.
//! Default задаёт нулевой gain; DB8 — little-endian ID + i32 gain.
//! Собственных visual и часов AI нет.

use super::enlargemaxmp::ENLARGE_MAX_MP_SKILL_ID;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub(crate) const ENLARGE_MAX_MP_STATE_BYTES: usize = 8;

/// OnUpdateProperties 0x005E22A0: GetSufferer, затем только player-формула.
/// Visual, IsEnded-gate и чтения часов у этого override отсутствуют.
pub(crate) fn update_enlarge_max_mp_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    crate::gameserver::appserver::states::state::update_player_state_properties::<EnlargeMaxMpState>(
        game, region_id, holder, key, |state, player| {
            player.update_state_combat_properties(|mut properties| { properties.maximum_mp = state.apply(properties.maximum_mp); properties });
        },
    )
}

pub(crate) fn restart_enlarge_max_mp_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnlargeMaxMpState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
}

pub(crate) fn end_enlarge_max_mp_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnlargeMaxMpState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, ENLARGE_MAX_MP_STATE_BYTES)
}


#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EnlargeMaxMpState {
    gain: i32,
}

impl EnlargeMaxMpState {
    pub(crate) const fn new(gain: i32) -> Self {
        Self { gain }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        ENLARGE_MAX_MP_SKILL_ID
    }
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> { let mut reader = LegacyReader::at(payload, offset)?; if reader.read_u32()? != ENLARGE_MAX_MP_SKILL_ID { return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) }); } Ok(Self::new(reader.read_i32()?)) }
    pub(crate) fn encoded(self) -> [u8; ENLARGE_MAX_MP_STATE_BYTES] { let mut bytes = [0; ENLARGE_MAX_MP_STATE_BYTES]; bytes[..4].copy_from_slice(&ENLARGE_MAX_MP_SKILL_ID.to_le_bytes()); bytes[4..].copy_from_slice(&self.gain.to_le_bytes()); bytes }

    pub(crate) const fn apply(self, value: u32) -> u32 {
        let result = value.wrapping_add(self.gain as u32);
        if result > i32::MAX as u32 {
            i32::MAX as u32
        } else {
            result
        }
    }
}
