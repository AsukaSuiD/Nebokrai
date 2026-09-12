//! Временная ловкость CAgilityState2 (0x81).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/agilitystate2.cpp
//! и первичное наложение agility2.cpp. Постоянные DA/DB/DC — другое семейство.
//!
//! Наложение завершает первый непустой ID81 без RTTI/ended-фильтра и уничтожает
//! свежий остаток той же позиции. Только затем caller читает WORD full_miss
//! и длительность. Begin(U,U) читает часы и отправляет BFE03 до append;
//! loop=0 завершает visual после этого единственного сообщения. UpdateProperty
//! выполняется после попытки Begin независимо от результата.
//!
//! AI использует строгий абсолютный unsigned wrapping deadline, без death-gate.
//! End разрешает фактического S и удаляет тот же указатель: без visual и записи
//! ended. Отсутствующий либо чужой S не заменяется держателем арены.
//! Restart Begin(NULL, holder) сохраняет U/timestamp и заменяет S; SetRegion
//! меняет только регион U. OnUpdateProperties прибавляет WORD full_miss игроку
//! с переполнением, не вызывает visual и не читает часы.
//!
//! DB хранит DWORD ID, DWORD remaining и WORD full_miss. Getter остатка читает
//! одни либо двое часов; Unserialize — часы после внешнего ID, до remaining/WORD.
//! Общая арена сохраняет независимые записи и их позиции; безопасный массив
//! байтов заменяет исходный vector без изменения десятибайтового формата.

use super::agility2::AGILITY_2_SKILL_ID;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    end_and_destroy_state_at, remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
    update_player_state_properties, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const AGILITY_STATE_2_BYTES: usize = 10;

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

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(
            self.started_at_ms,
            self.keep_time_ms as u32,
            now_milliseconds,
        ) as i32
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != AGILITY_2_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let keep_time_ms = reader.read_i32()?;
        Ok(Self::new(reader.read_u16()?, now_ms, keep_time_ms))
    }

    pub(crate) fn encoded(self, now: impl FnMut() -> u32) -> [u8; AGILITY_STATE_2_BYTES] {
        let mut bytes = [0; AGILITY_STATE_2_BYTES];
        bytes[..4].copy_from_slice(&AGILITY_2_SKILL_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.client_time(now).to_le_bytes());
        bytes[8..].copy_from_slice(&self.full_miss.to_le_bytes());
        bytes
    }

    pub(crate) fn encoded_for_install(self) -> [u8; AGILITY_STATE_2_BYTES] {
        let mut bytes = [0; AGILITY_STATE_2_BYTES];
        bytes[..4].copy_from_slice(&AGILITY_2_SKILL_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.keep_time_ms.to_le_bytes());
        bytes[8..].copy_from_slice(&self.full_miss.to_le_bytes());
        bytes
    }
}

fn participant(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..shape.identity()
    }))
}

pub(crate) fn replace_agility_state_2(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce() -> AgilityState2, now: &mut dyn FnMut() -> u32,
) -> bool {
    let previous = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == AGILITY_2_SKILL_ID));
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, source.0, source.1, position);
    }
    let mut state = create();
    let begun = (|| {
        resolve_state_move_shape(game, source.0, source.1)?;
        state.started_at_ms = now();
        let user = participant(game, source)?;
        let sufferer = participant(game, source)?;
        if resolve_state_move_shape(game, sufferer.0, sufferer.1).is_some() {
            let mut message = CMessage::new(0x000b_fe03);
            message.add_long(sufferer.1.object_type);
            message.add_long(sufferer.1.id);
            message.add_ulong(state.skill_id());
            message.add_long(state.client_time(&mut *now));
            message.add_ulong(0);
            let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
        }
        let record = state.encoded_for_install();
        let shape = resolve_state_move_shape_mut(game, source.0, source.1)?;
        let key = shape.append_applied_state_record(state, &record);
        shape.mark_applied_state_begun(key);
        shape.set_applied_state_user(key, Some(user));
        shape.set_applied_state_sufferer(key, Some(sufferer));
        shape.begin_applied_state_visual(key, 0);
        shape.update_applied_state_visual_base(key);
        Some(())
    })().is_some();
    let _ = game.update_move_shape_properties(source.0, source.1);
    begun
}

pub(crate) fn update_agility_state_2_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    update_player_state_properties::<AgilityState2>(game, region_id, holder, key, |state, player| {
        player.update_state_combat_properties(|properties| state.apply_to_player(properties));
    })
}

pub(crate) fn restart_agility_state_2(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AgilityState2>(key)).is_none()
    { return false; }
    let Some(sufferer) = participant(game, (region_id, holder)) else { return false; };
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_applied_state_sufferer(key, Some(sufferer));
    }
    if begin_applied_state_visual(game, region_id, holder, key, 0) {
        update_property_state_visual::<AgilityState2>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.client_time(now) as u32,
        );
    }
    true
}

pub(crate) fn update_agility_state_2(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AgilityState2>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_agility_state_2(game, region_id, holder, key)
}

pub(crate) fn end_agility_state_2(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AgilityState2>(key)).is_none()
    { return false; }
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, AGILITY_STATE_2_BYTES)
}
