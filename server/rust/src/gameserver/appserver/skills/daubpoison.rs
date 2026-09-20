//! Смазка оружия ядом CDaubPoison (0xDF).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/daubpoison.cpp.
//! Общий selfstatecast сохраняет Begin с исходной целью, Check исходного U,
//! MP, OnChangeStates, visual и End. Применение и visual используют только U:
//! запрошенная S не становится получателем смазки.
//!
//! После visual1 завершается первый непустой слот ID0xDF без фильтра RTTI
//! или ended; свежий остаток той же позиции уничтожается. Только затем
//! прежняя таблица AI отдаёт persist. Primary Begin(U,U) получает свои часы,
//! публикует состояние до append; отдельного UpdateProperty после него нет.
//! Существующий state owner хранит единственный payload и обслуживает wire,
//! DB и снятие. Результат установки не отменяет завершение навыка End(1).

use super::daubpoisonstate::{DAUB_POISON_STATE_ID, begin_primary_daub_poison_state};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_and_destroy_state_at, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const DAUB_POISON_SKILL_ID: u32 = DAUB_POISON_STATE_ID;
const STATE_PERSIST_TIME: u32 = 10_002;

pub(super) fn apply_daub_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    if let Some((position, _)) = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == DAUB_POISON_STATE_ID))
    {
        let _ = end_and_destroy_state_at(game, source.0, source.1, position);
    }
    let keep = properties.query_property(STATE_PERSIST_TIME);
    let _ = begin_primary_daub_poison_state(game, source, keep, &mut || runtime.now_milliseconds());
}
