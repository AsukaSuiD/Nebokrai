//! Накопление и расход энергии CEnergyHoldingState.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/energyholdingstate.cpp.
//! Данные, signedness лимита, 12-байтная запись и формула множителя — Zone
//! `effects/energyholding.rs`; живые add/consume/restart/End перенесены буквально
//! в Zone `skills/energyholdingstate.rs` (порция №6c «self/zone-касты»; основание
//! и машинные статусы см. там). Здесь — тонкие делегации с прежними
//! сигнатурами и impl машины накопления `accumulatedstate` (её владелец
//! поделён с SoulCollect и остаётся в этом пакете); потребители не меняются.

use super::accumulatedstate::{AccumulatedState, AccumulationParticipant};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_zone::skills::energyholdingstate as zone_state;

pub(crate) use nebokrai_zone::effects::{
    ENERGY_HOLDING_STATE_BYTES, ENERGY_HOLDING_STATE_ID, EnergyHoldingState,
};

impl AccumulatedState for EnergyHoldingState {
    const ID: u32 = ENERGY_HOLDING_STATE_ID;
    const PARTICIPANT: AccumulationParticipant = AccumulationParticipant::User;
    fn increment(&mut self) -> bool { self.add_energy() }
    fn record(self) -> [u8; ENERGY_HOLDING_STATE_BYTES] { self.encoded() }
    fn client_fields(self) -> (u32, u32) { EnergyHoldingState::client_fields(self) }
}

pub(crate) fn typed_first_energy_holding(
    game: &CGame, source: (i32, ShapeIdentity),
) -> Option<&EnergyHoldingState> {
    zone_state::typed_first_energy_holding(game, source)
}

/// Параметры нового ctor запрашиваются только при отсутствии typed первого
/// ID89. Существующий заряд сохраняет уровень и процент первоначального Begin.
pub(crate) fn add_energy_holding(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce(&CGame) -> Option<EnergyHoldingState>, now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::add_energy_holding(game, source, create, now)
}

pub(crate) fn consume_energy_holding_multiplier(game: &mut CGame, source: (i32, ShapeIdentity)) -> f64 {
    zone_state::consume_energy_holding_multiplier(game, source)
}

pub(crate) fn clear_energy_holding(game: &mut CGame, source: (i32, ShapeIdentity)) -> bool {
    zone_state::clear_energy_holding(game, source)
}

pub(crate) fn restart_energy_holding_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::restart_energy_holding_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn end_energy_holding_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    zone_state::end_energy_holding_state(game, region_id, holder, key)
}
