//! Накопление и расход энергии CEnergyHoldingState.
//! Данные, signedness лимита, 12-байтная запись и формула множителя —
//! `effects/energyholding.rs`.
//!
//! Quirks: заряд ограничен сохранённым уровнем; первичный Begin ничего не
//! публикует, первый Add предшествует push_back; InverseChopped выбирает
//! первый непустой слот ID89 включая ended (End + dtor даже при несовпадении
//! RTTI); процент не сохранён в DB — Unserialize оставляет его нулём, и
//! клиентские remaining/additional тоже нулевые, а не процент усиления.
//!
//! Швы: hub `selfcast::SelfCastGame`; машина `accumulatedstate` — швы
//! `add_energy_holding_state` и `update_energy_holding_accumulated_visual`
//! (пока у старого пакета, поделена с SoulCollect).
//!
//! Исходный владелец PDB: `appserver/skills/energyholdingstate.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#roarcpillarcallosityenergyholding--self-касты

use crate::regions::ShapeIdentity;

use super::selfcast::{SelfCastGame, SelfCastMoveShape};
use super::state::StateKey;

pub use crate::effects::{ENERGY_HOLDING_STATE_BYTES, ENERGY_HOLDING_STATE_ID, EnergyHoldingState};

fn first_energy_slot<Game: SelfCastGame>(game: &Game, source: (i32, ShapeIdentity)) -> Option<(usize, StateKey)> {
    game.resolve_state_move_shape(source.0, source.1)?
        .find_state_position(|state| state.state_id() == ENERGY_HOLDING_STATE_ID)
}

pub fn typed_first_energy_holding<Game: SelfCastGame>(
    game: &Game, source: (i32, ShapeIdentity),
) -> Option<&EnergyHoldingState> {
    let (_, key) = first_energy_slot(game, source)?;
    game.resolve_state_move_shape(source.0, source.1)?.applied_state(key)
}

/// Параметры нового ctor запрашиваются только при отсутствии typed первого
/// ID89. Существующий заряд сохраняет уровень и процент первоначального Begin.
pub fn add_energy_holding<Game: SelfCastGame>(
    game: &mut Game, source: (i32, ShapeIdentity),
    create: impl FnOnce(&Game) -> Option<EnergyHoldingState>, now: &mut dyn FnMut() -> u32,
) -> bool {
    game.add_energy_holding_state(source, create, now)
}

pub fn consume_energy_holding_multiplier<Game: SelfCastGame>(game: &mut Game, source: (i32, ShapeIdentity)) -> f64 {
    let Some((position, key)) = first_energy_slot(game, source) else { return 1.0; };
    let multiplier = game.resolve_state_move_shape(source.0, source.1)
        .and_then(|shape| shape.applied_state::<EnergyHoldingState>(key))
        .map_or(1.0, |state| state.attack_multiplier());
    let _ = game.end_and_destroy_state_at(source.0, source.1, position);
    multiplier
}

pub fn clear_energy_holding<Game: SelfCastGame>(game: &mut Game, source: (i32, ShapeIdentity)) -> bool {
    let Some((position, _)) = first_energy_slot(game, source) else { return false; };
    game.end_and_destroy_state_at(source.0, source.1, position)
}

pub fn restart_energy_holding_state<Game: SelfCastGame>(
    game: &mut Game, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, _now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<EnergyHoldingState>(key)).is_none() { return false; }
    game.begin_base_applied_state(region_id, holder, key)
        && game.begin_applied_state_visual(region_id, holder, key, 1)
}

pub fn end_energy_holding_state<Game: SelfCastGame>(
    game: &mut Game, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<EnergyHoldingState>(key)).is_none() { return false; }
    game.update_energy_holding_accumulated_visual(region_id, holder, key, 2);
    if !game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_ended(key)) { return false; }
    let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key) else { return false; };
    game.remove_applied_state_from(region_id, holder, key, target, ENERGY_HOLDING_STATE_BYTES)
}
