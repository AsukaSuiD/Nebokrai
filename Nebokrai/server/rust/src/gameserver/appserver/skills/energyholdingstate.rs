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

use super::accumulatedstate::{
    AccumulatedState, AccumulationParticipant, add_accumulated_state, update_accumulated_visual,
};
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, end_and_destroy_state_at,
    remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::CGame;

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

impl AccumulatedState for EnergyHoldingState {
    const ID: u32 = ENERGY_HOLDING_STATE_ID;
    const PARTICIPANT: AccumulationParticipant = AccumulationParticipant::User;
    fn increment(&mut self) -> bool { self.add_energy() }
    fn record(self) -> [u8; ENERGY_HOLDING_STATE_BYTES] { self.encoded() }
    fn client_fields(self) -> (u32, u32) { (0, 0) }
}

/// Параметры нового ctor запрашиваются только при отсутствии typed первого
/// ID89. Существующий заряд сохраняет уровень и процент первоначального Begin.
pub(crate) fn add_energy_holding(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce(&CGame) -> Option<EnergyHoldingState>, now: &mut dyn FnMut() -> u32,
) -> bool {
    add_accumulated_state(game, source, create, now)
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
    update_accumulated_visual::<EnergyHoldingState>(game, (region_id, holder), key, 2);
    if !resolve_state_move_shape_mut(game, region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_ended(key)) { return false; }
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return false; };
    remove_applied_state_from(game, region_id, holder, key, target, ENERGY_HOLDING_STATE_BYTES)
}
