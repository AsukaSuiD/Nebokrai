//! Защитная стойка CPillarState и её переключение повторным применением.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/pillarstate.cpp.
//!
//! Первый непустой ID74 снимается через End и destructor свежего остатка
//! той же позиции; новый экземпляр при этом не создаётся. Без такого слота
//! Begin(U,U) публикует loop1 visual, запрещает движение и лишь затем
//! передаёт состояние общей SlotMap-арене. После попытки нового Begin
//! вызывается UpdateProperty, но ветвь снятия не добавляет второй вызов.
//! Само состояние не меняет свойства: коэффициент читает поздний PostDefense.
//!
//! End публикует снятие, заново разрешает S, снимает один запрет движения и
//! удаляет именно этот объект из арены S. Он не записывает ended: чужой либо
//! отсутствующий S оставляет запись до внешнего destructor. AI сравнивает
//! unsigned wrapping start+keep строго с now, без особой ветви keep=0.
//! NULL-user restart сохраняет начало срока и источник, создаёт новый visual
//! и запрещает движение исходному S после публикации, не заменяя запись.
//!
//! DB: ID/remaining/IEEE-754 factor (12 байт). Load читает часы до двух полей,
//! restart их не обновляет. Клиентский срок читает часы один либо два раза;
//! additional равен нулю. SetRegion меняет только регион сохранённого U.

use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_base_applied_state, begin_applied_state_visual,
    end_and_destroy_state_at, remove_applied_state_from, resolve_applied_state_sufferer,
    update_applied_state_end_visual, update_property_state_visual,
};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const PILLAR_STATE_ID: u32 = 0x74;
pub(crate) const PILLAR_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PillarState { started_at_ms: u32, keep_time_ms: u32, damage_factor_bits: u32 }

impl PillarState {
    pub(crate) const fn new(keep_time_ms: u32, damage_factor: f32) -> Self {
        Self { started_at_ms: 0, keep_time_ms, damage_factor_bits: damage_factor.to_bits() }
    }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != PILLAR_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let remaining = reader.read_u32()?;
        Ok(Self { started_at_ms: now_ms, keep_time_ms: remaining, damage_factor_bits: reader.read_u32()? })
    }

    pub(crate) fn encoded_for_install(self) -> [u8; PILLAR_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; PILLAR_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; PILLAR_STATE_BYTES] {
        let mut bytes = [0; PILLAR_STATE_BYTES];
        bytes[..4].copy_from_slice(&PILLAR_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining.to_le_bytes());
        bytes[8..].copy_from_slice(&self.damage_factor_bits.to_le_bytes());
        bytes
    }
    pub(crate) const fn skill_id(self) -> u32 { PILLAR_STATE_ID }
    pub(crate) const fn damage_factor(self) -> f32 { f32::from_bits(self.damage_factor_bits) }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32 }
}

pub(crate) fn toggle_pillar_state(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce(&CGame) -> Option<PillarState>, now: &mut dyn FnMut() -> u32,
) -> bool {
    let previous = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == PILLAR_STATE_ID));
    if let Some((position, _)) = previous {
        return end_and_destroy_state_at(game, source.0, source.1, position).is_some();
    }
    let Some(mut state) = create(game) else { return false; };
    let begun = (|| {
        resolve_state_move_shape(game, source.0, source.1)?;
        state.started_at_ms = now();
        let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
        let participant = (shape.get_region_id(), ShapeIdentity {
            ex_id: CGuid::GUID_INVALID, ..shape.identity()
        });
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(shape.identity().object_type);
        message.add_long(shape.identity().id);
        message.add_ulong(PILLAR_STATE_ID);
        message.add_long(state.client_time(&mut *now));
        message.add_ulong(0);
        let _ = game.send_move_shape_around(participant.0, participant.1, &message);
        // Объектный Begin запрещает движение исходному param_2 после visual.
        let shape = resolve_state_move_shape_mut(game, source.0, source.1)?;
        shape.set_moveable(false);
        let record = state.encoded_for_install();
        let key = shape.append_applied_state_record(state, &record);
        shape.mark_applied_state_begun(key);
        shape.set_applied_state_user(key, Some(participant));
        shape.set_applied_state_sufferer(key, Some(participant));
        Some(())
    })().is_some();
    let _ = game.update_move_shape_properties(source.0, source.1);
    begun
}

pub(crate) fn restart_pillar_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key)).is_none() { return false; }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_property_state_visual::<PillarState>(
            game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
            |state, now| state.client_time(now) as u32,
        );
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_moveable(false);
    }
    true
}

pub(crate) fn update_pillar_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_pillar_state(game, region_id, holder, key)
}

pub(crate) fn end_pillar_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<PillarState>(key)).is_none() { return false; }
    update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return false; };
    if let Some(shape) = resolve_state_move_shape_mut(game, target.0, target.1) {
        shape.set_moveable(true);
    }
    remove_applied_state_from(game, region_id, holder, key, target, PILLAR_STATE_BYTES)
}
