//! Живой путь CCallosityState/CCallosityState2 (0x75/0x7d).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/callositystate.cpp/.h
//! и callositystate2.cpp/.h. Данные, срок, запись и формула находятся в Zone
//! `effects/callosity.rs`; тела замены, restart, AI и End перенесены буквально
//! в Zone `skills/callositystate.rs` (порция №6c «self/zone-касты»; основание
//! и машинные статусы см. там). Здесь — тонкие делегации с прежними
//! сигнатурами; потребители не меняются.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_zone::skills::callositystate as zone_state;

pub(crate) use nebokrai_zone::effects::CallosityFamilyState;
// Безпотребительный `CALLOSITY_STATE_BYTES` со старого пути снят порцией №6c
// (ширину записи читают из Zone напрямую).

pub(crate) fn replace_callosity_state(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce() -> CallosityFamilyState, now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::replace_callosity_state(game, source, create, now)
}

pub(crate) fn end_callosity_state_key(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    zone_state::end_callosity_state_key(game, region_id, holder, key)
}

pub(crate) fn update_callosity_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::update_callosity_state_properties(game, region_id, holder, key, now)
}

pub(crate) fn restart_callosity_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::restart_callosity_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn update_callosity_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    zone_state::update_callosity_state(game, region_id, holder, key, now_ms)
}
