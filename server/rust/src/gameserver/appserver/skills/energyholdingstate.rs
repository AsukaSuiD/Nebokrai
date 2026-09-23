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
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, end_and_destroy_state_at,
    remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::effects::{
    ENERGY_HOLDING_STATE_BYTES, ENERGY_HOLDING_STATE_ID, EnergyHoldingState,
};

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
    fn client_fields(self) -> (u32, u32) { EnergyHoldingState::client_fields(self) }
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
        .map_or(1.0, |state| f64::from(state.energy_count()) * f64::from(state.parameter_percent()) * 0.01 + 1.0);
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
