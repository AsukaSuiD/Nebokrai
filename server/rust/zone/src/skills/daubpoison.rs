//! Смазка оружия ядом `CDaubPoison` (`0xDF`): ID навыка, правило срока нового
//! состояния и тело применения после visual(1) с семейной заменой первого
//! непустого 0xDF-слота. Скелет Begin/Check/AI принадлежит hub `selfstatecast`
//! старого пакета (общий для пяти усилений), обвязка состояния —
//! `skills/daubpoisonstate.rs` рядом.
//!
//! Машинные quirks: замена ищет первый НЕпустой слот id `0xDF` без
//! RTTI/ended-фильтра; keep-time ctor-а — только из Query(10002); отказ Begin
//! state не отменяет End(1) навыка (проверки результата установки нет).
//!
//! Источник не типа Player отклоняется переходным hub `selfstatecast`
//! (нативный код выполнял небезопасный доступ к MP `[U+0x284]` без проверки
//! типа — основание в шапке hub); здесь воспроизводится только машинно
//! достижимый путь, поведение hub не меняется.
//!
//! Швы: hub `statecast::StateCastGame` (арена `find_state_position`/
//! `end_and_destroy_state_at`) и `daubpoisonstate::begin_primary_daub_poison_state`;
//! драйвер `selfstatecast.rs` старого пакета не меняется.
//!
//! Исходный владелец PDB: `appserver/skills/daubpoison.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#daubpoison--cdaubpoison-0xdf-и-cdaubpoisonstate

use crate::content::CSkillBaseProperties;
use crate::effects::DAUB_POISON_STATE_ID;
use crate::regions::ShapeIdentity;

use super::daubpoisonstate::begin_primary_daub_poison_state;
use super::statecast::{StateCastGame, StateCastMoveShape};

pub const DAUB_POISON_SKILL_ID: u32 = DAUB_POISON_STATE_ID;
const STATE_PERSIST_TIME: u32 = 10_002;

/// Единственный запрос срока нового состояния выполняется после завершения
/// прежнего слота; сам запрос остаётся у живого Game.
pub fn daub_poison_keep_time_ms(mut query_property: impl FnMut(u32) -> u32) -> u32 {
    query_property(STATE_PERSIST_TIME)
}

/// Применение из AI `0x566690` после visual(1): первый непустой слот `0xDF`
/// завершается с destructor-ом свежего остатка той же позиции, затем ctor
/// keep из `Query(10002)` и primary Begin(U,U) с append у того же держателя;
/// результат установки завершения навыка End(1) не отменяет.
pub fn apply_daub_poison<Game: StateCastGame>(
    game: &mut Game,
    source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    now: &mut dyn FnMut() -> u32,
) {
    if let Some((position, _)) = game
        .resolve_state_move_shape(source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == DAUB_POISON_STATE_ID))
    {
        let _ = game.end_and_destroy_state_at(source.0, source.1, position);
    }
    let keep = daub_poison_keep_time_ms(|key| properties.query_property(key));
    let _ = begin_primary_daub_poison_state(game, source, keep, now);
}
