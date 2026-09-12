//! Накопление и расход энергии CEnergyHoldingState.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/energyholdingstate.cpp.
//!
//! Заряд ограничен сохранённым уровнем. AddEnergy требует живого GetUser и
//! публикует снятие, затем установку; loop1 остаётся живым между пакетами.
//! Первичный Begin ничего не публикует, первый Add предшествует push_back.
//! End разрешает Sufferer заново и удаляет именно этот экземпляр его арены.
//! InverseChopped выбирает первый непустой слот ID89, включая ended, вызывает
//! End и уничтожает свежий остаток той же позиции даже при несовпадении RTTI.
//!
//! DB содержит только ID/уровень/число зарядов (12 байт). Процент не сохранён:
//! фабрика вызывает пустой ctor, Unserialize оставляет его равным нулю.
//! Клиентские remaining/additional тоже равны нулю, а не проценту усиления.
//! Владение и порядок слотов обеспечивает общая SlotMap-арена; локальный
//! visual первичного Begin оставляет ей незавершённое loop1-состояние.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, end_and_destroy_state_at,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_applied_state_user,
    resolve_state_move_shape, resolve_state_move_shape_mut, update_applied_state_visual_base,
};
use crate::gameserver::appserver::states::visualeffect::CVisualEffect;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const ENERGY_HOLDING_STATE_ID: u32 = 0x89;
pub(crate) const ENERGY_HOLDING_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EnergyHoldingState {
    skill_level: u32,
    energy_count: u32,
    parameter_percent: u32,
}

impl EnergyHoldingState {
    pub(crate) const fn new(skill_level: u32, parameter_percent: u32) -> Self {
        Self { skill_level, energy_count: 0, parameter_percent }
    }
    pub(crate) const fn skill_id(self) -> u32 { ENERGY_HOLDING_STATE_ID }
    pub(crate) const fn skill_level(self) -> u32 { self.skill_level }
    pub(crate) const fn energy_count(self) -> u32 { self.energy_count }

    fn add_energy(&mut self) -> bool {
        if self.energy_count >= self.skill_level { return false; }
        self.energy_count = self.energy_count.wrapping_add(1);
        true
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ENERGY_HOLDING_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self {
            skill_level: reader.read_u32()?, energy_count: reader.read_u32()?, parameter_percent: 0,
        })
    }

    pub(crate) fn encoded(self) -> [u8; ENERGY_HOLDING_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(ENERGY_HOLDING_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(ENERGY_HOLDING_STATE_ID);
        writer.write_u32(self.skill_level);
        writer.write_u32(self.energy_count);
        bytes.try_into().expect("размер состояния накопления энергии фиксирован")
    }
}

fn first_energy_slot(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(usize, StateKey)> {
    resolve_state_move_shape(game, source.0, source.1)?
        .find_state_position(|state| state.state_id() == ENERGY_HOLDING_STATE_ID)
}

pub(crate) fn typed_first_energy_holding(
    game: &CGame, source: (i32, ShapeIdentity),
) -> Option<&EnergyHoldingState> {
    let (_, key) = first_energy_slot(game, source)?;
    resolve_state_move_shape(game, source.0, source.1)?.applied_state(key)
}

fn publish_energy_visual(game: &mut CGame, target: (i32, ShapeIdentity), mode: u32) {
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let region = shape.shape().get_region_id();
    let identity = shape.shape().identity();
    let mut message = CMessage::new(if mode == 1 { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_ulong(ENERGY_HOLDING_STATE_ID);
    if mode == 1 {
        message.add_ulong(0);
        message.add_ulong(0);
    }
    let _ = game.send_move_shape_around(region, identity, &message);
}

fn update_energy_visual(game: &mut CGame, source: (i32, ShapeIdentity), key: StateKey, mode: u32) {
    let Some(ended) = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.applied_state_visual_ended(key)) else { return; };
    if !ended {
        if let Some(target) = resolve_applied_state_sufferer(game, source.0, source.1, key) {
            publish_energy_visual(game, target, mode);
        }
    }
    update_applied_state_visual_base(game, source.0, source.1, key);
}

/// Параметры нового ctor запрашиваются только при отсутствии typed первого
/// ID89. Существующий заряд сохраняет уровень и процент первоначального Begin.
pub(crate) fn add_energy_holding(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce(&CGame) -> Option<EnergyHoldingState>, now: &mut dyn FnMut() -> u32,
) -> bool {
    if let Some((_, key)) = first_energy_slot(game, source) {
        if resolve_state_move_shape(game, source.0, source.1)
            .and_then(|shape| shape.applied_state::<EnergyHoldingState>(key)).is_some() {
            if resolve_applied_state_user(game, source.0, source.1, key).is_some() {
                let added = resolve_state_move_shape_mut(game, source.0, source.1)
                    .and_then(|shape| shape.applied_state_mut::<EnergyHoldingState>(key))
                    .is_some_and(EnergyHoldingState::add_energy);
                if added {
                    update_energy_visual(game, source, key, 2);
                    update_energy_visual(game, source, key, 1);
                }
            }
            return true;
        }
    }
    let Some(mut state) = create(game) else { return false; };
    // CState::Begin с ненулевым user читает часы, хотя Energy не использует их.
    let _ = now();
    let Some(shape) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
    let participant = (shape.shape().get_region_id(), ShapeIdentity {
        ex_id: CGuid::GUID_INVALID, ..shape.shape().identity()
    });
    let mut visual = CVisualEffect::new();
    visual.begin_visual_effect(1);
    if resolve_state_move_shape(game, participant.0, participant.1).is_some() && state.add_energy() {
        publish_energy_visual(game, participant, 2);
        visual.update_visual_effect();
        publish_energy_visual(game, participant, 1);
        visual.update_visual_effect();
    }
    let Some(shape) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    let record = state.encoded();
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(participant));
    shape.set_applied_state_sufferer(key, Some(participant));
    true
}

pub(crate) fn consume_energy_holding_multiplier(game: &mut CGame, source: (i32, ShapeIdentity)) -> f64 {
    let Some((position, key)) = first_energy_slot(game, source) else { return 1.0; };
    let multiplier = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.applied_state::<EnergyHoldingState>(key))
        .map_or(1.0, |state| f64::from(state.energy_count) * f64::from(state.parameter_percent) * 0.01 + 1.0);
    let _ = end_and_destroy_state_at(game, source.0, source.1, position);
    multiplier
}

pub(crate) fn clear_energy_holding(game: &mut CGame, source: (i32, ShapeIdentity)) -> bool {
    let Some((position, _)) = first_energy_slot(game, source) else { return false; };
    end_and_destroy_state_at(game, source.0, source.1, position).is_some()
}

pub(crate) fn restart_energy_holding_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnergyHoldingState>(key)).is_none() { return false; }
    begin_base_applied_state(game, region_id, holder, key)
        && begin_applied_state_visual(game, region_id, holder, key, 1)
}

pub(crate) fn end_energy_holding_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnergyHoldingState>(key)).is_none() { return false; }
    update_energy_visual(game, (region_id, holder), key, 2);
    if !resolve_state_move_shape_mut(game, region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_ended(key)) { return false; }
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, ENERGY_HOLDING_STATE_BYTES)
}
