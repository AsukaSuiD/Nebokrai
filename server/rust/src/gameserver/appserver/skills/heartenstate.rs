//! CHeartenState (0x144), gameserver.exe/GameServer.pdb,
//! appserver/skills/heartenstate.cpp.
//!
//! Первичный Begin записывает часы только при ненулевом U и создаёт loop1
//! visual без пакета. Пакет начала отправляется при каждом пересчёте свойств
//! actual S; лишь игрок получает wrapping-прибавку maximum HP с пределом
//! INT_MAX. После End visual цель разрешается заново для RemoveState.
//! Перезапуск с NULL U сохраняет часы и прежнего пользователя.
//! DB-запись содержит ID, остаток срока и знаковую прибавку HP; загрузка
//! читает часы перед полями, клиентский остаток — два живых чтения часов.
//! Payload и DB-span принадлежат одной записи общей арены.

use super::hearten::HEARTEN_SKILL_ID;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut, timed_client_state_time, update_applied_state_end_visual,
    update_player_state_properties, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

pub(crate) const HEARTEN_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct HeartenState {
    started_at_ms: u32,
    keep_time_ms: u32,
    max_hp_gain: i32,
}

impl HeartenState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, max_hp_gain: i32) -> Self {
        Self { started_at_ms, keep_time_ms, max_hp_gain }
    }

    pub(crate) const fn skill_id(&self) -> u32 { HEARTEN_SKILL_ID }

    pub(crate) const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(&self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) const fn apply(&self, value: u32) -> u32 {
        let result = value.wrapping_add(self.max_hp_gain as u32);
        if result > i32::MAX as u32 { i32::MAX as u32 } else { result }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != HEARTEN_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?))
    }

    pub(crate) fn encoded(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; HEARTEN_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(HEARTEN_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(HEARTEN_SKILL_ID);
        writer.write_u32(self.client_time(now_milliseconds) as u32);
        writer.write_i32(self.max_hp_gain);
        bytes.try_into().expect("размер состояния воодушевления фиксирован")
    }

    pub(crate) fn encoded_for_install(&self) -> [u8; HEARTEN_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(HEARTEN_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(HEARTEN_SKILL_ID);
        writer.write_u32(self.keep_time_ms);
        writer.write_i32(self.max_hp_gain);
        bytes.try_into().expect("размер состояния воодушевления фиксирован")
    }
}

pub(crate) fn begin_primary_hearten_state(
    game: &mut CGame,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: (i32, ShapeIdentity),
    mut state: HeartenState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    if user.is_some() { state.started_at_ms = now(); }
    let participant = |(region, identity)| {
        let shape = resolve_state_move_shape(game, region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, sufferer.0, sufferer.1)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    // Loop1 создаёт общий каталог. Между Begin и append нет внешнего callback;
    // первый UpdateVisualEffect принадлежит последующему UpdateProperty.
    Some(key)
}

pub(crate) fn update_hearten_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
    key: StateKey, now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, sufferer)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<HeartenState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    if sufferer.object_type != 400 { return true; }
    update_player_state_properties::<HeartenState>(game, region_id, holder, key, |state, player| {
        player.update_state_combat_properties(|mut properties| {
            properties.maximum_hp = state.apply(properties.maximum_hp);
            properties
        });
    })
}

pub(crate) fn restart_hearten_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HeartenState>(key)).is_none()
    { return false; }
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_hearten_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HeartenState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_hearten_state(game, region_id, holder, key)
}

pub(crate) fn end_hearten_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<HeartenState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(sufferer) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, sufferer, HEARTEN_STATE_BYTES)
}
