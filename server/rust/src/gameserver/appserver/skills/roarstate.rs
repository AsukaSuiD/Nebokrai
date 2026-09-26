//! Живая установка, visual и End подавления атаки CRoarState.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/roarstate.cpp/.h.
//! Данные и числовые правила находятся в Zone `effects/roar.rs`; тела замены,
//! restart, AI и End перенесены буквально в Zone `skills/roarstate.rs`
//! (порция №6c «self/zone-касты»; основание и машинные статусы см. там).
//! Здесь — тонкие делегации с прежними сигнатурами; потребители не меняются.

use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_zone::skills::roarstate as zone_state;

// Безпотребительный реэкспорт `RoarState/ROAR_STATE_ID/ROAR_STATE_BYTES`
// со старого пути снят порцией №6c (живые типы адресуются из Zone напрямую).

pub(super) fn replace_roar_state(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::replace_roar_state(game, source, target, properties, now)
}

pub(crate) fn update_roar_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::update_roar_state_properties(game, region_id, holder, key, now)
}

pub(crate) fn restart_roar_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::restart_roar_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn update_roar_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    zone_state::update_roar_state(game, region_id, holder, key, now_ms)
}

pub(crate) fn end_roar_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    zone_state::end_roar_state(game, region_id, holder, key)
}
