//! Состояние очищения CCureState (0x131), gameserver.exe + GameServer.pdb,
//! appserver/skills/curestate.cpp. Срок задаётся конструктором; нулевой срок
//! истекает только при started < now. DB-запись содержит ID и remaining,
//! без базовых identities. Load читает часы перед remaining.
//! Primary Begin проверяет S до часов, сохраняет U/S и публикует visual
//! до регистрации. Cure заменяет первый прежний экземпляр после нового Begin;
//! остальные producers сами определяют End прежнего состояния и добавляют новый в хвост.
//! End отправляет visual, заново разрешает S и удаляет только этот экземпляр
//! с обычным UpdateProperty.
//! Повторный Begin(NULL,S) не меняет timestamp.

pub(crate) const CURE_STATE_SKILL_ID: u32 = 305;
pub(crate) const CURE_STATE_BYTES: usize = 8;

use super::manashieldstate::MANA_SHIELD_STATE_BEGIN_MESSAGE;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    timed_client_state_time,
    resolve_state_move_shape, resolve_state_move_shape_mut, resolve_applied_state_sufferer,
    end_and_destroy_state_at, remove_applied_state_from, update_applied_state_end_visual,
    update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CureState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl CureState {
    pub(crate) const fn new(keep_time_ms: u32) -> Self {
        Self { started_at_ms: 0, keep_time_ms }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        CURE_STATE_SKILL_ID
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_state_time(self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != CURE_STATE_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self { started_at_ms: now_ms, keep_time_ms: reader.read_u32()? })
    }

    pub(crate) fn encoded(self, now: impl FnMut() -> u32) -> [u8; CURE_STATE_BYTES] {
        self.encoded_with_remaining(self.client_state_time(now))
    }

    pub(crate) fn encoded_for_install(self) -> [u8; CURE_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    fn encoded_with_remaining(self, remaining: u32) -> [u8; CURE_STATE_BYTES] {
        let mut bytes = [0; CURE_STATE_BYTES];
        bytes[..4].copy_from_slice(&CURE_STATE_SKILL_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&remaining.to_le_bytes());
        bytes
    }
}

/// Здесь Begin нового состояния предшествует поиску и End старого:
/// во время его visual новый экземпляр ещё не принадлежит вектору состояний.
pub(crate) fn begin_and_replace_cure_state(
    game: &mut CGame, user: Option<(i32, ShapeIdentity)>, sufferer: Option<(i32, ShapeIdentity)>,
    state: CureState, now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let target = sufferer?;
    begin_cure_state(game, target.0, target.1, user, sufferer, state, true, now)
}

/// Полный Begin и append без неявного поиска/End прежнего Cure.
pub(crate) fn begin_primary_cure_state(
    game: &mut CGame, holder_region: i32, holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>, sufferer: Option<(i32, ShapeIdentity)>,
    state: CureState, now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    begin_cure_state(game, holder_region, holder, user, sufferer, state, false, now)
}

#[allow(clippy::too_many_arguments, reason = "место регистрации и порядок замены задаются конкретным producer-ом")]
fn begin_cure_state(
    game: &mut CGame, holder_region: i32, holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>, sufferer: Option<(i32, ShapeIdentity)>,
    mut state: CureState, replace_previous: bool, now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    resolve_state_move_shape(game, holder_region, holder)?;
    resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
    if user.is_some() { state.started_at_ms = now(); }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity {
            ex_id: nebokrai_shared::values::CGuid::GUID_INVALID, ..shape.identity()
        }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let mut message = CMessage::new(MANA_SHIELD_STATE_BEGIN_MESSAGE);
    message.add_long(sufferer.1.object_type);
    message.add_long(sufferer.1.id);
    message.add_long(CURE_STATE_SKILL_ID as i32);
    message.add_ulong(state.client_state_time(&mut *now));
    message.add_long(0);
    let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
    let placement = if replace_previous {
        let shape = resolve_state_move_shape(game, holder_region, holder)?;
        let previous = shape.find_state_position(|state| state.state_id() == CURE_STATE_SKILL_ID);
        let placement = previous.and_then(|(_, key)| shape.applied_state_replacement_location(key));
        if let Some((index, _)) = previous {
            end_and_destroy_state_at(game, holder_region, holder, index)?;
        }
        placement
    } else { None };
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, holder_region, holder)?;
    let key = match placement {
        Some(location) => shape.insert_replacement_state_record(state, &record, location)?,
        None => shape.append_applied_state_record(state, &record),
    };
    shape.begin_applied_state_visual(key, 1);
    shape.update_applied_state_visual_base(key);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

pub(crate) fn restart_cure_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CureState>(key)).is_none()
    { return false; }
    if !crate::gameserver::appserver::states::state::begin_base_applied_state(
        game, region_id, holder, key,
    ) { return false; }
    if crate::gameserver::appserver::states::state::begin_applied_state_visual(
        game, region_id, holder, key, 1,
    ) {
        update_property_state_visual::<CureState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.client_state_time(now),
        );
    }
    true
}

pub(crate) fn end_player_cure_state(game: &mut CGame, player_id: i32) -> bool {
    let Some(key) = game.find_player(player_id)
        .and_then(|player| player.move_shape().cure_state_key()) else { return false };
    end_player_cure_state_key(game, player_id, key)
}

pub(crate) fn end_player_cure_state_key(game: &mut CGame, player_id: i32, key: StateKey) -> bool {
    let Some(player) = game.find_player(player_id) else { return false };
    let region_id = player.shape().get_region_id();
    let holder = player.shape().identity();
    end_cure_state_key(game, region_id, holder, key)
}


pub(crate) fn update_cure_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.cure_state_by_key(key))
        .is_some_and(|state| state.expired(now_ms));
    expired && end_cure_state_key(game, region_id, holder, key)
}

pub(crate) fn end_cure_state_key(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CureState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, CURE_STATE_BYTES)
}
