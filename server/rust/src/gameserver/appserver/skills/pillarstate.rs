//! Живое переключение, visual и End защитной стойки CPillarState.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/pillarstate.cpp/.h.
//! Данные, срок и сохраняемая запись находятся в Zone `effects/pillar.rs`;
//! тела перенесены буквально в Zone `skills/pillarstate.rs` (порция №6c
//! «self/zone-касты»; основание и машинные статусы см. там). Здесь — тонкие
//! делегации с прежними сигнатурами; потребители не меняются.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_zone::skills::pillarstate as zone_state;

pub(crate) use nebokrai_zone::effects::PillarState;
// Безпотребительные `PILLAR_STATE_BYTES/PILLAR_STATE_ID` со старого пути
// сняты порцией №6c (значения читают из Zone напрямую).

pub(crate) fn toggle_pillar_state(
    game: &mut CGame, source: (i32, ShapeIdentity),
    create: impl FnOnce(&CGame) -> Option<PillarState>, now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::toggle_pillar_state(game, source, create, now)
}

pub(crate) fn restart_pillar_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    zone_state::restart_pillar_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn update_pillar_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey, now_ms: u32,
) -> bool {
    zone_state::update_pillar_state(game, region_id, holder, key, now_ms)
}

pub(crate) fn end_pillar_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    zone_state::end_pillar_state(game, region_id, holder, key)
}
