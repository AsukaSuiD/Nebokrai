//! Живые Begin, restart и End CCureState в переходном Game.
//! Источник поведения: appserver/skills/curestate.cpp/.h. Тела перенесены
//! буквально в Zone `skills/curestate.rs` (порция №6a «state-касты пятёрки
//! + heal-квартет»; основание — codec fold Serialize `0x1F51E0`,
//! Unserialize `0x1E9AC0` — см. там). Здесь — делегации с прежними
//! сигнатурами: обход состояний, producer-ы (cure-излучатели, lifeshield,
//! ragebreak, fury) и потребители не меняются.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::effects::{CURE_STATE_BYTES, CURE_STATE_SKILL_ID, CureState};

/// Здесь Begin нового состояния предшествует поиску и End старого:
/// во время его visual новый экземпляр ещё не принадлежит вектору состояний.
pub(crate) fn begin_and_replace_cure_state(
    game: &mut CGame, user: Option<(i32, ShapeIdentity)>, sufferer: Option<(i32, ShapeIdentity)>,
    state: CureState, now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    nebokrai_zone::skills::curestate::begin_and_replace_cure_state(game, user, sufferer, state, now)
}

/// Полный Begin и append без неявного поиска/End прежнего Cure.
pub(crate) fn begin_primary_cure_state(
    game: &mut CGame, holder_region: i32, holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>, sufferer: Option<(i32, ShapeIdentity)>,
    state: CureState, now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    nebokrai_zone::skills::curestate::begin_primary_cure_state(game, holder_region, holder, user, sufferer, state, now)
}

pub(crate) fn restart_cure_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::curestate::restart_cure_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn end_player_cure_state(game: &mut CGame, player_id: i32) -> bool {
    nebokrai_zone::skills::curestate::end_player_cure_state(game, player_id)
}

pub(crate) fn end_player_cure_state_key(game: &mut CGame, player_id: i32, key: StateKey) -> bool {
    nebokrai_zone::skills::curestate::end_player_cure_state_key(game, player_id, key)
}

pub(crate) fn update_cure_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    nebokrai_zone::skills::curestate::update_cure_state(game, region_id, holder, key, now_ms)
}

/// Полный End состояния по ключу внутри общего обхода владельца.
pub(crate) fn end_cure_state_key(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    nebokrai_zone::skills::curestate::end_cure_state_key(game, region_id, holder, key)
}
