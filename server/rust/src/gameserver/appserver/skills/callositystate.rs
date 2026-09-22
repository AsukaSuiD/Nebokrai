//! Состояния закалки CCallosityState/CCallosityState2 (0x75/0x7d).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/callositystate.cpp
//! и callositystate2.cpp. Оба варианта имеют одинаковые поля, lifecycle,
//! формулу и wire; различается только ID. Одна структура и общая арена
//! заменяют дублирующие владельцы, не схлопывая загруженные экземпляры.
//!
//! Наложение находит первый непустой слот с любым из двух ID без RTTI/ended-фильтра,
//! вызывает End и уничтожает свежий остаток той же позиции. Только затем caller
//! вычисляет длительность и WORD-коэффициент. Объектный Begin читает часы,
//! сохраняет U/S и запускает visual до append; UpdateProperty вызывается после
//! попытки Begin независимо от её результата. OnUpdateProperties сначала
//! захватывает S, вызывает существующий visual и прибавляет WORD blast_attack именно
//! захваченному игроку, даже если сам visual завершён.
//!
//! AI проверяет строгий абсолютный unsigned wrapping deadline без death-gate.
//! End не записывает ended: visual Update(1) с базовым tail, затем свежий GetS
//! и RemoveState того же указателя. Удаление чужого либо отсутствующего S
//! не подменяется удалением записи из арены держателя. SetRegion меняет только
//! регион U; restart Begin(NULL, holder) сохраняет timestamp/U и заменяет S.
//!
//! DB: DWORD ID, DWORD remaining, WORD factor. GetRemainedTime выполняет
//! одно либо два чтения часов; Unserialize читает часы после внешнего ID, до полей
//! remaining/factor. Начальный visual и property-update передают BFE03
//! (S type/id, state ID, remaining, ноль), завершение — BFE04 (S type/id, ID).
//! Границы чтения и записи задают общий codec и безопасные массивы байтов.

use super::callosity::CALLOSITY_SKILL_ID;
use super::callosity2::CALLOSITY_2_SKILL_ID;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    default_additional_data, end_and_destroy_state_at, remove_applied_state_from,
    resolve_applied_state_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
    timed_client_state_time, update_applied_state_end_visual, update_property_state_visual,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;

pub(crate) const CALLOSITY_STATE_BYTES: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallosityFamilyState {
    skill_id: u32,
    blast_factor: u16,
    started_at_ms: u32,
    time_to_keep: i32,
}

impl CallosityFamilyState {
    pub(crate) const fn new(
        skill_id: u32, blast_factor: u16, started_at_ms: u32, time_to_keep: i32,
    ) -> Self {
        Self { skill_id, blast_factor, started_at_ms, time_to_keep }
    }

    pub(crate) const fn skill_id(self) -> u32 { self.skill_id }
    pub(crate) const fn blast_factor(self) -> u16 { self.blast_factor }
    pub(crate) const fn time_to_keep(self) -> i32 { self.time_to_keep }
    pub(crate) const fn additional_data(self) -> u32 { default_additional_data() }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.time_to_keep as u32) < now_ms
    }

    pub(crate) fn client_state_time(self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.time_to_keep as u32, now) as i32
    }

    pub(crate) fn decode(
        payload: &[u8], offset: usize, now_ms: u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        if !matches!(skill_id, CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID) {
            return Err(LegacyReadBlock {
                offset, needed: 4, available: payload.len().saturating_sub(offset),
            });
        }
        let time_to_keep = reader.read_i32()?;
        let blast_factor = reader.read_u16()?;
        Ok(Self::new(skill_id, blast_factor, now_ms, time_to_keep))
    }

    pub(crate) fn encoded(self, now: impl FnMut() -> u32) -> [u8; CALLOSITY_STATE_BYTES] {
        let mut bytes = [0; CALLOSITY_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.client_state_time(now).to_le_bytes());
        bytes[8..].copy_from_slice(&self.blast_factor.to_le_bytes());
        bytes
    }

    pub(crate) fn encoded_for_install(self) -> [u8; CALLOSITY_STATE_BYTES] {
        let mut bytes = [0; CALLOSITY_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.time_to_keep.to_le_bytes());
        bytes[8..].copy_from_slice(&self.blast_factor.to_le_bytes());
        bytes
    }

    pub(crate) const fn apply_to_player(
        self, mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.blast_attack = properties.blast_attack.wrapping_add(self.blast_factor);
        properties
    }
}

fn participant(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    Some((shape.get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..shape.identity()
    }))
}

pub(crate) fn replace_callosity_state(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce() -> CallosityFamilyState, now: &mut dyn FnMut() -> u32,
) -> bool {
    let previous = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| {
            matches!(state.state_id(), CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID)
        }));
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
            message.add_ulong(state.skill_id);
            message.add_long(state.client_state_time(&mut *now));
            message.add_ulong(0);
            let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
        }
        let record = state.encoded_for_install();
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

pub(crate) fn end_callosity_state_key(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).is_none()
    { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, CALLOSITY_STATE_BYTES)
}

pub(crate) fn update_callosity_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<CallosityFamilyState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_state_time(now) as u32,
    );
    if target.object_type == 400 {
        let Some(state) = resolve_state_move_shape(game, region_id, holder)
            .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).copied()
        else { return false; };
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|properties| state.apply_to_player(properties));
        }
    }
    true
}

pub(crate) fn restart_callosity_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key)).is_none()
    { return false; }
    let Some(sufferer) = participant(game, (region_id, holder)) else { return false; };
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_applied_state_sufferer(key, Some(sufferer));
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<CallosityFamilyState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.client_state_time(now) as u32,
        );
    }
    true
}

pub(crate) fn update_callosity_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<CallosityFamilyState>(key))
        .is_some_and(|state| state.expired(now_ms))
    { return false; }
    end_callosity_state_key(game, region_id, holder, key)
}
