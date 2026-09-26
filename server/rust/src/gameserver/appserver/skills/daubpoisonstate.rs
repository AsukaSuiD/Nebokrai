//! Тонкий путь к живым restart/update/End CDaubPoisonState в Zone.
//! Источник поведения: gameserver.exe/GameServer.pdb,
//! `appserver/skills/daubpoisonstate.cpp/.h`. Тела Begin/restart/update/End
//! перенесены буквально в `nebokrai_zone::skills::daubpoisonstate`
//! (основание и статусы MATCH — в шапке Zone-файла; кластер D, порция D4);
//! данные, срок и сохраняемая запись — Zone `effects/daubpoison.rs`
//! (перенесены раньше). Первичный Begin вызывается только из Zone
//! `skills/daubpoison.rs` (ветка применения CDaubPoison) и здесь не
//! дублируется. Здесь — делегации с прежними сигнатурами; обход состояний
//! (`states/state.rs`) не меняется.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;

pub(crate) fn restart_daub_poison_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::daubpoisonstate::restart_daub_poison_state(
        game, region_id, holder, key, changing_region, now,
    )
}

pub(crate) fn update_daub_poison_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    nebokrai_zone::skills::daubpoisonstate::update_daub_poison_state(
        game, region_id, holder, key, now_ms,
    )
}

pub(crate) fn end_daub_poison_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    nebokrai_zone::skills::daubpoisonstate::end_daub_poison_state(game, region_id, holder, key)
}
